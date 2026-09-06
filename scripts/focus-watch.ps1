# Ground truth for the focus probe: log every FOREGROUND change on the desktop,
# and every FOCUS / SHOW / HIDE event on windows owned by the process named
# $ProcessName (default msedge), with millisecond timestamps. A WinEvent hook
# receives these the moment the owning thread performs them; nothing is polled.
param([string]$Log = "$env:TEMP\ghost-focus-watch.log", [string]$ProcessName = 'msedge')
Add-Type -Namespace FW -Name N -MemberDefinition @'
public delegate void WinEventDelegate(IntPtr hWinEventHook, uint eventType, IntPtr hwnd, int idObject, int idChild, uint dwEventThread, uint dwmsEventTime);
[DllImport("user32.dll")] public static extern IntPtr SetWinEventHook(uint eventMin, uint eventMax, IntPtr hmodWinEventProc, WinEventDelegate lpfnWinEventProc, uint idProcess, uint idThread, uint dwFlags);
[DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint procId);
[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr h, System.Text.StringBuilder s, int n);
[DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, System.Text.StringBuilder s, int n);
[DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
[DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint flags);
[DllImport("user32.dll")] public static extern bool GetMessage(out MSG m, IntPtr h, uint a, uint b);
[DllImport("user32.dll")] public static extern bool TranslateMessage(ref MSG m);
[DllImport("user32.dll")] public static extern IntPtr DispatchMessage(ref MSG m);
public struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam; public IntPtr lParam; public uint time; public int x; public int y; }
'@
$names = @{ 0x0003 = 'FOREGROUND'; 0x8005 = 'FOCUS'; 0x8002 = 'SHOW'; 0x8003 = 'HIDE' }
$pids = @{}
function Get-ProcName($procId) { if (-not $pids.ContainsKey($procId)) { $pids[$procId] = try { (Get-Process -Id $procId -ErrorAction Stop).ProcessName } catch { '?' } }; $pids[$procId] }
"$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff') watcher started (pid $PID) for $ProcessName" | Out-File -Append -Encoding utf8 $Log
$cb = [FW.N+WinEventDelegate]{ param($hook, $ev, $hwnd, $idObject, $idChild, $thread, $ms)
  $procId = [uint32]0; [void][FW.N]::GetWindowThreadProcessId($hwnd, [ref]$procId)
  $pn = Get-ProcName $procId
  $isFg = ([int]$ev -eq 3)
  if (-not $isFg -and $pn -ne $ProcessName) { return }
  $root = [FW.N]::GetAncestor($hwnd, 2)
  $t = New-Object System.Text.StringBuilder 200; [void][FW.N]::GetWindowTextW($root, $t, 200)
  $c = New-Object System.Text.StringBuilder 96; [void][FW.N]::GetClassName($hwnd, $c, 96)
  $fg = [FW.N]::GetForegroundWindow()
  $obj = switch ([int]$idObject) { 0 { 'window' } -4 { 'client' } -8 { 'caret' } default { "obj$idObject" } }; $line = "$(Get-Date -Format 'HH:mm:ss.fff') $($names[[int]$ev]) [$obj] proc=$pn hwnd=$([int64]$hwnd) class=$($c.ToString()) root=$([int64]$root) title=`"$($t.ToString())`" fg=$([int64]$fg)"
  $line | Out-File -Append -Encoding utf8 $Log
}
$h1 = [FW.N]::SetWinEventHook(0x0003, 0x0003, [IntPtr]::Zero, $cb, 0, 0, 0)
$h2 = [FW.N]::SetWinEventHook(0x8002, 0x8003, [IntPtr]::Zero, $cb, 0, 0, 0)
$h3 = [FW.N]::SetWinEventHook(0x8005, 0x8005, [IntPtr]::Zero, $cb, 0, 0, 0)
$m = New-Object FW.N+MSG
while ([FW.N]::GetMessage([ref]$m, [IntPtr]::Zero, 0, 0)) { [void][FW.N]::TranslateMessage([ref]$m); [void][FW.N]::DispatchMessage([ref]$m) }
