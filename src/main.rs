mod core;
mod features;
mod ui;

use iced::{window, Size};
use ui::WLaunch;

fn main() -> iced::Result {
    env_logger::init();

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
