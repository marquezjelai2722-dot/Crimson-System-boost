# Crimson System Boost - Windows desktop GUI
# Windows 10/11, Windows PowerShell 5.1+, .NET Framework WinForms.
# Normal-user mode. No registry hacks, service disabling, overclocking, or anti-cheat bypass.

$ErrorActionPreference = "Stop"

# Optional source modules. The release builder also embeds their functions into the EXE.
$moduleRoot = Join-Path $PSScriptRoot "modules"
if (Test-Path -LiteralPath $moduleRoot) {
    foreach ($moduleName in @("Crimson.Inventory.psm1","Crimson.Security.psm1","Crimson.Maintenance.psm1","Crimson.Performance.psm1")) {
        $modulePath = Join-Path $moduleRoot $moduleName
        if (Test-Path -LiteralPath $modulePath) { Import-Module $modulePath -Force -ErrorAction Stop }
    }
}

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
[Windows.Forms.Application]::EnableVisualStyles()
$StateDir = Join-Path $env:LOCALAPPDATA "CrimsonSystemBoost"
$StateFile = Join-Path $StateDir "process-priority-backup.json"
$NL = [Environment]::NewLine
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
    $defender=Get-CrimsonDefenderStatus
    $script:Info.Text=@(
        "OPERATING SYSTEM  $($os.Caption) ($($os.OSArchitecture))","",
        "PROCESSOR         $($cpu.Name)",
        "CPU LAYOUT        $($cpu.NumberOfCores) cores / $($cpu.NumberOfLogicalProcessors) threads",
        "INSTALLED RAM     $ramText",
        "GRAPHICS          $($gpu.Name)",
        "DISPLAY REPORT    $hz",
        "SYSTEM DRIVE      $drive",
        "SECURITY STATUS   $($defender.Status)","",
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
    Get-CrimsonTemporaryFiles -OlderThanDays 7
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
    $result = Remove-CrimsonTemporaryFiles -Files $files -Confirm:$false
    Add-Log ("Deleted {0} temporary file(s); skipped {1} locked or inaccessible file(s)." -f $result.Removed,$result.Skipped)
}


