<#
.SYNOPSIS
    Regenerates docs/images/main-window-en.png and main-window-uk.png for the READMEs.

.DESCRIPTION
    Checklist step F1 asks for the main-window screenshot to be refreshed whenever the README
    changes. Doing that by hand means re-deriving all of the staging below, so it is a script.

    Two things it deliberately does NOT do:

    * **It never writes a PATH.** The list is filled through the Backups tab's Restore, which
      loads a Snapshot into the Working Copy — Apply is what would write, and Apply is never
      pressed. So no real PATH is touched, and none appears in the picture.
    * **It never takes the foreground or synthesises global input.** Somebody may be using the
      machine, and synthetic keystrokes go wherever focus is. The Backups tab is reached with the
      application's own `--tab backups` argument, the Snapshot row is focused by posting
      `WM_KEYDOWN` straight to the list, and Restore is pressed with `BM_CLICK`. All three carry
      handles and scalars only — never a pointer into this process, which is the one thing a
      cross-process `SendMessage` cannot dereference.

    The demo PATH carries one Entry of every Issue type, so the Status column earns its place in
    the picture. Its **clean** entries have to be real directories that are absent from this
    machine's System PATH: duplicates are evaluated across both Scopes, System first, so a clean
    row that the System PATH also holds would flag `Duplicate` and the screenshot would show a
    list where nothing is healthy. That precondition is asserted rather than assumed — on a
    machine where it fails, change the entries rather than the assertion.

.NOTES
    Run from a session that may launch a GUI. Needs a release build:
    cargo build --release --locked --target x86_64-pc-windows-msvc
#>
[CmdletBinding()]
param(
    [string]$Exe = (Join-Path $PSScriptRoot '..\target\x86_64-pc-windows-msvc\release\PathMaster.exe'),
    [string]$Out = (Join-Path $PSScriptRoot '..\docs\images'),
    [string]$Staging = (Join-Path ([System.IO.Path]::GetTempPath()) 'pathmaster-screenshot')
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class Shot {
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc p, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr parent, EnumProc p, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindowEnabled(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr wp, IntPtr lp);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr h, uint msg, IntPtr wp, IntPtr lp);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }

  public static IntPtr[] TopLevel(uint pid) {
    var found = new System.Collections.ArrayList();
    EnumWindows((h, l) => {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p == pid && IsWindowVisible(h)) found.Add(h);
      return true;
    }, IntPtr.Zero);
    return (IntPtr[])found.ToArray(typeof(IntPtr));
  }
  public static IntPtr[] Children(IntPtr parent) {
    var found = new System.Collections.ArrayList();
    EnumChildWindows(parent, (h, l) => { found.Add(h); return true; }, IntPtr.Zero);
    return (IntPtr[])found.ToArray(typeof(IntPtr));
  }
  public static string Text(IntPtr h) { var s = new StringBuilder(1024); GetWindowTextW(h, s, 1024); return s.ToString(); }
  public static string Cls(IntPtr h) { var s = new StringBuilder(256); GetClassNameW(h, s, 256); return s.ToString(); }
  public static int[] Rect(IntPtr h) { RECT r; GetWindowRect(h, out r); return new int[]{ r.L, r.T, r.R - r.L, r.B - r.T }; }

  public const uint WM_KEYDOWN = 0x0100, WM_KEYUP = 0x0101, BM_CLICK = 0x00F5;
  const uint LVM_GETITEMCOUNT = 0x1004, LVM_GETNEXTITEM = 0x100C;
  public static int ItemCount(IntPtr list) { return (int)SendMessageW(list, LVM_GETITEMCOUNT, IntPtr.Zero, IntPtr.Zero); }
  // LVNI_FOCUSED = 1; wParam -1 means "search from the start".
  public static int FocusedItem(IntPtr list) { return (int)SendMessageW(list, LVM_GETNEXTITEM, (IntPtr)(-1), (IntPtr)1); }
}
'@

$VK_DOWN = 0x28

# The demo PATH, in the order a real one would mix them. `clean` marks the rows the picture needs
# to show with an empty Status column; everything else is there for the Issue type in its comment.
$demo = @(
    @{ entry = 'C:\scoop\shims';                       clean = $true  }
    @{ entry = 'C:\scoop\apps\python\current';         clean = $true  }
    @{ entry = 'C:\scoop\apps\python\current\Scripts'; clean = $true  }
    @{ entry = 'C:\Tools\Python313\Scripts';           clean = $false } # Missing
    @{ entry = 'C:\scoop\apps\nodejs\current';         clean = $true  }
    @{ entry = '"C:\Program Files\Git\cmd"';           clean = $false } # Missing, Quoted
    @{ entry = '.\bin';                                clean = $false } # Relative
    @{ entry = 'C:\scoop\apps\gcc\current\bin';        clean = $true  }
    @{ entry = 'C:\SCOOP\SHIMS';                       clean = $false } # Duplicate of row 1
    @{ entry = '';                                     clean = $false } # Empty
)
# Restore's label per language, so the button is found by what it says rather than by position.
$restoreLabel = @{ en = 'Restore'; uk = 'Відновити' }

