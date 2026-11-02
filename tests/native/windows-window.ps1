[CmdletBinding(DefaultParameterSetName='Resize')]
param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true,ParameterSetName='Resize')][ValidateRange(680,3840)][int]$Width,
    [Parameter(Mandatory=$true,ParameterSetName='Resize')][ValidateRange(480,2160)][int]$Height,
    [Parameter(Mandatory=$true,ParameterSetName='Focus')][switch]$Focus
)
$ErrorActionPreference = 'Stop'
$buildRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target')) + [IO.Path]::DirectorySeparatorChar
$testExecutable = [IO.Path]::GetFullPath($Executable)
if (-not $testExecutable.StartsWith($buildRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Only the isolated native test application can be controlled' }
$target = Get-Process | Where-Object { $_.Path -and [string]::Equals($_.Path, $testExecutable, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
if (-not $target -or $target.MainWindowHandle -eq 0) { throw 'Isolated test window was not found' }
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeTestWindow {
    [StructLayout(LayoutKind.Sequential)] public struct Rectangle { public int left, top, right, bottom; }
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out Rectangle rectangle);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window, out Rectangle rectangle);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window, int command);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint from, uint to, bool attach);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
}
'@
if ($Focus) {
    [uint32]$foregroundProcess = 0
    $foregroundThread = [NativeTestWindow]::GetWindowThreadProcessId([NativeTestWindow]::GetForegroundWindow(), [ref]$foregroundProcess)
    $inputThread = [NativeTestWindow]::GetCurrentThreadId()
    $attached = [NativeTestWindow]::AttachThreadInput($inputThread, $foregroundThread, $true)
    try {
        [void][NativeTestWindow]::ShowWindow($target.MainWindowHandle,9)
        $activationShell = New-Object -ComObject WScript.Shell
        [void]$activationShell.AppActivate($target.Id)
        [void][NativeTestWindow]::SetForegroundWindow($target.MainWindowHandle)
        Start-Sleep -Milliseconds 100
    }
    finally { if ($attached) { [void][NativeTestWindow]::AttachThreadInput($inputThread, $foregroundThread, $false) } }
    [void][NativeTestWindow]::GetWindowThreadProcessId([NativeTestWindow]::GetForegroundWindow(), [ref]$foregroundProcess)
    if ($foregroundProcess -ne $target.Id) { throw 'The isolated test application did not receive foreground focus' }
    return
}
$rectangle = New-Object NativeTestWindow+Rectangle
$client = New-Object NativeTestWindow+Rectangle
if (-not [NativeTestWindow]::GetWindowRect($target.MainWindowHandle, [ref]$rectangle)) { throw 'Could not read the isolated test window bounds' }
if (-not [NativeTestWindow]::GetClientRect($target.MainWindowHandle, [ref]$client)) { throw 'Could not read the isolated test client bounds' }
$outerWidth = $Width + ($rectangle.right - $rectangle.left) - ($client.right - $client.left)
$outerHeight = $Height + ($rectangle.bottom - $rectangle.top) - ($client.bottom - $client.top)
# Preserve position, focus and stacking order; resize only the verified process.
if (-not [NativeTestWindow]::SetWindowPos($target.MainWindowHandle, [IntPtr]::Zero, $rectangle.left, $rectangle.top, $outerWidth, $outerHeight, 0x0014)) { throw 'Could not resize the isolated test window' }
