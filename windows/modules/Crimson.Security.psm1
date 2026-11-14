Set-StrictMode -Version Latest

function Get-CrimsonDefenderStatus {
    [CmdletBinding()]
    param()
    if (-not (Get-Command Get-MpComputerStatus -ErrorAction SilentlyContinue)) {
        return [pscustomobject]@{ Available=$false; Enabled=$false; RealTimeProtection=$false; Status='Microsoft Defender cmdlets unavailable' }
    }
    try {
        $status = Get-MpComputerStatus -ErrorAction Stop
        [pscustomobject]@{
            Available = $true
            Enabled = [bool]$status.AntivirusEnabled
            RealTimeProtection = [bool]$status.RealTimeProtectionEnabled
            Status = if ($status.AntivirusEnabled -and $status.RealTimeProtectionEnabled) { 'Defender active' } else { 'Protection not fully active' }
        }
    } catch {
        [pscustomobject]@{ Available=$false; Enabled=$false; RealTimeProtection=$false; Status=('Defender status unavailable: ' + $_.Exception.Message) }
    }
}

function Get-CrimsonThreatDetections {
    [CmdletBinding()]
    param()
    if (-not (Get-Command Get-MpThreatDetection -ErrorAction SilentlyContinue)) {
        throw 'Microsoft Defender threat history is unavailable on this device.'
    }
    Get-MpThreatDetection -ErrorAction Stop
}

function Start-CrimsonDefenderFullScan {
    [CmdletBinding(SupportsShouldProcess)]
    param()
    if (-not (Get-Command Start-MpScan -ErrorAction SilentlyContinue)) {
        throw 'Microsoft Defender scan cmdlet is unavailable.'
    }
    if ($PSCmdlet.ShouldProcess('This Windows device', 'Start Microsoft Defender full scan')) {
        Start-MpScan -ScanType FullScan -ErrorAction Stop
    }
}

Export-ModuleMember -Function Get-CrimsonDefenderStatus, Get-CrimsonThreatDetections, Start-CrimsonDefenderFullScan
