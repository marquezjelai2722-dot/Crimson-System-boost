Set-StrictMode -Version Latest

function Get-CrimsonInstalledSoftware {
    [CmdletBinding()]
    param()
    $roots = @(
        'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
        'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*'
    )
    $items = foreach ($root in $roots) {
        try {
            Get-ItemProperty -Path $root -ErrorAction Stop | ForEach-Object {
                if (-not [string]::IsNullOrWhiteSpace([string]$_.DisplayName)) {
                    $size = if ($_.EstimatedSize) { [long]$_.EstimatedSize * 1KB } else { $null }
                    [pscustomobject]@{
                        Name = [string]$_.DisplayName
                        Version = [string]$_.DisplayVersion
                        Publisher = [string]$_.Publisher
                        EstimatedBytes = $size
                        LargeFootprint = ($null -ne $size -and $size -ge 1GB)
                    }
                }
            }
        } catch {
            Write-Verbose ("Inventory source skipped: " + $root)
        }
    }
    $items | Sort-Object Name -Unique
}

function Get-CrimsonResourceHeavyProcesses {
    [CmdletBinding()]
    param([ValidateRange(1,50)][int]$Top = 15)
    Get-Process -ErrorAction SilentlyContinue |
        Sort-Object WorkingSet64 -Descending |
        Select-Object -First $Top ProcessName, Id,
            @{Name='MemoryMB';Expression={[math]::Round($_.WorkingSet64 / 1MB, 1)}},
            @{Name='CpuSeconds';Expression={if ($_.CPU) {[math]::Round($_.CPU,1)} else {0}}}
}

Export-ModuleMember -Function Get-CrimsonInstalledSoftware, Get-CrimsonResourceHeavyProcesses
