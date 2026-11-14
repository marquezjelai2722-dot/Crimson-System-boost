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
                # Registry uninstall entries are inconsistent. Read optional fields
                # through PSObject.Properties so one incomplete key cannot stop the scan.
                try {
                    $entry = $_
                    $nameProperty = $entry.PSObject.Properties['DisplayName']
                    $name = if ($nameProperty) { [string]$nameProperty.Value } else { '' }
                    if ([string]::IsNullOrWhiteSpace($name)) { return }

                    $versionProperty = $entry.PSObject.Properties['DisplayVersion']
                    $publisherProperty = $entry.PSObject.Properties['Publisher']
                    $sizeProperty = $entry.PSObject.Properties['EstimatedSize']
                    $size = if ($sizeProperty -and $sizeProperty.Value) {
                        [long]$sizeProperty.Value * 1KB
                    } else { $null }

                    [pscustomobject]@{
                        Name = $name
                        Version = if ($versionProperty) { [string]$versionProperty.Value } else { '' }
                        Publisher = if ($publisherProperty) { [string]$publisherProperty.Value } else { '' }
                        EstimatedBytes = $size
                        LargeFootprint = ($null -ne $size -and $size -ge 1GB)
                    }
                } catch {
                    Write-Verbose ("Skipped malformed uninstall entry in " + $root)
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
