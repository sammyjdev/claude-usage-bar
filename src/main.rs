mod autostart;
mod calibration;
mod error;
mod logs;
mod render;
mod tray;
mod usage;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--once") => run_once(),
        Some("--diagnose") => run_diagnose(),
        Some("--selftest") => run_selftest(),
        Some("--install") => autostart::install(),
        Some("--uninstall") => autostart::uninstall(),
        _ => tray::run(),
    }
}

fn run_once() {
    match logs::collect(chrono::Utc::now()) {
        Ok(u) => {
            println!("5h: {}", render::window_value(&u.five_hour));
            println!("7d: {}", render::window_value(&u.seven_day));
            let tok = |w: &usage::Window| format!("{} tok", render::format_tokens(w.tokens));
            println!(
                "7d opus:   {}",
                u.seven_day_opus.as_ref().map(tok).unwrap_or_else(|| "-".to_string())
            );
            println!(
                "7d sonnet: {}",
                u.seven_day_sonnet.as_ref().map(tok).unwrap_or_else(|| "-".to_string())
            );
        }
        Err(e) => {
            eprintln!("error: {e:?}");
            std::process::exit(1);
        }
    }
}

/// Report what was found in the logs, for support and for contributors sending
/// limit-event samples from other plans, locales, and Claude Code versions.
fn run_diagnose() {
    let d = logs::diagnose(chrono::Utc::now());
    println!("logs dir:      {}", d.dir.as_deref().unwrap_or("NOT FOUND"));
    println!("files (<=7d):  {}", d.files);
    println!("usage events:  {}", d.usage_events);
    println!("malformed:     {}", d.malformed);
    println!("limit events:  {}", d.limit_events.len());
    for (kind, ts, reset) in &d.limit_events {
        println!("  - {ts}  {kind}  resets {}", reset.as_deref().unwrap_or("?"));
    }
    match d.calibration.five_hour_limit {
        Some(limit) => println!(
            "5h calibration: {} tok (learned {})",
            render::format_tokens(limit),
            d.calibration.five_hour_updated.as_deref().unwrap_or("?")
        ),
        None => println!("5h calibration: not calibrated (no session-limit hit seen yet)"),
    }
}

fn run_selftest() {
    use render::Level;
    assert_eq!(render::level_for(49), Level::Green);
    assert_eq!(render::level_for(80), Level::Red);
    assert_eq!(render::ascii_bar(100), "▓▓▓▓▓▓▓▓▓▓");
    assert_eq!(
        render::icon_rgba(Level::Green).len(),
        render::ICON_SIZE * render::ICON_SIZE * 4
    );
    println!("selftest: PASS");
}
