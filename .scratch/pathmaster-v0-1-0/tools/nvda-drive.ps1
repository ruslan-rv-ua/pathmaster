<#
.SYNOPSIS
  Drive a prototype with synthetic keystrokes and read back what NVDA said.

.DESCRIPTION
  Measuring what a screen reader announces normally needs a human at the keyboard. This does it
  unattended: it sends one key at a time, waits until NVDA has finished speaking, and returns the
  slice of NVDA's own log that the run produced.

  Requires NVDA's logging level to be Input/Output (NVDA+Ctrl+G -> General). At that level NVDA
  writes every utterance as a `Speaking [...]` line and every keystroke as an `Input:` line.
  It logs system-wide, so put it back to Info afterwards.

  Only the bytes appended during the run are read, so unrelated speech from before it is never
  touched.

.EXAMPLE
  .\nvda-drive.ps1 -Launch -Exe ..\prototypes\02-nvda-baseline\target\release\nvda-baseline.exe
  .\nvda-drive.ps1 -Keys 'TAB,DOWN,DOWN,CTRL+HOME'
  .\nvda-drive.ps1 -Probe
  .\nvda-drive.ps1 -Keys 'ALT+F4'

.NOTES
  Written for ticket 02 (NVDA baseline). Ticket 08 needs the same loop, so it lives here rather
  than inside a prototype. Do not add accessibility calls to 02's prototype — it is the baseline.
#>
param(
  [string]$Exe   = '',
  [string]$Keys  = '',
  [switch]$Launch,
  [switch]$Probe,
  [string]$Log       = (Join-Path $env:TEMP 'nvda.log'),
  [string]$StateFile = (Join-Path $env:TEMP 'nvda-drive-state.txt'),
  [int]$AppPid = 0,
  [int]$QuietMs = 900,
  [int]$MaxWaitMs = 6000
)

$ErrorActionPreference = 'Stop'

if (-not ([System.Management.Automation.PSTypeName]'NvdaDrive').Type) {
Add-Type @'
using System; using System.Text; using System.Runtime.InteropServices;
public class NvdaDrive {
  [DllImport("user32.dll")] public static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr dwExtraInfo);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint idAttach, uint idAttachTo, bool fAttach);
  [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
  [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr h, EnumProc cb, IntPtr p);
  [DllImport("user32.dll")] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool GetGUIThreadInfo(uint t, ref GUITHREADINFO i);
  [DllImport("oleacc.dll")] public static extern int AccessibleObjectFromWindow(IntPtr hwnd, uint id, ref Guid iid, [MarshalAs(UnmanagedType.IUnknown)] out object ppv);
  public delegate bool EnumProc(IntPtr h, IntPtr p);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int l,t,r,b; }
  [StructLayout(LayoutKind.Sequential)] public struct GUITHREADINFO {
    public int cbSize; public int flags;
    public IntPtr hwndActive, hwndFocus, hwndCapture, hwndMenuOwner, hwndMoveSize, hwndCaret;
    public RECT rc; }
}
'@
}

# --- key table -------------------------------------------------------------

$VK = @{
  'TAB'=0x09; 'ENTER'=0x0D; 'ESC'=0x1B; 'SPACE'=0x20; 'BACK'=0x08; 'DEL'=0x2E; 'APPS'=0x5D
  'DOWN'=0x28; 'UP'=0x26; 'LEFT'=0x25; 'RIGHT'=0x27; 'HOME'=0x24; 'END'=0x23; 'PGUP'=0x21; 'PGDN'=0x22
  'CTRL'=0x11; 'SHIFT'=0x10; 'ALT'=0x12; 'INS'=0x2D
  'F1'=0x70;'F2'=0x71;'F3'=0x72;'F4'=0x73;'F5'=0x74;'F6'=0x75;'F7'=0x76;'F8'=0x77;'F9'=0x78;'F10'=0x79;'F11'=0x7A;'F12'=0x7B
}
0..25 | ForEach-Object { $VK[[char](65 + $_)] = 65 + $_ }   # A..Z

