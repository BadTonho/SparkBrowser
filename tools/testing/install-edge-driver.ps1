$ErrorActionPreference = 'Stop'

# Choose the installed WebView2 runtime, which can differ from the Edge browser version.
$runtimeDirectory = Join-Path ${env:ProgramFiles(x86)} 'Microsoft\EdgeWebView\Application'
$runtimeVersion = Get-ChildItem -LiteralPath $runtimeDirectory -Directory |
    Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
    Sort-Object { [version]$_.Name } -Descending |
    Select-Object -First 1 -ExpandProperty Name
if (-not $runtimeVersion) { throw 'Microsoft Edge WebView2 Runtime was not found.' }

$destination = Join-Path (Resolve-Path '.').Path '.cache/native-tools/edge'
New-Item -ItemType Directory -Path $destination -Force | Out-Null
$archive = Join-Path $destination 'edgedriver.zip'
Invoke-WebRequest -Uri "https://msedgedriver.microsoft.com/$runtimeVersion/edgedriver_win64.zip" -OutFile $archive
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
$driver = Join-Path $destination 'msedgedriver.exe'
if (-not (Test-Path -LiteralPath $driver)) { throw 'The driver archive did not contain msedgedriver.exe.' }
Write-Output "WebView2 $runtimeVersion driver installed at $driver"
