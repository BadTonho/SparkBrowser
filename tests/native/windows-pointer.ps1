param([Parameter(Mandatory=$true)][string]$Executable, [Parameter(Mandatory=$true)][string]$ActionsFile)
$ErrorActionPreference = 'Stop'
$buildRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target')) + [IO.Path]::DirectorySeparatorChar
$runsRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-runs')) + [IO.Path]::DirectorySeparatorChar
$testExecutable = [IO.Path]::GetFullPath($Executable)
$inputFile = [IO.Path]::GetFullPath($ActionsFile)
if (-not $testExecutable.StartsWith($buildRoot, [StringComparison]::OrdinalIgnoreCase) -or
    -not $inputFile.StartsWith($runsRoot, [StringComparison]::OrdinalIgnoreCase) -or
    [IO.Path]::GetFileName([IO.Path]::GetDirectoryName($inputFile)) -ne 'artifacts') { throw 'Only isolated test input is allowed' }
$actions = Get-Content -LiteralPath $inputFile -Raw | ConvertFrom-Json
if (@($actions).Count -gt 64) { throw 'Too many pointer actions' }
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeTestPointer {
    [StructLayout(LayoutKind.Sequential)] public struct Point { public int x,y; }
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int left,top,right,bottom; }
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window,out Rect rect);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window,ref Point point);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(Point point);
    [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr window,uint flags);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
    [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point point);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
}
'@
$target = Get-Process | Where-Object { $_.Path -and [string]::Equals($_.Path, $testExecutable, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
foreach ($action in $actions) {
    if ($action.type -eq 'pointerUp' -and $action.button -eq 0) {
        [NativeTestPointer]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
        continue
    }
    if (-not $target -or $target.MainWindowHandle -eq 0 -or [NativeTestPointer]::GetForegroundWindow() -ne $target.MainWindowHandle) { throw 'Isolated main window must have foreground focus for pointer input' }
    switch ($action.type) {
        'pointerMove' {
            $area = New-Object NativeTestPointer+Rect
            $origin = New-Object NativeTestPointer+Point
            [void][NativeTestPointer]::GetClientRect($target.MainWindowHandle,[ref]$area)
            [void][NativeTestPointer]::ClientToScreen($target.MainWindowHandle,[ref]$origin)
            $scale = [NativeTestPointer]::GetDpiForWindow($target.MainWindowHandle) / 96.0
            $x = [int][Math]::Round([double]$action.x * $scale); $y = [int][Math]::Round([double]$action.y * $scale)
            if ($x -lt 0 -or $y -lt 0 -or $x -ge $area.right -or $y -ge $area.bottom) { throw 'Pointer destination must stay inside the isolated client' }
            $start = New-Object NativeTestPointer+Point
            [void][NativeTestPointer]::GetCursorPos([ref]$start)
            $steps = [Math]::Max(1,[Math]::Ceiling([double]$action.duration / 16))
            for ($i=1; $i -le $steps; $i++) {
                if ([NativeTestPointer]::GetForegroundWindow() -ne $target.MainWindowHandle) { throw 'Isolated test window lost focus during pointer movement' }
                [void][NativeTestPointer]::SetCursorPos([int]($start.x + ($origin.x + $x - $start.x)*$i/$steps), [int]($start.y + ($origin.y + $y - $start.y)*$i/$steps))
                if ($steps -gt 1) { Start-Sleep -Milliseconds 16 }
            }
        }
        'pointerDown' {
            if ($action.button -ne 0) { throw 'Only the left test button is supported' }
            $point = New-Object NativeTestPointer+Point
            [void][NativeTestPointer]::GetCursorPos([ref]$point)
            if ([NativeTestPointer]::GetAncestor([NativeTestPointer]::WindowFromPoint($point),2) -ne $target.MainWindowHandle) { throw 'Pointer-down must hit the isolated main window' }
            [NativeTestPointer]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
        }
        'pause' { if ($action.duration -lt 0 -or $action.duration -gt 2000) { throw 'Invalid test pause' }; Start-Sleep -Milliseconds $action.duration }
        default { throw 'Unsupported pointer action' }
    }
}