function Show-InstalledSoftware {
    try {
        $items = @(Get-CrimsonInstalledSoftware)
        $large = @($items | Where-Object { $_.LargeFootprint })
        Add-Log ("Installed software: {0}; large reported footprint (>=1 GB): {1}." -f $items.Count,$large.Count)
        $lines = $items | Select-Object -First 100 | ForEach-Object {
            $size = if ($null -ne $_.EstimatedBytes) { "{0:N1} GB" -f ($_.EstimatedBytes/1GB) } else { "size unknown" }
            $tag = if ($_.LargeFootprint) { "  [LARGE INSTALL]" } else { "" }
            "{0}  |  {1}  |  {2}{3}" -f $_.Name,$size,$_.Publisher,$tag
        }
        [Windows.Forms.MessageBox]::Show(($lines -join $NL),"Installed software inventory") | Out-Null
    } catch { Add-Log ("Software inventory failed: "+$_.Exception.Message) }
}
function Show-ResourceUse {
    try {
        $items = @(Get-CrimsonResourceHeavyProcesses -Top 15)
        $lines = $items | ForEach-Object { "{0}  PID {1}  |  {2} MB RAM  |  CPU time {3}s" -f $_.ProcessName,$_.Id,$_.MemoryMB,$_.CpuSeconds }
        [Windows.Forms.MessageBox]::Show(($lines -join $NL),"Largest process memory use") | Out-Null
    } catch { Add-Log ("Resource report failed: "+$_.Exception.Message) }
}
function Invoke-DefenderFullScan {
    try {
        $status = Get-CrimsonDefenderStatus
        if (-not $status.Available) {
            [Windows.Forms.MessageBox]::Show($status.Status,"Security scan unavailable") | Out-Null
            return
        }
        if (-not $status.Enabled) {
            [Windows.Forms.MessageBox]::Show("Microsoft Defender is not active. Crimson will not claim the system is clean.","Protection unavailable") | Out-Null
            return
        }
        $answer = [Windows.Forms.MessageBox]::Show(
            "Start a Microsoft Defender full scan? It can take a long time and may affect performance while gaming.",
            "Confirm full malware scan",
            [Windows.Forms.MessageBoxButtons]::YesNo,
            [Windows.Forms.MessageBoxIcon]::Warning)
        if ($answer -ne [Windows.Forms.DialogResult]::Yes) { Add-Log "Full scan cancelled."; return }
        Start-CrimsonDefenderFullScan -Confirm:$false
        Add-Log "Microsoft Defender full scan requested. Check Windows Security for progress and results."
    } catch { Add-Log ("Defender scan request failed: "+$_.Exception.Message) }
}
function Show-DefenderHistory {
    try {
        $items = @(Get-CrimsonThreatDetections)
        if ($items.Count -eq 0) {
            $message = "No detections are recorded in Microsoft Defender history. This is not a guarantee that every file is clean."
        } else {
            $message = ($items | Select-Object -First 30 | ForEach-Object {
                "{0}  |  {1}  |  {2}" -f $_.ThreatName,$_.InitialDetectionTime,$_.ActionSuccess
            }) -join $NL
        }
        [Windows.Forms.MessageBox]::Show($message,"Microsoft Defender threat history") | Out-Null
    } catch {
        [Windows.Forms.MessageBox]::Show($_.Exception.Message,"Threat history unavailable") | Out-Null
    }
}
function Open-DisplaySettings {
    try { Open-CrimsonDisplaySettings; Add-Log "Opened Windows display settings." }
    catch { Add-Log ("Could not open display settings: "+$_.Exception.Message) }
}
function Open-PowerSettings {
    try { Open-CrimsonPowerSettings; Add-Log "Opened Windows power settings. Use supported power modes; do not disable core parking or idle states." }
    catch { Add-Log ("Could not open power settings: "+$_.Exception.Message) }
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
$right=New-Object Windows.Forms.Panel; $right.Location=New-Object Drawing.Point(320,102); $right.Size=New-Object Drawing.Size(640,580)
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
$logTitle=New-Object Windows.Forms.Label; $logTitle.Text="ACTIVITY LOG"; $logTitle.Location=New-Object Drawing.Point(18,272)
$logTitle.AutoSize=$true; $logTitle.ForeColor=$C.Muted; $logTitle.Font=New-Object Drawing.Font("Segoe UI",10,[Drawing.FontStyle]::Bold); $right.Controls.Add($logTitle)
$script:Log=New-Object Windows.Forms.RichTextBox; $script:Log.Location=New-Object Drawing.Point(18,302); $script:Log.Size=New-Object Drawing.Size(600,202)
$script:Log.Anchor="Top,Bottom,Left,Right"; $script:Log.ReadOnly=$true; $script:Log.BorderStyle="None"; $script:Log.BackColor=$C.Panel2
$script:Log.ForeColor=$C.Text; $script:Log.Font=New-Object Drawing.Font("Consolas",8.5); $right.Controls.Add($script:Log)
$software=New-Button "Installed software" 18 518 142 32
$resources=New-Button "Resource use" 170 518 142 32
$defender=New-Button "Defender full scan" 322 518 142 32 $true
$display=New-Button "Display settings" 474 518 142 32
$power=New-Button "Power settings" 18 558 142 28
$history=New-Button "Threat history" 170 558 142 28
$foot=New-Object Windows.Forms.Label; $foot.Text="No false virus labels  •  No forced awake cores  •  No thermal bypass"
$foot.Location=New-Object Drawing.Point(322,563); $foot.AutoSize=$true; $foot.ForeColor=$C.Muted
$foot.Font=New-Object Drawing.Font("Segoe UI",8)
$right.Controls.AddRange(@($software,$resources,$defender,$display,$power,$history,$foot))
$software.Add_Click({Show-InstalledSoftware}); $resources.Add_Click({Show-ResourceUse})
$defender.Add_Click({Invoke-DefenderFullScan}); $display.Add_Click({Open-DisplaySettings}); $power.Add_Click({Open-PowerSettings}); $history.Add_Click({Show-DefenderHistory})

$refresh.Add_Click({Refresh-Apps}); $apply.Add_Click({Apply-Profile}); $restore.Add_Click({Restore-Profile})
$preview.Add_Click({Preview-Cleanup}); $clean.Add_Click({Clean-Temp}); $scan.Add_Click({try{Read-System}catch{Add-Log ("System scan failed: "+$_.Exception.Message)}})
$form.Add_Shown({Refresh-Apps; $scan.PerformClick(); Add-Log "Ready. Select a visible game process to apply a reversible priority request."})
[void]$form.ShowDialog()
