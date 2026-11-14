# Crimson inventory and native security bridge.
# Read-only inventory. Microsoft Defender owns malware detection; this module
# never treats app names, publisher names, or install locations as malware proof.

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
        Get-ItemProperty -Path $root -ErrorAction SilentlyContinue |
            Where-Object { $_.DisplayName -and -not $_.SystemComponent } |
            Select-Object @{N='Name';E={$_.DisplayName}},
                @{N='Version';E={$_.DisplayVersion}},
                Publisher, InstallDate,
                @{N='InstallLocation';E={$_.InstallLocation}},
                @{N='RegistrySource';E={$root}}
    }
    $items | Sort-Object Name, Version -Unique
}

function Get-CrimsonDefenderStatus {
    [CmdletBinding()]
    param()

    if (-not (Get-Command Get-MpComputerStatus -ErrorAction SilentlyContinue)) {
        return [pscustomobject]@{
            Available = $false
            RealTimeProtectionEnabled = $null
            AntivirusEnabled = $null
            AntivirusSignatureAge = $null
            Status = 'Microsoft Defender status API unavailable'
        }
    }

    try {
        $state = Get-MpComputerStatus -ErrorAction Stop
        [pscustomobject]@{
            Available = $true
            RealTimeProtectionEnabled = [bool]$state.RealTimeProtectionEnabled
            AntivirusEnabled = [bool]$state.AntivirusEnabled
            AntivirusSignatureAge = $state.AntivirusSignatureAge
            Status = if ($state.RealTimeProtectionEnabled -and $state.AntivirusEnabled) {
                'Microsoft Defender reports protection enabled'
            } else {
                'Review Microsoft Defender protection status'
            }
        }
    } catch {
        [pscustomobject]@{
            Available = $true
            RealTimeProtectionEnabled = $null
            AntivirusEnabled = $null
            AntivirusSignatureAge = $null
            Status = "Status query failed: $($_.Exception.Message)"
        }
    }
}

function Start-CrimsonDefenderQuickScan {
    [CmdletBinding(SupportsShouldProcess = $true)]
    param()

    if (-not (Get-Command Start-MpScan -ErrorAction SilentlyContinue)) {
        throw 'Microsoft Defender scan command is unavailable. Use the security app supplied by your antivirus provider.'
    }
    if ($PSCmdlet.ShouldProcess('This device', 'Request Microsoft Defender quick scan')) {
        Start-MpScan -ScanType QuickScan -ErrorAction Stop
        return 'Quick scan request sent to Microsoft Defender.'
    }
}

Export-ModuleMember -Function Get-CrimsonInstalledSoftware, Get-CrimsonDefenderStatus, Start-CrimsonDefenderQuickScan
