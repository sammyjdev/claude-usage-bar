//! System-tray UI: tao event loop + tray-icon, with a background poll thread.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::client::fetch_usage;
use crate::error::WidgetError;
use crate::render::{self, Level};
use crate::usage::Usage;

/// Events delivered into the tao event loop.
enum UserEvent {
    Menu(MenuEvent),
    Poll(Result<Usage, WidgetError>),
}

const POLL_INTERVAL: Duration = Duration::from_secs(300);

fn icon(level: Level) -> Icon {
    Icon::from_rgba(
        render::icon_rgba(level),
        render::ICON_SIZE as u32,
        render::ICON_SIZE as u32,
    )
    .expect("icon_rgba always yields a valid RGBA buffer")
}

/// Wake the macOS run loop so tray changes draw immediately. No-op elsewhere.
fn wake_macos() {
    #[cfg(target_os = "macos")]
    {
        use objc2_core_foundation::CFRunLoop;
        if let Some(rl) = CFRunLoop::main() {
            rl.wake_up();
        }
    }
}

fn disabled(text: &str) -> MenuItem {
    MenuItem::new(text, false, None)
}

/// Build the dropdown menu. Returns the menu plus the ids of the two
/// actionable items so the event loop can match clicks against them.
fn build_menu(usage: Option<&Usage>, error_note: Option<&str>) -> (Menu, MenuId, MenuId) {
    let menu = Menu::new();
    let now = chrono::Utc::now();

    if let Some(u) = usage {
        let _ = menu.append(&disabled(&render::window_row("Janela de 5h", &u.five_hour)));
        let reset5 = render::reset_row(&u.five_hour, false, now);
        if !reset5.is_empty() {
            let _ = menu.append(&disabled(&reset5));
        }
        let _ = menu.append(&PredefinedMenuItem::separator());
        let _ = menu.append(&disabled(&render::window_row("Semanal (7d)", &u.seven_day)));
        let reset7 = render::reset_row(&u.seven_day, true, now);
        if !reset7.is_empty() {
            let _ = menu.append(&disabled(&reset7));
        }
        let _ = menu.append(&PredefinedMenuItem::separator());

        let sonnet = u
            .seven_day_sonnet
            .as_ref()
            .map(|w| format!("{}%", w.utilization))
            .unwrap_or_else(|| "—".to_string());
        let opus = u
            .seven_day_opus
            .as_ref()
            .map(|w| format!("{}%", w.utilization))
            .unwrap_or_else(|| "—".to_string());
        let _ = menu.append(&disabled(&format!("Semanal · Sonnet   {sonnet}")));
        let _ = menu.append(&disabled(&format!("Semanal · Opus     {opus}")));

        if let Some(ex) = &u.extra_usage {
            if ex.is_enabled {
                let credits = ex
                    .used_credits
                    .map(render::format_credits)
                    .unwrap_or_else(|| "—".to_string());
                let currency = ex.currency.clone().unwrap_or_default();
                let _ = menu.append(&disabled(&format!(
                    "Uso extra          {credits} créditos ({currency})"
                )));
            }
        }
        let _ = menu.append(&PredefinedMenuItem::separator());
    }

    if let Some(note) = error_note {
        let _ = menu.append(&disabled(note));
    }

    let refresh = MenuItem::new("Atualizar agora", true, None);
    let quit = MenuItem::new("Sair", true, None);
    let refresh_id = refresh.id().clone();
    let quit_id = quit.id().clone();
    let _ = menu.append(&refresh);
    let _ = menu.append(&quit);

    (menu, refresh_id, quit_id)
}

/// Update the tray from a poll result.
fn apply(
    tray: &Option<TrayIcon>,
    last_usage: &mut Option<Usage>,
    refresh_id: &mut Option<MenuId>,
    quit_id: &mut Option<MenuId>,
    result: Result<Usage, WidgetError>,
) {
    let Some(tray) = tray else { return };

    match result {
        Ok(u) => {
            let level = render::worst_level(u.five_hour.utilization, u.seven_day.utilization);
            let _ = tray.set_icon(Some(icon(level)));
            let _ = tray.set_tooltip(Some(render::tooltip_text(&u)));
            tray.set_title(Some(render::title_text(&u)));
            let (menu, r, q) = build_menu(Some(&u), None);
            tray.set_menu(Some(Box::new(menu)));
            *refresh_id = Some(r);
            *quit_id = Some(q);
            *last_usage = Some(u);
        }
        Err(e) => {
            let (note, title) = render::error_text(&e);
            let keep_last = matches!(e, WidgetError::Network(_)) && last_usage.is_some();
            if keep_last {
                // Transient network failure: keep the last good data, just flag it.
                let _ = tray.set_tooltip(Some(format!("⚠ {note}")));
                let (menu, r, q) = build_menu(last_usage.as_ref(), Some(&note));
                tray.set_menu(Some(Box::new(menu)));
                *refresh_id = Some(r);
                *quit_id = Some(q);
            } else {
                let _ = tray.set_icon(Some(icon(Level::Grey)));
                let _ = tray.set_tooltip(Some(note.clone()));
                tray.set_title(Some(title));
                let (menu, r, q) = build_menu(last_usage.as_ref(), Some(&note));
                tray.set_menu(Some(Box::new(menu)));
                *refresh_id = Some(r);
                *quit_id = Some(q);
            }
        }
    }
}

pub fn run() -> ! {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();

    // Forward menu events into the event loop.
    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::Menu(event));
    }));

    // Background poll thread: fetch, push result, then wait 5 min OR until a
    // manual-refresh signal arrives on `refresh_rx`.
    let (refresh_tx, refresh_rx) = mpsc::channel::<()>();
    let poll_proxy = event_loop.create_proxy();
    thread::spawn(move || loop {
        let result = fetch_usage();
        if poll_proxy.send_event(UserEvent::Poll(result)).is_err() {
            return; // event loop has shut down
        }
        match refresh_rx.recv_timeout(POLL_INTERVAL) {
            Ok(()) | Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => return,
        }
    });

    let mut tray: Option<TrayIcon> = None;
    let mut last_usage: Option<Usage> = None;
    let mut refresh_id: Option<MenuId> = None;
    let mut quit_id: Option<MenuId> = None;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => {
                let (menu, r_id, q_id) = build_menu(None, Some("carregando…"));
                refresh_id = Some(r_id);
                quit_id = Some(q_id);
                tray = Some(
                    TrayIconBuilder::new()
                        .with_menu(Box::new(menu))
                        .with_tooltip("Claude — carregando…")
                        .with_icon(icon(Level::Grey))
                        .build()
                        .expect("failed to build tray icon"),
                );
                wake_macos();
            }
            Event::UserEvent(UserEvent::Poll(result)) => {
                apply(&tray, &mut last_usage, &mut refresh_id, &mut quit_id, result);
                wake_macos();
            }
            Event::UserEvent(UserEvent::Menu(ev)) => {
                if Some(&ev.id) == quit_id.as_ref() {
                    tray.take();
                    *control_flow = ControlFlow::Exit;
                } else if Some(&ev.id) == refresh_id.as_ref() {
                    let _ = refresh_tx.send(());
                }
            }
            _ => {}
        }
    })
}
