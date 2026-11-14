Set-StrictMode -Version Latest

function Open-CrimsonDisplaySettings {
    [CmdletBinding()]
    param()
    Start-Process 'ms-settings:display'
}

function Get-CrimsonProcessorTopology {
    [CmdletBinding()]
    param()
    Get-CimInstance Win32_Processor -ErrorAction Stop |
        Select-Object Name, NumberOfCores, NumberOfLogicalProcessors
}

function Get-CrimsonPowerPlan {
    [CmdletBinding()]
    param()
    powercfg.exe /getactivescheme 2>&1
}

function Open-CrimsonPowerSettings {
    [CmdletBinding()]
    param()
    Start-Process 'ms-settings:powersleep'
}

Export-ModuleMember -Function Open-CrimsonDisplaySettings, Get-CrimsonProcessorTopology, Get-CrimsonPowerPlan, Open-CrimsonPowerSettings