if (-not (Test-Path $Exe)) { throw "no release build at $Exe" }
$Exe = (Resolve-Path $Exe).Path
New-Item -ItemType Directory -Force $Out | Out-Null

# The precondition the picture depends on. Normalisation folds case and trailing separators, so
# the comparison does too.
$normalise = { param($p) $p.Trim('"').TrimEnd('\').Replace('/', '\').ToLowerInvariant() }
$systemPath = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Environment' -Name Path).Path
$systemEntries = @($systemPath -split ';' | Where-Object { $_ } | ForEach-Object { & $normalise $_ })
foreach ($row in $demo | Where-Object { $_.clean }) {
    if (-not (Test-Path -PathType Container $row.entry)) {
        throw "'$($row.entry)' is meant to read as healthy but does not exist - pick another"
    }
    if ($systemEntries -contains (& $normalise $row.entry)) {
        throw "'$($row.entry)' is in this machine's System PATH and would flag Duplicate - pick another"
    }
}

foreach ($lang in 'en', 'uk') {
    # A fresh Data Directory each time: the language is read once at startup, and a leftover
    # settings.json would carry the previous run's window geometry into this one.
    if (Test-Path $Staging) { Remove-Item -Recurse -Force $Staging }
    New-Item -ItemType Directory -Force "$Staging\data\backups" | Out-Null
    Copy-Item $Exe "$Staging\PathMaster.exe"

    Set-Content -Path "$Staging\data\settings.json" -Encoding utf8 -Value (
        [pscustomobject]@{ language = $lang; maxBackups = 50 } | ConvertTo-Json)
    Set-Content -Path "$Staging\data\backups\2026-08-24T10-00-00-User.json" -Encoding utf8 -Value (
        [pscustomobject]@{
            timestamp = '2026-08-24T10:00:00'
            scope     = 'User'
            valueType = 'REG_EXPAND_SZ'
            entries   = @($demo.entry)
        } | ConvertTo-Json)

    # The application's own argument (spec §9). It is also the one page change that needs no
    # message sent to a notebook: wx listens for the tab control's own notification, which
    # TCM_SETCURSEL does not send.
    $app = Start-Process -FilePath "$Staging\PathMaster.exe" -ArgumentList '--tab', 'backups' -PassThru
    try {
        Start-Sleep -Seconds 4
        $window = @([Shot]::TopLevel($app.Id))[0]
        if (-not $window) { throw "no window appeared for $lang" }
        $children = [Shot]::Children($window)

        # Every page's list exists as a child at once and only the active page's is visible.
        # Without that filter the first match is the User tab's list — the real PATH.
        $lists = @($children | Where-Object { [Shot]::Cls($_) -eq 'SysListView32' -and [Shot]::IsWindowVisible($_) })
        if ($lists.Count -ne 1) { throw "expected one visible list, saw $($lists.Count)" }
        $snapshots = $lists[0]
        if ([Shot]::ItemCount($snapshots) -ne 1) {
            throw "expected the one staged Snapshot, saw $([Shot]::ItemCount($snapshots)) rows"
        }

        # Restore follows the focused Snapshot row, so the row comes first.
        [Shot]::PostMessageW($snapshots, [Shot]::WM_KEYDOWN, [IntPtr]$VK_DOWN, [IntPtr]0) | Out-Null
        [Shot]::PostMessageW($snapshots, [Shot]::WM_KEYUP, [IntPtr]$VK_DOWN, [IntPtr]0) | Out-Null
        Start-Sleep -Milliseconds 900
        if ([Shot]::FocusedItem($snapshots) -lt 0) { throw "no Snapshot row took focus" }

        $button = $children | Where-Object {
            [Shot]::Cls($_) -eq 'Button' -and [Shot]::IsWindowVisible($_) -and
            [Shot]::Text($_) -eq $restoreLabel[$lang]
        } | Select-Object -First 1
        if (-not $button) { throw "no '$($restoreLabel[$lang])' button found" }
        if (-not [Shot]::IsWindowEnabled($button)) { throw "Restore is disabled - nothing would happen" }

        [Shot]::SendMessageW($button, [Shot]::BM_CLICK, [IntPtr]0, [IntPtr]0) | Out-Null
        Start-Sleep -Seconds 2

        $r = [Shot]::Rect($window)
        $bitmap = New-Object System.Drawing.Bitmap($r[2], $r[3])
        $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
        $hdc = $graphics.GetHdc()
        # PW_RENDERFULLCONTENT: without it a composited window comes back blank.
        [Shot]::PrintWindow($window, $hdc, 2) | Out-Null
        $graphics.ReleaseHdc($hdc)
        $graphics.Dispose()
        $path = Join-Path $Out "main-window-$lang.png"
        $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
        $bitmap.Dispose()
        Write-Host ("{0}: {1}x{2}" -f $path, $r[2], $r[3])
    }
    finally {
        $app.Kill()
        Start-Sleep -Milliseconds 500
    }
}
if (Test-Path $Staging) { Remove-Item -Recurse -Force $Staging -ErrorAction SilentlyContinue }
