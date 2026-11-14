# Crimson System Boost - Windows desktop GUI
# Windows 10/11, Windows PowerShell 5.1+, .NET Framework WinForms.
# Normal-user mode. No registry hacks, service disabling, overclocking, or anti-cheat bypass.

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[Windows.Forms.Application]::EnableVisualStyles()
$StateDir = Join-Path $env:LOCALAPPDATA "CrimsonSystemBoost"
$StateFile = Join-Path $StateDir "process-priority-backup.json"
$NL = [Environment]::NewLine
$InventoryModule = Join-Path $PSScriptRoot "modules\Crimson.Inventory.psm1"
if (Test-Path -LiteralPath $InventoryModule) {
    Import-Module $InventoryModule -Force -ErrorAction Stop
}
$C = @{
    Bg=[Drawing.Color]::FromArgb(17,19,24); Panel=[Drawing.Color]::FromArgb(27,31,39)
    Panel2=[Drawing.Color]::FromArgb(39,45,56); Border=[Drawing.Color]::FromArgb(69,76,89)
    Text=[Drawing.Color]::FromArgb(242,244,248); Muted=[Drawing.Color]::FromArgb(160,170,184)
    Red=[Drawing.Color]::FromArgb(229,72,77); Green=[Drawing.Color]::FromArgb(54,179,126)
}
function New-Button([string]$Text,[int]$X,[int]$Y,[int]$W,[int]$H=38,[bool]$Primary=$false) {
    $b=New-Object Windows.Forms.Button
    $b.Text=$Text; $b.Location=New-Object Drawing.Point($X,$Y); $b.Size=New-Object Drawing.Size($W,$H)
    $b.FlatStyle="Flat"; $b.FlatAppearance.BorderSize=1; $b.FlatAppearance.BorderColor=$C.Border
    $b.BackColor=if($Primary){$C.Red}else{$C.Panel2}; $b.ForeColor=$C.Text
    $b.Font=New-Object Drawing.Font("Segoe UI",9,[Drawing.FontStyle]::Bold)
    $b.Cursor=[Windows.Forms.Cursors]::Hand
    return $b
}
function Add-Log([string]$Message) {
    $script:Log.AppendText(("["+(Get-Date -Format "HH:mm:ss")+"] "+$Message+$NL))
    $script:Log.SelectionStart=$script:Log.TextLength; $script:Log.ScrollToCaret()
}
function Show-InstalledSoftware {
    try {
        $items = @(Get-CrimsonInstalledSoftware)
        Add-Log ("Installed software inventory: " + $items.Count + " entries.")
        foreach ($item in ($items | Select-Object -First 24)) {
            Add-Log ("  " + $item.Name + $(if ($item.Version) { "  v" + $item.Version } else { "" }))
        }
        if ($items.Count -gt 24) { Add-Log ("  plus " + ($items.Count - 24) + " more.") }
    } catch { Add-Log ("Inventory failed: " + $_.Exception.Message) }
}
function Show-DefenderStatus {
    try {
        $state = Get-CrimsonDefenderStatus
        Add-Log ("SECURITY: " + $state.Status)
        if ($state.Available) {
            Add-Log ("Real-time protection: " + $state.RealTimeProtectionEnabled)
            Add-Log ("Antivirus enabled: " + $state.AntivirusEnabled)
            Add-Log ("Signature age (days): " + $state.AntivirusSignatureAge)
        }
    } catch { Add-Log ("Defender status failed: " + $_.Exception.Message) }
}
function Request-DefenderQuickScan {
    $answer = [Windows.Forms.MessageBox]::Show(
        "Request Microsoft Defender to run a quick scan? Crimson will not replace or disable your installed antivirus.",
        "Confirm security scan",
        [Windows.Forms.MessageBoxButtons]::YesNo,
        [Windows.Forms.MessageBoxIcon]::Information
    )
    if ($answer -ne [Windows.Forms.DialogResult]::Yes) { Add-Log "Security scan cancelled."; return }
    try {
        $result = Start-CrimsonDefenderQuickScan -Confirm:$false
        Add-Log ([string]$result)
    } catch { Add-Log ("Defender scan request failed: " + $_.Exception.Message) }
}
function Refresh-Apps {
    $script:Apps.Items.Clear(); $script:AppIds=@{}
    $found=Get-Process | Where-Object { $_.MainWindowTitle } | Sort-Object ProcessName
    foreach($p in $found) {
        $item=("{0}  |  PID {1}" -f $p.ProcessName,$p.Id)
        [void]$script:Apps.Items.Add($item); $script:AppIds[$item]=$p.Id
    }
    Add-Log ("Loaded "+$found.Count+" visible applications.")
}
function Read-System {
    $os=Get-CimInstance Win32_OperatingSystem
    $cpu=Get-CimInstance Win32_Processor | Select-Object -First 1
    $ram=(Get-CimInstance Win32_PhysicalMemory | Measure-Object Capacity -Sum).Sum
    $gpu=Get-CimInstance Win32_VideoController | Select-Object -First 1
    $vol=Get-Volume -DriveLetter $env:SystemDrive.TrimEnd(':') -ErrorAction SilentlyContinue
    $ramText=if($ram){"{0:N1} GB" -f ($ram/1GB)}else{"Unavailable"}
    $hz=if($gpu -and $gpu.CurrentRefreshRate -gt 0){"$($gpu.CurrentRefreshRate) Hz"}else{"Unavailable"}
    $drive=if($vol){"{0:N1} GB free / {1:N1} GB" -f ($vol.SizeRemaining/1GB),($vol.Size/1GB)}else{"Unavailable"}
    $script:Info.Text=@(
        "OPERATING SYSTEM  $($os.Caption) ($($os.OSArchitecture))","",
        "PROCESSOR         $($cpu.Name)",
        "CPU LAYOUT        $($cpu.NumberOfCores) cores / $($cpu.NumberOfLogicalProcessors) threads",
        "INSTALLED RAM     $ramText",
        "GRAPHICS          $($gpu.Name)",
        "DISPLAY REPORT    $hz",
        "SYSTEM DRIVE      $drive","",
        "Display refresh rate is not the same as game FPS."
    ) -join $NL
    Add-Log "System information refreshed."
}
function Apply-Profile {
    $selected=[string]$script:Apps.SelectedItem
    if(-not $selected -or -not $script:AppIds.ContainsKey($selected)) {
        [Windows.Forms.MessageBox]::Show("Select a running application first.","Crimson") | Out-Null; return
    }
    try {
        $p=Get-Process -Id ([int]$script:AppIds[$selected]) -ErrorAction Stop
        New-Item -ItemType Directory -Force -Path $StateDir | Out-Null
        $saved=@()
        if(Test-Path $StateFile){try{$saved=@(Get-Content $StateFile -Raw | ConvertFrom-Json)}catch{$saved=@()}}
        $saved=@($saved | Where-Object {[int]$_.Id -ne $p.Id})
        $saved+= [pscustomobject]@{Id=$p.Id;Name=$p.ProcessName;Priority=$p.PriorityClass.ToString()}
        ConvertTo-Json -InputObject @($saved) -Depth 4 | Set-Content -LiteralPath $StateFile -Encoding UTF8
        $p.PriorityClass=[Diagnostics.ProcessPriorityClass]::AboveNormal
        Add-Log "AboveNormal requested for $($p.ProcessName), PID $($p.Id). No FPS gain is guaranteed."
    } catch { Add-Log "Priority request failed: $($_.Exception.Message)" }
}
function Restore-Profile {
    if(-not (Test-Path $StateFile)){Add-Log "No saved profile found.";return}
    $entries=@(Get-Content $StateFile -Raw | ConvertFrom-Json); $count=0
    foreach($e in $entries) {
        try {
            $p=Get-Process -Id ([int]$e.Id) -ErrorAction Stop
            if($p.ProcessName -eq [string]$e.Name) {
                $p.PriorityClass=[Enum]::Parse([Diagnostics.ProcessPriorityClass],[string]$e.Priority); $count++
            }
        } catch { Add-Log "Skipped PID $($e.Id); it may have exited." }
    }
    Add-Log "Restored $count process priority setting(s)."
}
function Get-TempFiles {
    $cutoff=(Get-Date).AddDays(-7)
    $roots=@($env:TEMP,(Join-Path $env:WINDIR "Temp")) | Where-Object {$_ -and (Test-Path $_)} | Select-Object -Unique
    foreach($root in $roots) {
        Get-ChildItem -LiteralPath $root -Force -File -ErrorAction SilentlyContinue |
            Where-Object {$_.LastWriteTime -lt $cutoff}
    }
}
function Preview-Cleanup {
    $files=@(Get-TempFiles); $bytes=($files | Measure-Object Length -Sum).Sum
    if(-not $bytes){$bytes=0}
    Add-Log ("Preview: "+$files.Count+" files older than 7 days; "+("{0:N1} MB" -f ($bytes/1MB))+" potential space.")
    foreach($f in ($files | Select-Object -First 15)){Add-Log ("  "+$f.FullName)}
    if($files.Count -gt 15){Add-Log ("  plus "+($files.Count-15)+" more file(s).")}
    Add-Log "Preview only; nothing was deleted."
}
function Clean-Temp {
    $files=@(Get-TempFiles)
    if($files.Count -eq 0){Add-Log "No eligible temporary files found.";return}
    $answer=[Windows.Forms.MessageBox]::Show(
        ("Delete "+$files.Count+" temporary file(s) older than seven days? Locked files will be skipped."),
        "Confirm cleanup",[Windows.Forms.MessageBoxButtons]::YesNo,[Windows.Forms.MessageBoxIcon]::Warning)
    if($answer -ne [Windows.Forms.DialogResult]::Yes){Add-Log "Cleanup cancelled.";return}
    $removed=0
    foreach($f in $files){try{Remove-Item -LiteralPath $f.FullName -Force -ErrorAction Stop;$removed++}catch{Add-Log ("Skipped locked file: "+$f.Name)}}
    Add-Log "Deleted $removed temporary file(s)."
}

