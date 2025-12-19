mod core;
mod features;
mod ui;

use iced::{window, Size};
use std::env;
use ui::WLaunch;

fn main() -> iced::Result {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    // Check for daemon mode
    if args.len() > 1 && (args[1] == "--daemon" || args[1] == "-d") {
        run_clipboard_daemon();
        return Ok(());
    }

    // Show help
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        print_help();
        return Ok(());
    }

    // Run the GUI launcher
    iced::application("WLaunch", WLaunch::update, WLaunch::view)
        .subscription(WLaunch::subscription)
        .theme(WLaunch::theme)
        .window(window::Settings {
            size: Size::new(800.0, 500.0),
            position: window::Position::Centered,
            resizable: false,
            decorations: false,
            transparent: true,
            level: window::Level::AlwaysOnTop,
            exit_on_close_request: true,
            #[cfg(target_os = "linux")]
            platform_specific: window::settings::PlatformSpecific {
                application_id: "wlaunch".to_string(),
                ..Default::default()
            },
            #[cfg(not(target_os = "linux"))]
            platform_specific: Default::default(),
            ..Default::default()
        })
        .run_with(WLaunch::new)
}

fn print_help() {
    println!("WLaunch - A Raycast-like application launcher for Linux");
    println!();
    println!("USAGE:");
    println!("    wlaunch [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -d, --daemon    Run clipboard monitor daemon in background");
    println!("    -h, --help      Print this help message");
    println!();
    println!("MODES:");
    println!("    (no args)       Launch the GUI application launcher");
    println!("    --daemon        Monitor clipboard and save history");
}

fn run_clipboard_daemon() {
    use arboard::Clipboard;
    use chrono::Local;
    use core::Config;
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::thread;
    use std::time::Duration;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ClipboardEntry {
        content: String,
        timestamp: String,
    }

    fn load_history() -> Vec<ClipboardEntry> {
        let path = Config::data_path("clipboard_history.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&content) {
                    return history;
                }
            }
        }
        Vec::new()
    }

    fn save_history(history: &[ClipboardEntry]) {
        let path = Config::data_path("clipboard_history.json");
        let _ = fs::create_dir_all(path.parent().unwrap());
        if let Ok(content) = serde_json::to_string_pretty(history) {
            let _ = fs::write(path, content);
        }
    }

    println!("WLaunch clipboard daemon started");
    println!("Monitoring clipboard changes...");

    let config = Config::load().unwrap_or_default();
    let max_size = config.clipboard_history_size();

    let mut clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to access clipboard: {}", e);
            return;
        }
    };

    let mut last_content = String::new();
    let mut history = load_history();

    loop {
        if let Ok(content) = clipboard.get_text() {
            if !content.is_empty() && content != last_content {
                last_content = content.clone();

                // Add to history
                let entry = ClipboardEntry {
                    content: content.clone(),
                    timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                };

                // Remove duplicates
                history.retain(|e| e.content != content);
                history.insert(0, entry);

                // Trim to max size
                if history.len() > max_size {
                    history.truncate(max_size);
                }

                save_history(&history);
                log::debug!("Clipboard updated: {}", &content[..content.len().min(50)]);
            }
        }

        thread::sleep(Duration::from_millis(500));
    }
}