# Keys that must carry KEYEVENTF_EXTENDEDKEY, or Windows reads them as their numpad twins.
# INS is here because NVDA's default modifier is the *extended* Insert.
$EXTENDED = @('DOWN','UP','LEFT','RIGHT','HOME','END','PGUP','PGDN','INS','DEL','APPS')

function Send-One([string]$Spec) {
  $parts = $Spec.Split('+')
  $key   = $parts[-1]
  # Guard the single-token case: $parts[0..-1] is the range 0,-1 in PowerShell and would
  # press the key twice as its own modifier.
  $mods  = @()
  if ($parts.Count -gt 1) { $mods = @($parts[0..($parts.Count-2)]) | Where-Object { $_ } }

  foreach ($m in $mods) {
    if (-not $VK.ContainsKey($m)) { throw "unknown key '$m' in '$Spec'" }
    $f = if ($EXTENDED -contains $m) { 1 } else { 0 }
    [NvdaDrive]::keybd_event([byte]$VK[$m], 0, $f, [UIntPtr]::Zero); Start-Sleep -Milliseconds 25
  }
  if (-not $VK.ContainsKey($key)) { throw "unknown key '$key' in '$Spec'" }
  $f = if ($EXTENDED -contains $key) { 1 } else { 0 }
  [NvdaDrive]::keybd_event([byte]$VK[$key], 0, $f, [UIntPtr]::Zero); Start-Sleep -Milliseconds 40
  [NvdaDrive]::keybd_event([byte]$VK[$key], 0, ($f -bor 2), [UIntPtr]::Zero)
  foreach ($m in ($mods | Sort-Object -Descending)) {
    $f = if ($EXTENDED -contains $m) { 1 } else { 0 }
    [NvdaDrive]::keybd_event([byte]$VK[$m], 0, ($f -bor 2), [UIntPtr]::Zero); Start-Sleep -Milliseconds 25
  }
}

# --- log helpers -----------------------------------------------------------

function Get-LogLen { try { (Get-Item $Log).Length } catch { 0 } }

# NVDA speaks asynchronously, so pace on the log going quiet rather than on a fixed sleep.
function Wait-Quiet {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $last = Get-LogLen; $lastChange = $sw.ElapsedMilliseconds
  while ($sw.ElapsedMilliseconds -lt $MaxWaitMs) {
    Start-Sleep -Milliseconds 150
    $now = Get-LogLen
    if ($now -ne $last) { $last = $now; $lastChange = $sw.ElapsedMilliseconds }
    elseif (($sw.ElapsedMilliseconds - $lastChange) -ge $QuietMs) { return }
  }
}

function Read-Appended([long]$Offset) {
  $fs = [System.IO.File]::Open($Log, 'Open', 'Read', 'ReadWrite')   # NVDA holds it open for writing
  [void]$fs.Seek($Offset, 'Begin')
  $sr = New-Object System.IO.StreamReader($fs, [System.Text.Encoding]::UTF8)
  $text = $sr.ReadToEnd(); $sr.Close(); $fs.Close()
  return $text
}

function Get-FgPid {
  $h = [NvdaDrive]::GetForegroundWindow(); $p = 0
  [void][NvdaDrive]::GetWindowThreadProcessId($h, [ref]$p); return $p
}

function Get-FocusHwnd {
  $fg = [NvdaDrive]::GetForegroundWindow()
  $tid = [NvdaDrive]::GetWindowThreadProcessId($fg, [ref]([uint32]0))
  $g = New-Object NvdaDrive+GUITHREADINFO
  $g.cbSize = [Runtime.InteropServices.Marshal]::SizeOf($g)
  [void][NvdaDrive]::GetGUIThreadInfo($tid, [ref]$g)
  return $g.hwndFocus
}

