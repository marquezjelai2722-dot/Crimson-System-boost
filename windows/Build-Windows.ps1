$ErrorActionPreference = "Stop"
$windowsRoot = $PSScriptRoot
$repoRoot = Split-Path -Parent $windowsRoot
$modulesRoot = Join-Path $windowsRoot "modules"
$dist = Join-Path $repoRoot "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

$moduleFiles = @(
    "Crimson.Inventory.psm1",
    "Crimson.Security.psm1",
    "Crimson.Maintenance.psm1",
    "Crimson.Performance.psm1"
)
$moduleSources = foreach ($name in $moduleFiles) {
    $path = Join-Path $modulesRoot $name
    if (-not (Test-Path -LiteralPath $path)) { throw "Required module missing: $path" }
    $source = Get-Content -LiteralPath $path -Raw
    # Functions are compiled into the portable EXE; module-only directives are omitted.
    $source = $source -replace '(?m)^Set-StrictMode -Version Latest\s*', ''
    $source = $source -replace '(?m)^Export-ModuleMember.*(?:\r?\n|$)', ''
    $source
}
$guiPath = Join-Path $windowsRoot "Crimson-Windows-GUI.ps1"
$guiSource = Get-Content -LiteralPath $guiPath -Raw
$bundlePath = Join-Path $dist "Crimson-Windows-Bundle.ps1"
$bundle = ($moduleSources -join [Environment]::NewLine) + [Environment]::NewLine + $guiSource
Set-Content -LiteralPath $bundlePath -Value $bundle -Encoding UTF8

if (-not (Get-Command Invoke-PS2EXE -ErrorAction SilentlyContinue)) {
    throw "PS2EXE is required. Install it with: Install-Module ps2exe -Scope CurrentUser"
}
$outputPath = Join-Path $dist "Crimson-System-Boost-Windows.exe"
Invoke-PS2EXE -inputFile $bundlePath -outputFile $outputPath -noConsole -STA -title "Crimson System Boost" -description "Crimson System Boost Windows desktop utility" -company "Crimson System Boost"
if (-not (Test-Path -LiteralPath $outputPath)) { throw "EXE packaging did not produce an output file." }
$exe = Get-Item -LiteralPath $outputPath
if ($exe.Length -le 0) { throw "EXE output is empty." }
$stream = [System.IO.File]::OpenRead($exe.FullName)
try {
    $signature = New-Object byte[] 2
    [void]$stream.Read($signature, 0, 2)
} finally { $stream.Dispose() }
if ([System.Text.Encoding]::ASCII.GetString($signature) -ne "MZ") {
    throw "EXE does not have a valid Windows PE signature."
}
Write-Host ("Built portable EXE: {0:N0} bytes" -f $exe.Length)
