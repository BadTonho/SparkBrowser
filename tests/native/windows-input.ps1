param([Parameter(Mandatory=$true)][string]$Executable, [Parameter(Mandatory=$true)][int]$Key, [switch]$Up, [switch]$Stroke, [switch]$Prompt, [string]$RunDirectory)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeTestInput {
    [StructLayout(LayoutKind.Sequential)] public struct Keyboard { public ushort key, scan; public uint flags, time; public IntPtr extra; }
    [StructLayout(LayoutKind.Sequential)] public struct Mouse { public int x,y; public uint data,flags,time; public IntPtr extra; }
    [StructLayout(LayoutKind.Explicit)] public struct Payload { [FieldOffset(0)] public Keyboard keyboard; [FieldOffset(0)] public Mouse mouse; }
    [StructLayout(LayoutKind.Sequential)] public struct Input { public uint type; public Payload payload; }
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr window);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr window,int command);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint from, uint to, bool attach);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr window, out uint process);
    [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] input, int size);
    public static void Key(ushort key, bool up) {
        var input = new Input { type=1, payload=new Payload { keyboard=new Keyboard { key=key, flags=up?2u:0u } } };
        if (SendInput(1, new[]{input}, Marshal.SizeOf(typeof(Input)))!=1) throw new InvalidOperationException("Native test input failed");
    }
}
'@
# Only the already identity-verified isolated browser may receive key-down events.
# Always release injected keys in the caller's finally, even if the browser loses focus.
if (-not $Up) {
    $target = Get-Process | Where-Object { $_.Path -and [string]::Equals($_.Path, $Executable, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -First 1
    if (-not $target -or $target.MainWindowHandle -eq 0) { throw 'Isolated test window was not found' }
    [uint32]$foregroundProcess = 0
    $foregroundThread = [NativeTestInput]::GetWindowThreadProcessId([NativeTestInput]::GetForegroundWindow(), [ref]$foregroundProcess)
    if ($foregroundProcess -ne $target.Id -and $Key -eq 0x11) {
        $inputThread = [NativeTestInput]::GetCurrentThreadId()
        $attached = [NativeTestInput]::AttachThreadInput($inputThread, $foregroundThread, $true)
        try {
            [void][NativeTestInput]::ShowWindow($target.MainWindowHandle,9)
            $shell = New-Object -ComObject WScript.Shell
            [void]$shell.AppActivate($target.Id)
            [void][NativeTestInput]::SetForegroundWindow($target.MainWindowHandle)
            Start-Sleep -Milliseconds 100
        }
        finally { if ($attached) { [void][NativeTestInput]::AttachThreadInput($inputThread, $foregroundThread, $false) } }
    }
    [void][NativeTestInput]::GetWindowThreadProcessId([NativeTestInput]::GetForegroundWindow(), [ref]$foregroundProcess)
    $foregroundPath = (Get-Process -Id $foregroundProcess).Path
    if (-not [string]::Equals([IO.Path]::GetFullPath($foregroundPath), [IO.Path]::GetFullPath($Executable), [StringComparison]::OrdinalIgnoreCase)) {
        $verifiedPrompt = $false
        if ($Prompt -and $Key -eq 0x0D -and $RunDirectory) {
            $runsRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../.cache/native-runs')) + [IO.Path]::DirectorySeparatorChar
            $scenario = [IO.Path]::GetFullPath($RunDirectory)
            if (-not $scenario.StartsWith($runsRoot,[StringComparison]::OrdinalIgnoreCase)) { throw 'Prompt must belong to an isolated scenario' }
            $profilePattern = [regex]::Escape((Join-Path $scenario 'profile')) + '[\\/]'
            $browser = Get-CimInstance Win32_Process -Filter "ProcessId = $foregroundProcess"
            $verifiedPrompt = $browser.Name -eq 'msedgewebview2.exe' -and $browser.CommandLine -match $profilePattern
        }
        if (-not $verifiedPrompt) { throw "Native keyboard test requires its isolated browser in the foreground (actual process: $((Get-Process -Id $foregroundProcess).ProcessName))" }
    }
}
if ($Key -notin @(0x11,0x10,0x57,0x54,0x4C,0x76,0x0D)) { throw 'Unsupported test key' }
if ($Stroke) {
    if ($Up) { throw 'A complete keystroke cannot be a key release' }
    try {
        [NativeTestInput]::Key([uint16]$Key, $false)
        Start-Sleep -Milliseconds 30
    } finally { [NativeTestInput]::Key([uint16]$Key, $true) }
} else { [NativeTestInput]::Key([uint16]$Key, [bool]$Up) }