function Get-Class([IntPtr]$h) {
  $sb = New-Object Text.StringBuilder 256
  [void][NvdaDrive]::GetClassName($h, $sb, 256); return $sb.ToString()
}

# Windows refuses SetForegroundWindow from a process that does not own the foreground. Three
# standard releases of that lock, tried in order: a synthetic ALT (which clears the lock for the
# calling thread), attaching our input queue to the foreground thread, and BringWindowToTop.
function Set-Foreground([IntPtr]$Hwnd, [int]$WantPid) {
  for ($try = 0; $try -lt 5; $try++) {
    if ((Get-FgPid) -eq $WantPid) { return $true }
    [void][NvdaDrive]::ShowWindow($Hwnd, 9)                                  # SW_RESTORE
    [NvdaDrive]::keybd_event([byte]0x12, 0, 0, [UIntPtr]::Zero)              # ALT down
    [NvdaDrive]::keybd_event([byte]0x12, 0, 2, [UIntPtr]::Zero)              # ALT up
    $fgThread = [NvdaDrive]::GetWindowThreadProcessId([NvdaDrive]::GetForegroundWindow(), [ref]([uint32]0))
    $myThread = [NvdaDrive]::GetCurrentThreadId()
    $attached = $false
    if ($fgThread -ne 0 -and $fgThread -ne $myThread) {
      $attached = [NvdaDrive]::AttachThreadInput($myThread, $fgThread, $true)
    }
    [void][NvdaDrive]::BringWindowToTop($Hwnd)
    [void][NvdaDrive]::SetForegroundWindow($Hwnd)
    if ($attached) { [void][NvdaDrive]::AttachThreadInput($myThread, $fgThread, $false) }
    Start-Sleep -Milliseconds 350
  }
  return ((Get-FgPid) -eq $WantPid)
}

function Assert-LogLevel {
  if (-not (Test-Path $Log)) { Write-Warning "no NVDA log at $Log"; return }
  $tail = Get-Content $Log -Tail 400 -ErrorAction SilentlyContinue
  if (-not ($tail | Where-Object { $_ -like 'Speaking*' -or $_ -like 'Input:*' })) {
    Write-Warning "no Speaking/Input lines in the last 400 log lines - NVDA's logging level is probably still Info. Raise it with NVDA+Ctrl+G -> General -> Logging level -> Input/Output, or this run records nothing."
  }
}

# --- launch ----------------------------------------------------------------

if ($Launch) {
  if (-not $Exe) { throw '-Launch needs -Exe' }
  $Exe = (Resolve-Path $Exe).Path
  Assert-LogLevel
  $offset = Get-LogLen
  $proc = Start-Process -FilePath $Exe -PassThru
  Start-Sleep -Seconds 2; $proc.Refresh()
  $ok = Set-Foreground $proc.MainWindowHandle $proc.Id
  if (-not $ok) { Write-Warning "could not bring the window to the foreground (fgPid=$(Get-FgPid)); -Keys will retry" }
  Start-Sleep -Milliseconds 400
  Wait-Quiet
  "$($proc.Id)`t$offset" | Set-Content $StateFile
  "LAUNCHED pid=$($proc.Id) title='$($proc.MainWindowTitle)' fgPid=$(Get-FgPid) logOffset=$offset"
  # Only the process that started the app is granted the right to take the foreground, and that
  # right does not survive into a later PowerShell process. So -Launch and -Keys in one call is the
  # reliable shape; a separate -Keys call works only while the window is still foreground.
  if (-not $Keys) { return }
}

if (-not (Test-Path $StateFile)) { throw "no state file at $StateFile - run with -Launch first" }
$state     = (Get-Content $StateFile) -split "`t"
$targetPid = if ($AppPid) { $AppPid } else { [int]$state[0] }

# --- probe: what does the control itself say its state is? -----------------
# The cross-check that separates "the screen reader is silent" from "nothing happened".

