# Stand-in for the human during the background-desktop probe. Everything here
# goes through SendInput, exactly like a keyboard and mouse: the window it
# touches becomes the one Windows considers "the user's", which is the condition
# the probe needs (Ghost must work around a human who is actively typing).
#
# Steps run in this order, each only if given:
#   -SendToBack <h>     put window h behind the others (no activation)
#   -Raise <h>          put window h on top without activating it, so the click below lands
#   -ClickHwnd <h>      click the title bar of window h (activates it as a user would)
#   -AltTab             press Alt+Tab (switches to the previous window)
#   -Text <s>           type s, one character every -DelayMs
#   -CopyAll            Ctrl+A, Ctrl+C (so the caller can read what landed)
param(
  [long]$Raise = 0,
  [long]$SendToBack = 0,
  [long]$ClickHwnd = 0,
  [switch]$AltTab,
  [string]$Text = '',
  [int]$DelayMs = 40,
  [switch]$CopyAll
)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Sim {
  [StructLayout(LayoutKind.Sequential)] struct RECT { public int l, t, r, b; }
  [StructLayout(LayoutKind.Sequential)] struct KEYBDINPUT { public ushort wVk; public ushort wScan; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] struct MOUSEINPUT { public int dx; public int dy; public uint mouseData; public uint dwFlags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Explicit)] struct INPUTUNION { [FieldOffset(0)] public MOUSEINPUT mi; [FieldOffset(0)] public KEYBDINPUT ki; }
  [StructLayout(LayoutKind.Sequential)] struct INPUT { public uint type; public INPUTUNION u; }
  [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] static extern int GetSystemMetrics(int i);
  [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
  public static void Lower(long hwnd) { SetWindowPos((IntPtr)hwnd, (IntPtr)1, 0, 0, 0, 0, 0x0010 | 0x0002 | 0x0001); }
  // Put a window on top WITHOUT activating it, so a later click on its title
  // bar reaches it instead of whatever was covering it. The click is what makes
  // it the user's window; this only makes the click possible.
  public static void Raise(long hwnd) { SetWindowPos((IntPtr)hwnd, (IntPtr)0, 0, 0, 0, 0, 0x0010 | 0x0002 | 0x0001); }
  static void Send(params INPUT[] inputs) { SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT))); }
  static INPUT Key(ushort vk, bool up) { var i = new INPUT(); i.type = 1; i.u.ki.wVk = vk; i.u.ki.dwFlags = up ? 2u : 0u; return i; }
  static INPUT Uni(char c, bool up) { var i = new INPUT(); i.type = 1; i.u.ki.wScan = c; i.u.ki.dwFlags = 4u | (up ? 2u : 0u); return i; }
  public static void TypeText(string s, int delayMs) {
    foreach (char c in s) { Send(Uni(c, false), Uni(c, true)); System.Threading.Thread.Sleep(delayMs); }
  }
  public static void Chord(ushort mod, ushort key) {
    // Alt+Tab needs the switcher to see the modifier settle before the key, and
    // to stay up briefly before the release commits the choice; a chord sent as
    // four back-to-back events is routinely missed.
    Send(Key(mod, false)); System.Threading.Thread.Sleep(80);
    Send(Key(key, false)); System.Threading.Thread.Sleep(60);
    Send(Key(key, true)); System.Threading.Thread.Sleep(150);
    Send(Key(mod, true)); System.Threading.Thread.Sleep(250);
  }
  public static void ClickTitleBar(long hwnd) {
    RECT r; if (!GetWindowRect((IntPtr)hwnd, out r)) return;
    int x = (r.l + r.r) / 2 - 200, y = r.t + 12;
    int sw = GetSystemMetrics(78), sh = GetSystemMetrics(79), sx = GetSystemMetrics(76), sy = GetSystemMetrics(77);
    var mv = new INPUT(); mv.type = 0; mv.u.mi.dx = (int)(((long)(x - sx) * 65535) / sw); mv.u.mi.dy = (int)(((long)(y - sy) * 65535) / sh); mv.u.mi.dwFlags = 0x0001 | 0x8000 | 0x4000;
    var dn = new INPUT(); dn.type = 0; dn.u.mi.dwFlags = 0x0002;
    var upi = new INPUT(); upi.type = 0; upi.u.mi.dwFlags = 0x0004;
    Send(mv); System.Threading.Thread.Sleep(60); Send(dn, upi); System.Threading.Thread.Sleep(250);
  }
}
'@
if ($SendToBack -ne 0) { [Sim]::Lower($SendToBack); Start-Sleep -Milliseconds 150 }
if ($Raise -ne 0) { [Sim]::Raise($Raise); Start-Sleep -Milliseconds 150 }
if ($ClickHwnd -ne 0) { [Sim]::ClickTitleBar($ClickHwnd) }
if ($AltTab) { [Sim]::Chord(0x12, 0x09); Start-Sleep -Milliseconds 400 }
if ($Text -ne '') { [Sim]::TypeText($Text, $DelayMs) }
if ($CopyAll) { [Sim]::Chord(0x11, 0x41); [Sim]::Chord(0x11, 0x43); Start-Sleep -Milliseconds 200 }