$form=New-Object Windows.Forms.Form
$form.Text="Crimson System Boost - Windows"; $form.StartPosition="CenterScreen"
$form.Size=New-Object Drawing.Size(1000,700); $form.MinimumSize=New-Object Drawing.Size(900,620)
$form.BackColor=$C.Bg; $form.ForeColor=$C.Text; $form.Font=New-Object Drawing.Font("Segoe UI",9)
$header=New-Object Windows.Forms.Panel; $header.Dock="Top"; $header.Height=82; $header.BackColor=$C.Panel
$form.Controls.Add($header)
$brand=New-Object Windows.Forms.Label; $brand.Text="CRIMSON  /  SYSTEM BOOST"; $brand.Location=New-Object Drawing.Point(24,14)
$brand.AutoSize=$true; $brand.Font=New-Object Drawing.Font("Segoe UI",17,[Drawing.FontStyle]::Bold); $brand.ForeColor=$C.Text
$header.Controls.Add($brand)
$tag=New-Object Windows.Forms.Label; $tag.Text="WINDOWS DESKTOP  •  REVERSIBLE PERFORMANCE TOOLS"
$tag.Location=New-Object Drawing.Point(28,49); $tag.AutoSize=$true; $tag.ForeColor=$C.Muted
$tag.Font=New-Object Drawing.Font("Segoe UI",8); $header.Controls.Add($tag)
$left=New-Object Windows.Forms.Panel; $left.Location=New-Object Drawing.Point(20,102); $left.Size=New-Object Drawing.Size(285,550)
$left.Anchor="Top,Bottom,Left"; $left.BackColor=$C.Panel; $form.Controls.Add($left)
$right=New-Object Windows.Forms.Panel; $right.Location=New-Object Drawing.Point(320,102); $right.Size=New-Object Drawing.Size(640,550)
$right.Anchor="Top,Bottom,Left,Right"; $right.BackColor=$C.Panel; $form.Controls.Add($right)

