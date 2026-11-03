param([Parameter(Mandatory=$true)][string]$Executable)
$ErrorActionPreference='Stop'
$buildRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target'))+[IO.Path]::DirectorySeparatorChar
$testPath=[IO.Path]::GetFullPath($Executable)
if(-not $testPath.StartsWith($buildRoot,[StringComparison]::OrdinalIgnoreCase)){throw 'Only the isolated test app is allowed'}
$target=Get-Process | Where-Object {$_.Path -and [string]::Equals($_.Path,$testPath,[StringComparison]::OrdinalIgnoreCase)} | Select-Object -First 1
if(-not $target){throw 'Isolated test app not found'}
Add-Type -TypeDefinition @'
using System;using System.Text;using System.Runtime.InteropServices;
public static class DragTestWindows {
 public delegate bool Callback(IntPtr hwnd,IntPtr data);
 [StructLayout(LayoutKind.Sequential)] public struct Rect {public int left,top,right,bottom;}
 [StructLayout(LayoutKind.Sequential)] public struct Point {public int x,y;}
 [DllImport("user32.dll")] public static extern bool EnumWindows(Callback callback,IntPtr data);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd,out uint process);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hwnd,out Rect rect);
 [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr hwnd,ref Point point);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
 [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr hwnd,uint command);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr hwnd,StringBuilder text,int count);
}
'@
$windows=[Collections.Generic.List[object]]::new()
$mainHandle=$target.MainWindowHandle
$callback=[DragTestWindows+Callback]{param($handle,$unused)
 [uint32]$ownerProcess=0;[void][DragTestWindows]::GetWindowThreadProcessId($handle,[ref]$ownerProcess)
 if($ownerProcess -eq $target.Id){
  $rect=New-Object DragTestWindows+Rect;[void][DragTestWindows]::GetWindowRect($handle,[ref]$rect)
  $origin=New-Object DragTestWindows+Point;[void][DragTestWindows]::ClientToScreen($handle,[ref]$origin)
  $text=[Text.StringBuilder]::new(256);[void][DragTestWindows]::GetWindowText($handle,$text,256)
  $windows.Add(@{main=$handle -eq $mainHandle;owner=[DragTestWindows]::GetWindow($handle,4) -eq $mainHandle;visible=[DragTestWindows]::IsWindowVisible($handle);foreground=[DragTestWindows]::GetForegroundWindow() -eq $handle;left=$rect.left;top=$rect.top;clientLeft=$origin.x;clientTop=$origin.y;width=$rect.right-$rect.left;height=$rect.bottom-$rect.top;scale=[DragTestWindows]::GetDpiForWindow($handle)/96.0;title=$text.ToString()})
 }
 return $true
}
[void][DragTestWindows]::EnumWindows($callback,[IntPtr]::Zero)
ConvertTo-Json -InputObject @($windows.ToArray()) -Compress
