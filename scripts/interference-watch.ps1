# Independent observer for the background-desktop probe. Records, with epoch-ms
# timestamps, everything that would tell a human "something took my computer":
#   - every FOREGROUND change on the desktop (WinEvent hook: nothing is polled)
#   - every INJECTED keystroke and mouse event (WH_KEYBOARD_LL / WH_MOUSE_LL see
#     the LLKHF_INJECTED / LLMHF_INJECTED flag on anything that came from
#     SendInput rather than the hardware)
# Hardware input is only COUNTED (never logged: those are the user's own keys),
# and its most recent time is carried on each foreground line as since_hw_ms so a
# change with no human input just before it can be told apart from one the user
# made. Hooks and the message loop live in C#, so a burst of mouse moves cannot
# stall a PowerShell callback and get the hook dropped.
#
#   powershell -NoProfile -ExecutionPolicy Bypass -File interference-watch.ps1 -Log <file> -Stop <stop-file>
#
# Runs until the stop file exists, then appends a summary line and exits.
param(
  [string]$Log = "$env:TEMP\ghost-interference-watch.jsonl",
  [string]$Stop = "$env:TEMP\ghost-interference-watch.stop"
)
Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Text;
using System.Diagnostics;
using System.Runtime.InteropServices;
public static class GhostWatch {
  public delegate IntPtr HookProc(int code, IntPtr w, IntPtr l);
  public delegate void WinEventDelegate(IntPtr h, uint ev, IntPtr hwnd, int idObject, int idChild, uint thread, uint ms);
  [DllImport("user32.dll", SetLastError = true)] static extern IntPtr SetWindowsHookEx(int id, HookProc p, IntPtr mod, uint tid);
  [DllImport("user32.dll")] static extern IntPtr CallNextHookEx(IntPtr h, int code, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern bool UnhookWindowsHookEx(IntPtr h);
  [DllImport("user32.dll")] static extern bool UnhookWinEvent(IntPtr h);
  [DllImport("user32.dll")] static extern IntPtr SetWinEventHook(uint min, uint max, IntPtr mod, WinEventDelegate p, uint pid, uint tid, uint flags);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool PeekMessage(out MSG m, IntPtr h, uint a, uint b, uint rm);
  [DllImport("user32.dll")] static extern bool TranslateMessage(ref MSG m);
  [DllImport("user32.dll")] static extern IntPtr DispatchMessage(ref MSG m);
  [DllImport("kernel32.dll", CharSet = CharSet.Unicode)] static extern IntPtr GetModuleHandle(string n);
  [StructLayout(LayoutKind.Sequential)] public struct MSG { public IntPtr hwnd; public uint message; public IntPtr wParam; public IntPtr lParam; public uint time; public int x; public int y; }
  [StructLayout(LayoutKind.Sequential)] struct KBDLLHOOKSTRUCT { public uint vkCode; public uint scanCode; public uint flags; public uint time; public IntPtr extra; }
  [StructLayout(LayoutKind.Sequential)] struct MSLLHOOKSTRUCT { public int x; public int y; public uint mouseData; public uint flags; public uint time; public IntPtr extra; }
  static HookProc kbProc, msProc; static WinEventDelegate weProc; static StreamWriter log;
  static long keyHw, keyInj, mouseHw, mouseInj, fgChanges, lastHw;
  static long Now() { return DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(); }
  static string Esc(string s) { return s.Replace("\\", "\\\\").Replace("\"", "\\\""); }
  static string Title(IntPtr h) { var sb = new StringBuilder(256); GetWindowTextW(h, sb, 256); return Esc(sb.ToString()); }
  static string Proc(IntPtr h) { uint pid; GetWindowThreadProcessId(h, out pid); try { return Process.GetProcessById((int)pid).ProcessName; } catch { return "?"; } }
  static void W(string s) { lock (log) { log.WriteLine(s); } }
  static IntPtr Kb(int code, IntPtr w, IntPtr l) {
    if (code >= 0) {
      int msg = (int)w;
      if (msg == 0x100 || msg == 0x104) {
        var k = (KBDLLHOOKSTRUCT)Marshal.PtrToStructure(l, typeof(KBDLLHOOKSTRUCT));
        if ((k.flags & 0x10) != 0) {
          keyInj++; var fg = GetForegroundWindow();
          W("{\"t\":" + Now() + ",\"kind\":\"key_injected\",\"vk\":" + k.vkCode + ",\"scan\":" + k.scanCode + ",\"extra\":" + (long)k.extra + ",\"fg\":" + (long)fg + ",\"fg_title\":\"" + Title(fg) + "\",\"fg_proc\":\"" + Proc(fg) + "\"}");
        } else { keyHw++; lastHw = Now(); }
      }
    }
    return CallNextHookEx(IntPtr.Zero, code, w, l);
  }
  static IntPtr Ms(int code, IntPtr w, IntPtr l) {
    if (code >= 0) {
      var m = (MSLLHOOKSTRUCT)Marshal.PtrToStructure(l, typeof(MSLLHOOKSTRUCT));
      int msg = (int)w;
      if ((m.flags & 0x01) != 0) {
        mouseInj++; var fg = GetForegroundWindow();
        W("{\"t\":" + Now() + ",\"kind\":\"mouse_injected\",\"msg\":" + msg + ",\"x\":" + m.x + ",\"y\":" + m.y + ",\"fg\":" + (long)fg + ",\"fg_title\":\"" + Title(fg) + "\"}");
      } else { mouseHw++; lastHw = Now(); }
    }
    return CallNextHookEx(IntPtr.Zero, code, w, l);
  }
  static void We(IntPtr h, uint ev, IntPtr hwnd, int idObject, int idChild, uint thread, uint ms) {
    if (ev != 3 || idObject != 0) return;
    fgChanges++;
    long now = Now();
    string since = lastHw == 0 ? "null" : (now - lastHw).ToString();
    W("{\"t\":" + now + ",\"kind\":\"foreground\",\"hwnd\":" + (long)hwnd + ",\"title\":\"" + Title(hwnd) + "\",\"proc\":\"" + Proc(hwnd) + "\",\"since_hw_ms\":" + since + "}");
  }
  public static void Run(string logPath, string stopFile) {
    log = new StreamWriter(logPath, true, new UTF8Encoding(false)); log.AutoFlush = true;
    kbProc = Kb; msProc = Ms; weProc = We;
    var mod = GetModuleHandle(null);
    var h1 = SetWindowsHookEx(13, kbProc, mod, 0);
    var h2 = SetWindowsHookEx(14, msProc, mod, 0);
    var h3 = SetWinEventHook(3, 3, IntPtr.Zero, weProc, 0, 0, 0);
    var fg = GetForegroundWindow();
    W("{\"t\":" + Now() + ",\"kind\":\"started\",\"pid\":" + Process.GetCurrentProcess().Id + ",\"hooks\":[" + ((long)h1 != 0) .ToString().ToLower() + "," + ((long)h2 != 0).ToString().ToLower() + "," + ((long)h3 != 0).ToString().ToLower() + "],\"fg\":" + (long)fg + ",\"fg_title\":\"" + Title(fg) + "\"}");
    MSG m;
    while (!File.Exists(stopFile)) {
      while (PeekMessage(out m, IntPtr.Zero, 0, 0, 1)) { TranslateMessage(ref m); DispatchMessage(ref m); }
      System.Threading.Thread.Sleep(5);
    }
    W("{\"t\":" + Now() + ",\"kind\":\"summary\",\"key_hw\":" + keyHw + ",\"key_injected\":" + keyInj + ",\"mouse_hw\":" + mouseHw + ",\"mouse_injected\":" + mouseInj + ",\"foreground_changes\":" + fgChanges + "}");
    UnhookWindowsHookEx(h1); UnhookWindowsHookEx(h2); UnhookWinEvent(h3);
    log.Close();
  }
}
'@
[GhostWatch]::Run($Log, $Stop)