$l1=New-Object Windows.Forms.Label; $l1.Text="GAME PROFILE"; $l1.Location=New-Object Drawing.Point(18,16)
$l1.AutoSize=$true; $l1.ForeColor=$C.Muted; $l1.Font=New-Object Drawing.Font("Segoe UI",10,[Drawing.FontStyle]::Bold); $left.Controls.Add($l1)
$l2=New-Object Windows.Forms.Label; $l2.Text="Running applications"; $l2.Location=New-Object Drawing.Point(18,48); $l2.AutoSize=$true; $left.Controls.Add($l2)
$script:Apps=New-Object Windows.Forms.ListBox; $script:Apps.Location=New-Object Drawing.Point(18,74); $script:Apps.Size=New-Object Drawing.Size(249,270)
$script:Apps.BackColor=$C.Panel2; $script:Apps.ForeColor=$C.Text; $script:Apps.BorderStyle="FixedSingle"; $script:Apps.IntegralHeight=$false
$left.Controls.Add($script:Apps)
$refresh=New-Button "Refresh" 18 356 112; $apply=New-Button "Apply profile" 144 356 123 38 $true
$restore=New-Button "Restore previous priority" 18 405 249
$left.Controls.AddRange(@($refresh,$apply,$restore))
$cleanTitle=New-Object Windows.Forms.Label; $cleanTitle.Text="SAFE TEMP CLEANER"; $cleanTitle.Location=New-Object Drawing.Point(18,472)
$cleanTitle.AutoSize=$true; $cleanTitle.ForeColor=$C.Muted; $cleanTitle.Font=New-Object Drawing.Font("Segoe UI",10,[Drawing.FontStyle]::Bold); $left.Controls.Add($cleanTitle)
$preview=New-Button "Preview" 18 506 112; $clean=New-Button "Delete old files" 144 506 123
$clean.FlatAppearance.BorderColor=$C.Red; $left.Controls.AddRange(@($preview,$clean))

