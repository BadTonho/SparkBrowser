param([Parameter(Mandatory=$true)][string]$Executable)
$ErrorActionPreference = 'Stop'
$expectedRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target')) + [IO.Path]::DirectorySeparatorChar
$expectedExecutable = [IO.Path]::GetFullPath($Executable)
if (-not $expectedExecutable.StartsWith($expectedRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Only the isolated test executable may be sampled' }
$roots = @(Get-Process -Name spark-browser -ErrorAction SilentlyContinue | Where-Object { $_.Path -and [string]::Equals($_.Path, $expectedExecutable, [StringComparison]::OrdinalIgnoreCase) })
if ($roots.Count -ne 1) { throw "Expected one isolated test process, found $($roots.Count)" }
$processIndex = @(Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name, CommandLine)
$owned = [Collections.Generic.HashSet[uint32]]::new()
[void]$owned.Add([uint32]$roots[0].Id)
do {
    $added = $false
    foreach ($item in $processIndex) {
        if ($owned.Contains([uint32]$item.ParentProcessId) -and $owned.Add([uint32]$item.ProcessId)) { $added = $true }
    }
} while ($added)
$processes = @(foreach ($ownedId in $owned) {
    $item = Get-Process -Id $ownedId -ErrorAction SilentlyContinue
    if ($item) {
        $metadata = $processIndex | Where-Object { $_.ProcessId -eq $ownedId } | Select-Object -First 1
        $role = if ($ownedId -eq $roots[0].Id) { 'application' } elseif ($metadata.CommandLine -match '--type=([a-zA-Z-]+)') { $Matches[1] } else { 'browser' }
        # Record the parsed engine role only, never full command lines or profile paths.
        [pscustomobject][ordered]@{ id=$item.Id; name=$item.ProcessName; role=$role; workingSetBytes=$item.WorkingSet64; privateBytes=$item.PrivateMemorySize64; cpuSeconds=$item.CPU; handles=$item.HandleCount }
    }
})
[ordered]@{
    sampledUtc=[DateTime]::UtcNow.ToString('o')
    rootPid=$roots[0].Id
    processCount=$processes.Count
    workingSetBytes=($processes | Measure-Object -Property workingSetBytes -Sum).Sum
    privateBytes=($processes | Measure-Object -Property privateBytes -Sum).Sum
    processes=$processes
} | ConvertTo-Json -Depth 5 -Compress
