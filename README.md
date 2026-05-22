# Claude Usage Bar

![CI](https://github.com/sammyjdev/claude-usage-bar/actions/workflows/ci.yml/badge.svg)

A small cross-platform **system-tray app** (macOS / Linux / Windows) that shows
how much of your Claude plan (Pro/Max) you've used — the rolling **5-hour** and
**weekly** limits — at a glance, without opening anything.

## How it works

- On startup and **every 5 minutes**, it reads the OAuth token Claude Code
  already stores on this machine and calls the `/api/oauth/usage` endpoint.
- The **tray icon is a coloured dot** — green `<50%`, orange `50–80%`,
  red `≥80%` — driven by whichever window (5h or 7d) is fuller. Grey means an
  error state.
- Hover for the exact numbers (tooltip); click for the full breakdown
  (5h / 7d windows with reset times, per-model Sonnet/Opus, extra-usage credits,
  a manual *Atualizar agora*, and *Sair*).
- On **macOS** the menu bar additionally shows the text `5h X% · 7d Y%`.

It is a single Rust binary. The code is portable except for two `#[cfg]`-gated
spots — `src/token/` (where the token comes from) and `src/autostart.rs`
(per-OS login auto-start). Built with [`tao`] (event loop) and [`tray-icon`].

```
src/
  main.rs       CLI dispatch / launches the tray
  usage.rs      response model + JSON decode
  client.rs     HTTPS GET to the usage endpoint
  render.rs     colour, reset-time formatting, icon generation (portable)
  tray.rs       tray icon + menu + 5-min poll loop
  token/        macOS Keychain vs. credentials-file token source
  autostart.rs  per-OS install/uninstall of login auto-start
```

[`tao`]: https://crates.io/crates/tao
[`tray-icon`]: https://crates.io/crates/tray-icon

## Install — download a prebuilt binary (easiest)

No toolchain needed. Grab the latest build for your OS from the
[**Releases**](https://github.com/sammyjdev/claude-usage-bar/releases) page:

| OS | Asset | After download |
|----|-------|----------------|
| macOS | `claude-usage-bar-macos.zip` | unzip → `ClaudeUsageBar.app` |
| Linux | `claude-usage-bar-linux.tar.gz` | `tar -xzf` → the `claude-usage-bar` binary |
| Windows | `claude-usage-bar-windows.zip` | unzip → `claude-usage-bar.exe` |

Then jump to your OS section below for the auto-start step (`--install`).

> **Unsigned-app warning.** The binaries are not code-signed, so the first
> launch is blocked: on **macOS** right-click the app → *Open* → *Open*; on
> **Windows** click *More info* → *Run anyway*. After the first launch it runs
> normally.

To build from source instead, follow the steps below.

## Prerequisite — building from source (all platforms)

Install the Rust toolchain via [rustup](https://rustup.rs):

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

---

## Install — macOS

1. Build the app bundle (a bare binary will **not** show a menu bar item;
   macOS needs the `.app`):

       ./build-macos.sh

   This produces `ClaudeUsageBar.app` in the project directory.

2. Enable auto-start on login — run `--install` **from inside the bundle** so
   the LaunchAgent points at the `.app`:

       ./ClaudeUsageBar.app/Contents/MacOS/claude-usage-bar --install

3. On first run macOS asks for Keychain access to `Claude Code-credentials` —
   click **Always Allow**.

To remove auto-start: `./ClaudeUsageBar.app/Contents/MacOS/claude-usage-bar --uninstall`.
The `.app` can also be moved to `/Applications`; if you move it, re-run `--install`.

---

## Install — Linux

1. Install the GTK / app-indicator development libraries the tray needs.
   On Debian/Ubuntu:

       sudo apt install libgtk-3-dev libxdo-dev libayatana-appindicator3-dev

   (Other distros: the equivalent `gtk3`, `xdo`, and `ayatana-appindicator3`
   `-devel` packages.)

2. Build the release binary:

       cargo build --release

3. Put the binary somewhere stable, e.g. `~/.local/bin/`:

       install -Dm755 target/release/claude-usage-bar ~/.local/bin/claude-usage-bar

4. Enable auto-start (writes `~/.config/autostart/claude-usage-bar.desktop`):

       ~/.local/bin/claude-usage-bar --install

   `--install` records the binary's *current* path — install from the final
   location, and re-run `--install` if you move it.

> **GNOME:** GNOME has no system tray by default. Install the
> [AppIndicator](https://extensions.gnome.org/extension/615/appindicator-support/)
> extension. KDE, XFCE, and most other desktops work out of the box.

To remove auto-start: `claude-usage-bar --uninstall`.

---

## Install — Windows

1. Build the release binary (uses the MSVC toolchain):

       cargo build --release

2. Move `target\release\claude-usage-bar.exe` to a stable location, e.g.
   `%LOCALAPPDATA%\Programs\claude-usage-bar\`.

3. Enable auto-start (writes an `HKCU\...\Run` registry value pointing at the
   binary's *current* path):

       claude-usage-bar.exe --install

   Re-run `--install` if you move the executable.

To remove auto-start: `claude-usage-bar.exe --uninstall`.

> **Token caveat — unverified.** The app has so far only been run on macOS. On
> Windows it expects the token at `%USERPROFILE%\.claude\.credentials.json`. If
> Claude Code on Windows keeps the token in the Credential Manager / DPAPI
> instead, the icon will show `⚠ token` and `src/token/file.rs` needs a
> Windows-specific replacement.

---

## CLI reference

    claude-usage-bar             runs the tray app (default)
    claude-usage-bar --once      prints current usage and exits
    claude-usage-bar --selftest  runs internal asserts, exits 0 on pass
    claude-usage-bar --install   enable auto-start on login
    claude-usage-bar --uninstall disable auto-start

`--once` is the quickest way to confirm the token + endpoint work on a new
machine before dealing with the tray.

## Token source

| OS | Where the token is read from |
|----|------------------------------|
| macOS | Keychain item `Claude Code-credentials` |
| Linux | `~/.claude/.credentials.json` |
| Windows | `%USERPROFILE%\.claude\.credentials.json` *(unverified — see above)* |

The token is re-read on every poll (Claude Code rotates it). If the icon shows
`⚠ token`, the token store was not found — confirm Claude Code is signed in on
this machine.

## Notes

The `/api/oauth/usage` endpoint is undocumented and may change without notice.
If the icon shows `⚠ fmt`, the response shape changed and `src/usage.rs` needs
updating.
