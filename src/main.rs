mod autostart;
mod client;
mod error;
mod render;
mod token;
mod tray;
mod usage;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--once") => run_once(),
        Some("--selftest") => run_selftest(),
        Some("--install") => autostart::install(),
        Some("--uninstall") => autostart::uninstall(),
        _ => tray::run(),
    }
}

fn run_once() {
    match client::fetch_usage() {
        Ok(u) => {
            println!("5h: {}%", u.five_hour.utilization);
            println!("7d: {}%", u.seven_day.utilization);
            println!(
                "7d sonnet: {}",
                u.seven_day_sonnet
                    .map(|w| format!("{}%", w.utilization))
                    .unwrap_or_else(|| "—".to_string())
            );
            println!(
                "7d opus:   {}",
                u.seven_day_opus
                    .map(|w| format!("{}%", w.utilization))
                    .unwrap_or_else(|| "—".to_string())
            );
            if let Some(ex) = u.extra_usage {
                println!(
                    "extra: {} {}",
                    ex.used_credits.map(render::format_credits).unwrap_or_default(),
                    ex.currency.unwrap_or_default()
                );
            }
        }
        Err(e) => {
            eprintln!("error: {e:?}");
            std::process::exit(1);
        }
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
