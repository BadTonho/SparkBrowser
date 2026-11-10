param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$RunDirectory
)
$ErrorActionPreference = 'Stop'
$buildRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target')) + [IO.Path]::DirectorySeparatorChar
$runsRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-runs')) + [IO.Path]::DirectorySeparatorChar
$testExecutable = [IO.Path]::GetFullPath($Executable)
$testDirectory = [IO.Path]::GetFullPath($RunDirectory)
if (-not $testExecutable.StartsWith($buildRoot, [StringComparison]::OrdinalIgnoreCase) -or
    -not $testDirectory.StartsWith($runsRoot, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Only isolated native-test processes can be inspected'
}
$profilePattern = [regex]::Escape((Join-Path $testDirectory 'profile')) + '[\\/]webview-(accounts|workspaces|ephemeral)[\\/]'
$deadline = [DateTime]::UtcNow.AddSeconds(25)
do {
    $application = @(Get-Process | Where-Object { $_.Path -and [string]::Equals($_.Path, $testExecutable, [StringComparison]::OrdinalIgnoreCase) })
    $profiles = @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" | Where-Object { $_.CommandLine -match $profilePattern })
    if ($application.Count -eq 0 -and $profiles.Count -eq 0) { exit 0 }
    Start-Sleep -Milliseconds 200
} while ([DateTime]::UtcNow -lt $deadline)
throw 'Isolated application/profile processes did not exit cleanly before restart'
