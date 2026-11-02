param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [ValidateSet('Inspect','Restore','Drag','Resize')][string]$Action='Inspect'
)
$ErrorActionPreference='Stop'
$buildRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target'))+[IO.Path]::DirectorySeparatorChar
$testExecutable=[IO.Path]::GetFullPath($Executable)
if(-not $testExecutable.StartsWith($buildRoot,[StringComparison]::OrdinalIgnoreCase)){throw 'Only the isolated test application is allowed'}
$target=Get-Process | Where-Object {$_.Path -and [string]::Equals($_.Path,$testExecutable,[StringComparison]::OrdinalIgnoreCase)} | Select-Object -First 1
if(-not $target){throw 'Isolated process not found'}
Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class FloatingTestWindow {
    public delegate bool Callback(IntPtr window,IntPtr data);
    [StructLayout(LayoutKind.Sequential)] public struct Rect {public int left,top,right,bottom;}
    [StructLayout(LayoutKind.Sequential)] public struct Point {public int x,y;}
    [DllImport("user32.dll")] public static extern bool EnumWindows(Callback callback,IntPtr data);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window,out uint process);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr window,StringBuilder text,int count);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window,out Rect bounds);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr window,ref Point point);
    [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr window,int index);
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr window);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window,int command);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint from,uint to,bool attach);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point point);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
    [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(Point point);
    [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr window,uint flags);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
}
'@
$script:floatingHandle=[IntPtr]::Zero
$callback=[FloatingTestWindow+Callback]{param($window,$data)
    [uint32]$windowProcess=0
    [void][FloatingTestWindow]::GetWindowThreadProcessId($window,[ref]$windowProcess)
    if($windowProcess -eq $target.Id){
        $title=New-Object Text.StringBuilder 256
        [void][FloatingTestWindow]::GetWindowText($window,$title,256)
        if($title.ToString() -eq 'Floating video'){$script:floatingHandle=$window}
    }
    return $true
}
[void][FloatingTestWindow]::EnumWindows($callback,[IntPtr]::Zero)
if($script:floatingHandle -eq [IntPtr]::Zero){throw 'Isolated floating window not found'}
if($Action -eq 'Restore'){[void][FloatingTestWindow]::ShowWindow($script:floatingHandle,9)}
$bounds=New-Object FloatingTestWindow+Rect
if(-not [FloatingTestWindow]::GetWindowRect($script:floatingHandle,[ref]$bounds)){throw 'Could not read floating bounds'}
if($Action -in @('Drag','Resize')){
    [uint32]$foregroundProcess=0
    $foregroundThread=[FloatingTestWindow]::GetWindowThreadProcessId([FloatingTestWindow]::GetForegroundWindow(),[ref]$foregroundProcess)
    $inputThread=[FloatingTestWindow]::GetCurrentThreadId()
    $attached=[FloatingTestWindow]::AttachThreadInput($inputThread,$foregroundThread,$true)
    try{
        [void][FloatingTestWindow]::ShowWindow($script:floatingHandle,9)
        $activationShell=New-Object -ComObject WScript.Shell
        [void]$activationShell.AppActivate($target.Id)
        [void][FloatingTestWindow]::SetForegroundWindow($script:floatingHandle)
        Start-Sleep -Milliseconds 100
    }finally{if($attached){[void][FloatingTestWindow]::AttachThreadInput($inputThread,$foregroundThread,$false)}}
    if([FloatingTestWindow]::GetForegroundWindow() -ne $script:floatingHandle){throw 'Isolated floating window must have foreground focus before pointer input'}
    $originalCursor=New-Object FloatingTestWindow+Point
    [void][FloatingTestWindow]::GetCursorPos([ref]$originalCursor)
    $scale=[FloatingTestWindow]::GetDpiForWindow($script:floatingHandle)/96.0
    $point=New-Object FloatingTestWindow+Point
    $point.x=$bounds.left+[int](130*$scale);$point.y=$bounds.top+[int](18*$scale)
    if($Action -eq 'Resize'){$point.x=$bounds.right-4;$point.y=$bounds.bottom-4}
    [void][FloatingTestWindow]::SetCursorPos($point.x,$point.y)
    Start-Sleep -Milliseconds 80
    if([FloatingTestWindow]::GetAncestor([FloatingTestWindow]::WindowFromPoint($point),2) -ne $script:floatingHandle){throw 'Drag point must belong to the isolated floating window'}
    [FloatingTestWindow]::mouse_event(0x0002,0,0,0,[UIntPtr]::Zero)
    try{
        Start-Sleep -Milliseconds 80
        for($step=1;$step -le 10;$step++){
            if([FloatingTestWindow]::GetForegroundWindow() -ne $script:floatingHandle){throw 'Isolated floating window lost foreground focus during drag'}
            [void][FloatingTestWindow]::SetCursorPos($point.x+$step*4,$point.y+$step*2)
            Start-Sleep -Milliseconds 20
        }
    }finally{
        [FloatingTestWindow]::mouse_event(0x0004,0,0,0,[UIntPtr]::Zero)
        [void][FloatingTestWindow]::SetCursorPos($originalCursor.x,$originalCursor.y)
    }
    Start-Sleep -Milliseconds 80
    [void][FloatingTestWindow]::GetWindowRect($script:floatingHandle,[ref]$bounds)
}
$clientOrigin=New-Object FloatingTestWindow+Point
if(-not [FloatingTestWindow]::ClientToScreen($script:floatingHandle,[ref]$clientOrigin)){throw 'Could not read floating client origin'}
@{minimized=[FloatingTestWindow]::IsIconic($script:floatingHandle);topInset=$clientOrigin.y-$bounds.top;left=$bounds.left;top=$bounds.top;width=$bounds.right-$bounds.left;height=$bounds.bottom-$bounds.top} | ConvertTo-Json -Compress
