#![allow(dead_code)] // removed in the final task once every module is wired

mod error;

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--once") => println!("once: not implemented"),
        Some("--selftest") => println!("selftest: not implemented"),
        Some("--install") => println!("install: not implemented"),
        Some("--uninstall") => println!("uninstall: not implemented"),
        _ => println!("tray: not implemented"),
    }
}
