param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][ValidateRange(680,3840)][int]$Width,
    [Parameter(Mandatory=$true)][ValidateRange(480,2160)][int]$Height
)
$ErrorActionPreference = 'Stop'
$buildRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target')) + [IO.Path]::DirectorySeparatorChar
$testExecutable = [IO.Path]::GetFullPath($Executable)
if (-not $testExecutable.StartsWith($buildRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Only the isolated native test application can be resized' }
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
}
'@
$rectangle = New-Object NativeTestWindow+Rectangle
$client = New-Object NativeTestWindow+Rectangle
if (-not [NativeTestWindow]::GetWindowRect($target.MainWindowHandle, [ref]$rectangle)) { throw 'Could not read the isolated test window bounds' }
if (-not [NativeTestWindow]::GetClientRect($target.MainWindowHandle, [ref]$client)) { throw 'Could not read the isolated test client bounds' }
$outerWidth = $Width + ($rectangle.right - $rectangle.left) - ($client.right - $client.left)
$outerHeight = $Height + ($rectangle.bottom - $rectangle.top) - ($client.bottom - $client.top)
# Preserve position, focus and stacking order; resize only the verified process.
if (-not [NativeTestWindow]::SetWindowPos($target.MainWindowHandle, [IntPtr]::Zero, $rectangle.left, $rectangle.top, $outerWidth, $outerHeight, 0x0014)) { throw 'Could not resize the isolated test window' }
