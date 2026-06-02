# Claude Usage Bar

![CI](https://github.com/sammyjdev/claude-usage-bar/actions/workflows/ci.yml/badge.svg)

A small cross-platform **system-tray app** (macOS / Linux / Windows) that shows
your recent Claude Code token usage at a glance, without opening anything. It
reads Claude Code's own local session logs. It makes **no network calls and
never touches your token or credentials.**

## How it works

- On startup and **every 60 seconds**, it parses the JSONL session logs Claude
  Code writes under `~/.claude/projects/`, summing token usage into an active
  **5h** block and a rolling **7d** total. Same source as `ccusage`.
- Tokens counted are `input + output + cache_creation`. Cache *reads* are
  excluded on purpose: they are cheap and automatic, and would otherwise be
  ~97% of the number, drowning out real consumption.
- **Real percentage by auto-calibration.** The plan limit is not published, so
  the app learns it: when you hit your 5h limit, Claude Code logs an event
  (`429` + `resets ...`), and the token count at that moment becomes your
  learned limit. After the first hit, the 5h window shows a real **percent**
  (`5h 78%`); before it, it shows absolute **tokens** (`5h 1.8M tok`), never a
  fake percent. See [docs/design/limits-and-calibration.md](docs/design/limits-and-calibration.md).
- The **tray icon is a coloured dot**: green `<50%`, orange `50-80%`,
  red `>=80%`, driven by whichever window (5h or 7d) is fuller. Before
  calibration it falls back to a heuristic cap (`FIVE_HOUR_CAP` /
  `SEVEN_DAY_CAP` in `src/logs.rs`). Grey means an error state.
- Hover for the exact values (tooltip); click for the full breakdown
  (5h / 7d windows, per-model Sonnet/Opus, a manual *Atualizar agora*, and
  *Sair*).
- On **macOS** the menu bar additionally shows the text `5h X · 7d Y`.

It is a single Rust binary. The code is portable except for two `#[cfg]`-gated
spots: `src/logs.rs` (the log directory location) and `src/autostart.rs`
(per-OS login auto-start). Built with [`tao`] (event loop) and [`tray-icon`].

```
src/
  main.rs        CLI dispatch / launches the tray
  usage.rs       usage model (Window / Usage structs)
  logs.rs        local JSONL log parsing + 5h/7d aggregation (testable, no I/O in logic)
  calibration.rs learned plan limit, persisted across runs
  render.rs      colour, token/reset formatting, icon generation (portable)
  tray.rs        tray icon + menu + 60s poll loop
  autostart.rs   per-OS install/uninstall of login auto-start
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

   No Keychain or permission prompt is involved: the app only reads plain log
   files under `~/.claude/projects/`.

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

> **Path caveat, unverified.** The app has so far only been run on macOS. It
> expects the logs at `%USERPROFILE%\.claude\projects\` on Windows (and
> `~/.claude/projects/` on Linux). If Claude Code on your machine writes them
> elsewhere, the icon shows `⚠ logs`; set `CLAUDE_CONFIG_DIR` to the `.claude`
> directory, or adjust `logs_dir()` in `src/logs.rs`.

---

## CLI reference

    claude-usage-bar             runs the tray app (default)
    claude-usage-bar --once      prints current usage and exits
    claude-usage-bar --diagnose  prints what was found in the logs (support)
    claude-usage-bar --selftest  runs internal asserts, exits 0 on pass
    claude-usage-bar --install   enable auto-start on login
    claude-usage-bar --uninstall disable auto-start

`--once` is the quickest way to confirm the logs are found on a new machine
before dealing with the tray. `--diagnose` reports the log directory, event
counts, detected limit events, and calibration state. If you are on a plan or
locale not yet handled, please open an issue with its output (it contains no
prompt or code content) so we can support more limit-event formats.

## Data source

| OS | Where logs are read from |
|----|--------------------------|
| macOS | `~/.claude/projects/**/*.jsonl` |
| Linux | `~/.claude/projects/**/*.jsonl` |
| Windows | `%USERPROFILE%\.claude\projects\**\*.jsonl` *(unverified, see above)* |

Set `CLAUDE_CONFIG_DIR` to override the `.claude` directory. Files older than
7 days are skipped (they cannot affect either window). If the icon shows
`⚠ logs`, no usable log files were found, confirm Claude Code has run on this
machine.

## Why local logs (and not the usage endpoint)

Earlier versions read Claude Code's OAuth token and called the undocumented
`api.anthropic.com/api/oauth/usage` endpoint. Anthropic's Usage Policy reserves
those Free/Pro/Max OAuth tokens for Claude Code and Claude.ai; using them from
any other tool violates the Consumer Terms, with server-side enforcement active
since January 2026, and put users at risk of an account ban. This version drops
the endpoint and the token entirely and reads the same local logs `ccusage`
uses. See `docs/superpowers/` for the design note.

## How the percentage is learned (not fetched)

A percentage needs a limit: `percent = consumption / limit`. The logs give the
consumption, but Anthropic does not publish the plan limit. So the app **learns**
your limit by watching, passively, for the moment you hit it.

Only the limit (the denominator) is inferred. Everything else is read directly:

| Value | How the app gets it | Precision |
|-------|---------------------|-----------|
| Your consumption (tokens per window) | reads it from the log | **exact** |
| The moment you were throttled | `429` event in the log | **exact** |
| The official reset (`3:20am`) | text of the limit event | **exact** (when an event exists) |
| **The plan limit (the denominator)** | **inferred** from your consumption at that moment | **estimated** |

When you hit your 5h limit, Claude Code logs `429 . resets 3:20am`; the token
count at that instant becomes your learned limit, persisted on disk. After the
first hit the 5h window shows a real **percent** (`5h 78%`); before it, absolute
**tokens** (`5h 1.8M tok`), never a fake percent.

A learned limit is trusted for **7 days** after the hit. A single old sample is
noisy and limits move, so once it goes stale the window reverts to tokens until
your next hit re-calibrates. The percent is therefore most meaningful right after
you have been throttled, when it is freshest.

**It is passive, and that is why it stays within Anthropic's rules.** The app
never probes, never calls anything, never spends quota to test, never touches the
token. It only reads events that already happened while you worked.

**It converges, it does not become exact.** Each hit refines the estimate, but a
residual error remains (model weighting, shared bucket, moving limits), so it is
"good enough", not the exact official number. Full detail and roadmap:
[docs/design/limits-and-calibration.md](docs/design/limits-and-calibration.md).

## Privacy

The app reads your local Claude Code logs, which contain code and prompts, but
extracts **only token counts and limit state**. It makes **no network calls**,
never reads or sends your token or credentials, and opens files read-only.
Nothing leaves your machine. The only file it writes is its own calibration
state at `<data_dir>/claude-usage-bar/calibration.json`.

## Limitations

Plans differ (Free, Pro, Max 5x/20x) and so do the limits, which is why the app
learns yours instead of hardcoding them. The honest caveats:

- **Needs one limit hit to calibrate.** Until you have hit your 5h limit at
  least once (so Claude Code logs it), the 5h window shows absolute tokens, not
  a percent. After that it self-corrects on every new hit.
- **Shared bucket.** Usage on claude.ai and Cowork counts against the same
  limit but is not in the local logs, so the measured consumption is a lower
  bound on the real one.
- **Single-device.** Each machine sees only its own logs; calibration is
  per-machine.
- **Model weighting.** Tokens are summed flat, but Anthropic weights Opus far
  heavier than Sonnet, so the percent is most accurate when your model mix is
  stable. Per-model weighting is a v2 item.
- **Weekly (7d).** The weekly-limit log string is not yet confirmed, so the 7d
  window stays in tokens (uncalibrated) for now.

Full rationale and roadmap: [docs/design/limits-and-calibration.md](docs/design/limits-and-calibration.md).

## Tested against

Claude Code `2.1.x` on macOS (log schema `~/.claude/projects/**/*.jsonl`). Log
formats can change between Claude Code versions; if usage stops updating, run
`--diagnose` and open an issue.
