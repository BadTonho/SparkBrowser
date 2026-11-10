[CmdletBinding(DefaultParameterSetName='Resize')]
param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true,ParameterSetName='Resize')][ValidateRange(680,3840)][int]$Width,
    [Parameter(Mandatory=$true,ParameterSetName='Resize')][ValidateRange(480,2160)][int]$Height,
    [Parameter(Mandatory=$true,ParameterSetName='Focus')][switch]$Focus,
    [Parameter(Mandatory=$true,ParameterSetName='Inspect')][switch]$Inspect
)
$ErrorActionPreference = 'Stop'
$buildRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-target')) + [IO.Path]::DirectorySeparatorChar
$testExecutable = [IO.Path]::GetFullPath($Executable)
if (-not $testExecutable.StartsWith($buildRoot, [StringComparison]::OrdinalIgnoreCase)) { throw 'Only the isolated native test application can be controlled' }
$target = Get-Process | Where-Object { $_.Path -and [string]::Equals($_.Path, $testExecutable, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
if (-not $target) { throw 'Isolated test process was not found' }
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class NativeTestWindow {
    public delegate bool Callback(IntPtr window, IntPtr data);
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
    [DllImport("user32.dll")] public static extern bool EnumWindows(Callback callback,IntPtr data);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr window);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr window,StringBuilder title,int count);
    [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr window,StringBuilder name,int count);
}
'@
$script:mainHandle = [IntPtr]::Zero
$script:mainArea = -1
$callback = [NativeTestWindow+Callback]{ param($window,$data)
    [uint32]$candidateProcess = 0
    [void][NativeTestWindow]::GetWindowThreadProcessId($window,[ref]$candidateProcess)
    if ($candidateProcess -eq $target.Id -and [NativeTestWindow]::IsWindowVisible($window)) {
        $class = New-Object Text.StringBuilder 256
        [void][NativeTestWindow]::GetClassName($window,$class,256)
        # A minimized main window can have a zero-sized client. Never choose
        # Tao's event target (or an OS dialog) when areas are tied at zero.
        if ($class.ToString() -ne 'Tauri Window') { return $true }
        $title = New-Object Text.StringBuilder 256
        [void][NativeTestWindow]::GetWindowText($window,$title,256)
        if ($title.ToString() -notin @('Floating video','Veloce split preview')) {
            $area = New-Object NativeTestWindow+Rectangle
            if ([NativeTestWindow]::GetClientRect($window,[ref]$area)) {
                $pixels = ($area.right-$area.left)*($area.bottom-$area.top)
                if ($pixels -gt $script:mainArea) { $script:mainArea=$pixels; $script:mainHandle=$window }
            }
        }
    }
    return $true
}
[void][NativeTestWindow]::EnumWindows($callback,[IntPtr]::Zero)
if ($script:mainHandle -eq [IntPtr]::Zero) { throw 'Isolated main window was not found' }
if ($Inspect) {
    [uint32]$foregroundProcess = 0
    $foreground = [NativeTestWindow]::GetForegroundWindow()
    [void][NativeTestWindow]::GetWindowThreadProcessId($foreground,[ref]$foregroundProcess)
    @{ mainForeground=($foreground -eq $script:mainHandle); foregroundProcess=(Get-Process -Id $foregroundProcess -ErrorAction SilentlyContinue).ProcessName } | ConvertTo-Json -Compress
    return
}
if ($Focus) {
    [uint32]$foregroundProcess = 0
    $foregroundThread = [NativeTestWindow]::GetWindowThreadProcessId([NativeTestWindow]::GetForegroundWindow(), [ref]$foregroundProcess)
    $inputThread = [NativeTestWindow]::GetCurrentThreadId()
    $attached = [NativeTestWindow]::AttachThreadInput($inputThread, $foregroundThread, $true)
    try {
        [void][NativeTestWindow]::ShowWindow($script:mainHandle,9)
        [void][NativeTestWindow]::SetForegroundWindow($script:mainHandle)
    }
    finally { if ($attached) { [void][NativeTestWindow]::AttachThreadInput($inputThread, $foregroundThread, $false) } }
    # Foreground activation across input queues is asynchronous. Observe this
    # single request after detaching; do not retry activation or the scenario.
    $focusDeadline = [DateTime]::UtcNow.AddSeconds(2)
    while ([NativeTestWindow]::GetForegroundWindow() -ne $script:mainHandle -and [DateTime]::UtcNow -lt $focusDeadline) {
        Start-Sleep -Milliseconds 10
    }
    [void][NativeTestWindow]::GetWindowThreadProcessId([NativeTestWindow]::GetForegroundWindow(), [ref]$foregroundProcess)
    if ([NativeTestWindow]::GetForegroundWindow() -ne $script:mainHandle) { throw 'The isolated test application did not receive foreground focus' }
    return
}
$rectangle = New-Object NativeTestWindow+Rectangle
$client = New-Object NativeTestWindow+Rectangle
if (-not [NativeTestWindow]::GetWindowRect($script:mainHandle, [ref]$rectangle)) { throw 'Could not read the isolated test window bounds' }
if (-not [NativeTestWindow]::GetClientRect($script:mainHandle, [ref]$client)) { throw 'Could not read the isolated test client bounds' }
$outerWidth = $Width + ($rectangle.right - $rectangle.left) - ($client.right - $client.left)
$outerHeight = $Height + ($rectangle.bottom - $rectangle.top) - ($client.bottom - $client.top)
# Preserve position, focus and stacking order; resize only the verified process.
if (-not [NativeTestWindow]::SetWindowPos($script:mainHandle, [IntPtr]::Zero, $rectangle.left, $rectangle.top, $outerWidth, $outerHeight, 0x0014)) { throw 'Could not resize the isolated test window' }
