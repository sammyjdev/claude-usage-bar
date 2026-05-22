# Claude Usage Bar

Cross-platform (macOS / Linux / Windows) system-tray app showing Claude plan
(Pro/Max) usage against the 5h and weekly limits. Data comes from the
`/api/oauth/usage` endpoint, authenticated with the OAuth token that Claude
Code stores on the local machine.

## Build

Requires the Rust toolchain (`rustup`).

    cargo build --release        # Linux / Windows — run target/release/claude-usage-bar
    ./build-macos.sh             # macOS — produces ClaudeUsageBar.app

macOS needs the `.app` bundle: a bare binary does not show a menu bar item.

## Run

    claude-usage-bar             # runs the tray app
    claude-usage-bar --once      # prints current usage and exits
    claude-usage-bar --selftest  # runs internal asserts, exits 0 on pass
    claude-usage-bar --install   # enable auto-start on login
    claude-usage-bar --uninstall # disable auto-start

## Token source

- **macOS** — Keychain item `Claude Code-credentials`.
- **Linux** — `~/.claude/.credentials.json`.
- **Windows** — assumed `%USERPROFILE%\.claude\.credentials.json`. **Unverified:**
  the app has so far only been run on macOS. If Claude Code on Windows keeps its
  token in the Credential Manager / DPAPI instead of a plain file, the icon will
  show `⚠ token` and `src/token/file.rs` needs a Windows-specific replacement.

If the icon shows `⚠ token`, the token store was not found — confirm Claude
Code is signed in on this machine.

## Platform notes

- **Linux** — the tray uses `StatusNotifierItem`. GNOME has no tray by default;
  install the *AppIndicator* extension. KDE / XFCE work out of the box.
- **Windows** — `--install` writes an `HKCU\...\Run` registry value.
- The `/api/oauth/usage` endpoint is undocumented and may change. `⚠ fmt` means
  the response shape changed and `src/usage.rs` needs updating.
