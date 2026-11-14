Set-StrictMode -Version Latest

function Get-CrimsonTemporaryFiles {
    [CmdletBinding()]
    param([ValidateRange(1,365)][int]$OlderThanDays = 7)
    $cutoff = (Get-Date).AddDays(-$OlderThanDays)
    $roots = @($env:TEMP, (Join-Path $env:WINDIR 'Temp')) |
        Where-Object { $_ -and (Test-Path -LiteralPath $_) } | Select-Object -Unique
    foreach ($root in $roots) {
        try {
            Get-ChildItem -LiteralPath $root -Force -File -ErrorAction Stop |
                Where-Object { $_.LastWriteTime -lt $cutoff }
        } catch {
            Write-Verbose ("Cleanup source skipped: " + $root)
        }
    }
}

function Remove-CrimsonTemporaryFiles {
    [CmdletBinding(SupportsShouldProcess)]
    param([Parameter(Mandatory)][System.IO.FileInfo[]]$Files)
    $removed = 0
    $skipped = 0
    foreach ($file in $Files) {
        try {
            if ($PSCmdlet.ShouldProcess($file.FullName, 'Delete old temporary file')) {
                Remove-Item -LiteralPath $file.FullName -Force -ErrorAction Stop
                $removed++
            }
        } catch {
            $skipped++
            Write-Verbose ("Skipped locked or inaccessible file: " + $file.Name)
        }
    }
    [pscustomobject]@{ Removed=$removed; Skipped=$skipped }
}

Export-ModuleMember -Function Get-CrimsonTemporaryFiles, Remove-CrimsonTemporaryFiles