if ($Probe) {
  $fg = [NvdaDrive]::GetForegroundWindow()
  $focus = Get-FocusHwnd
  "foreground pid : $(Get-FgPid) (target $targetPid)"
  "focus hwnd     : $focus  class=$(Get-Class $focus)"

  $kids = @()
  $cb = [NvdaDrive+EnumProc]{ param($h,$p) $script:kids += [pscustomobject]@{ h=$h; cls=(Get-Class $h) }; return $true }
  [void][NvdaDrive]::EnumChildWindows($fg, $cb, [IntPtr]::Zero)
  ($kids | Group-Object cls | ForEach-Object { "  child: $($_.Name) x$($_.Count)" })

  foreach ($lv in ($kids | Where-Object { $_.cls -eq 'SysListView32' })) {
    $count = [NvdaDrive]::SendMessage($lv.h, 0x1004, [IntPtr]::Zero, [IntPtr]::Zero)   # LVM_GETITEMCOUNT
    $sel   = [NvdaDrive]::SendMessage($lv.h, 0x1032, [IntPtr]::Zero, [IntPtr]::Zero)   # LVM_GETSELECTEDCOUNT
    $foc   = [NvdaDrive]::SendMessage($lv.h, 0x100C, [IntPtr](-1), [IntPtr]1)          # LVM_GETNEXTITEM/LVNI_FOCUSED
    "listview $($lv.h): items=$count selected=$sel focusedIndex=$foc focused=$($lv.h -eq $focus)"
  }

  # MSAA on the focused window: are the rows there, and does accFocus name the right one?
  $iid = [Guid]'618736e0-3c3d-11cf-810c-00aa00389b71'
  $acc = $null
  $hr = [NvdaDrive]::AccessibleObjectFromWindow($focus, [uint32]4294967292, [ref]$iid, [ref]$acc)  # OBJID_CLIENT
  if ($hr -eq 0 -and $acc) {
    "msaa hr=0 childCount=$($acc.accChildCount) role=$($acc.accRole(0)) accFocus=$($acc.accFocus)"
    foreach ($i in 1..([Math]::Min(12, [int]$acc.accChildCount))) {
      try { "  child {0,2} : name='{1}' role={2} state=0x{3:X}" -f $i, $acc.accName($i), $acc.accRole($i), [int]$acc.accState($i) }
      catch { "  child $i : <$($_.Exception.Message.Trim())>" }
    }
  } else { "msaa: AccessibleObjectFromWindow failed hr=$hr" }
  return
}

# --- send keys -------------------------------------------------------------

if (-not $Keys) { throw 'nothing to do - pass -Launch, -Probe or -Keys' }

$offset = Get-LogLen     # read only what this run appends
$fg = Get-FgPid
if ($fg -ne $targetPid) {
  $p = Get-Process -Id $targetPid -ErrorAction SilentlyContinue
  if (-not $p) { "ABORT: target process $targetPid is gone. No keys sent."; return }
  [void](Set-Foreground $p.MainWindowHandle $targetPid)
  $fg = Get-FgPid
}
if ($fg -ne $targetPid) { "ABORT: foreground pid $fg is not the target ($targetPid). No keys sent."; return }

$sent = @()
foreach ($spec in ($Keys -split ',' | ForEach-Object { $_.Trim().ToUpper() } | Where-Object { $_ })) {
  # Re-check before every key: synthetic input goes to whatever is focused, so never keep
  # typing into a window that stole focus mid-run.
  if ((Get-FgPid) -ne $targetPid) { "ABORT after $($sent.Count) keys: focus left the target."; break }
  Send-One $spec
  $sent += $spec
  Wait-Quiet
}

"SENT: $($sent -join ' , ')"
$chunk = Read-Appended $offset
"--- NVDA log, appended during this run ---"
($chunk -split "`r?`n") | Where-Object { $_ -match '^(Input:|Speaking)' }