$sys=New-Object Windows.Forms.Label; $sys.Text="SYSTEM OVERVIEW"; $sys.Location=New-Object Drawing.Point(18,16)
$sys.AutoSize=$true; $sys.ForeColor=$C.Muted; $sys.Font=New-Object Drawing.Font("Segoe UI",10,[Drawing.FontStyle]::Bold); $right.Controls.Add($sys)
$scan=New-Button "Refresh system information" 418 10 200; $right.Controls.Add($scan)
$script:Info=New-Object Windows.Forms.RichTextBox; $script:Info.Location=New-Object Drawing.Point(18,48); $script:Info.Size=New-Object Drawing.Size(600,205)
$script:Info.Anchor="Top,Left,Right"; $script:Info.ReadOnly=$true; $script:Info.BorderStyle="None"; $script:Info.BackColor=$C.Panel2
$script:Info.ForeColor=$C.Text; $script:Info.Font=New-Object Drawing.Font("Consolas",9); $right.Controls.Add($script:Info)
$securityTitle=New-Object Windows.Forms.Label; $securityTitle.Text="APP INVENTORY  /  NATIVE SECURITY"; $securityTitle.Location=New-Object Drawing.Point(18,268)
$securityTitle.AutoSize=$true; $securityTitle.ForeColor=$C.Muted; $securityTitle.Font=New-Object Drawing.Font("Segoe UI",10,[Drawing.FontStyle]::Bold); $right.Controls.Add($securityTitle)
$inventoryButton=New-Button "Installed software" 18 296 180
$defenderButton=New-Button "Protection status" 210 296 180
$securityScanButton=New-Button "Request quick scan" 402 296 180 38 $true
$right.Controls.AddRange(@($inventoryButton,$defenderButton,$securityScanButton))
$logTitle=New-Object Windows.Forms.Label; $logTitle.Text="ACTIVITY LOG"; $logTitle.Location=New-Object Drawing.Point(18,346)
$logTitle.AutoSize=$true; $logTitle.ForeColor=$C.Muted; $logTitle.Font=New-Object Drawing.Font("Segoe UI",10,[Drawing.FontStyle]::Bold); $right.Controls.Add($logTitle)
$script:Log=New-Object Windows.Forms.RichTextBox; $script:Log.Location=New-Object Drawing.Point(18,374); $script:Log.Size=New-Object Drawing.Size(600,132)
$script:Log.Anchor="Top,Bottom,Left,Right"; $script:Log.ReadOnly=$true; $script:Log.BorderStyle="None"; $script:Log.BackColor=$C.Panel2
$script:Log.ForeColor=$C.Text; $script:Log.Font=New-Object Drawing.Font("Consolas",8.5); $right.Controls.Add($script:Log)
$foot=New-Object Windows.Forms.Label; $foot.Text="No FPS guarantee  •  No anti-cheat bypass  •  No security or thermal disabling"
$foot.Location=New-Object Drawing.Point(18,518); $foot.AutoSize=$true; $foot.ForeColor=$C.Muted
$foot.Font=New-Object Drawing.Font("Segoe UI",8); $right.Controls.Add($foot)

$refresh.Add_Click({Refresh-Apps}); $apply.Add_Click({Apply-Profile}); $restore.Add_Click({Restore-Profile})
$inventoryButton.Add_Click({Show-InstalledSoftware}); $defenderButton.Add_Click({Show-DefenderStatus}); $securityScanButton.Add_Click({Request-DefenderQuickScan})
$preview.Add_Click({Preview-Cleanup}); $clean.Add_Click({Clean-Temp}); $scan.Add_Click({try{Read-System}catch{Add-Log ("System scan failed: "+$_.Exception.Message)}})
$form.Add_Shown({Refresh-Apps; $scan.PerformClick(); Add-Log "Ready. Select a visible game process to apply a reversible priority request."})
[void]$form.ShowDialog()
