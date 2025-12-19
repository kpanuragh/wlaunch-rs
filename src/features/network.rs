use crate::core::{Item, ItemType};
use std::process::Command;

pub struct NetworkManager;

impl NetworkManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query = query.to_lowercase();
        let mut items = Vec::new();

        // Add actions
        items.push(
            Item::new("wifi:scan", "Scan Networks", ItemType::WifiAction)
                .with_description("Scan for available WiFi networks")
                .with_icon("network-wireless"),
        );
        items.push(
            Item::new("wifi:toggle", "Toggle WiFi", ItemType::WifiAction)
                .with_description("Enable or disable WiFi")
                .with_icon("network-wireless"),
        );
        items.push(
            Item::new("wifi:disconnect", "Disconnect", ItemType::WifiAction)
                .with_description("Disconnect from current network")
                .with_icon("network-wireless-disconnected"),
        );

        // Get available networks
        if let Ok(output) = Command::new("nmcli")
            .args(["-t", "-f", "SSID,SIGNAL,SECURITY,IN-USE", "device", "wifi", "list"])
            .output()
        {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split(':').collect();
                        if parts.len() >= 4 {
                            let ssid = parts[0];
                            if ssid.is_empty() {
                                continue;
                            }

                            let signal: i32 = parts[1].parse().unwrap_or(0);
                            let security = parts[2];
                            let in_use = parts[3] == "*";

                            let mut item = Item::new(
                                format!("wifi:{}", ssid),
                                ssid,
                                ItemType::WifiNetwork,
                            )
                            .with_description(format!(
                                "Signal: {}% | {}{}",
                                signal,
                                security,
                                if in_use { " (Connected)" } else { "" }
                            ))
                            .with_icon(if signal > 75 {
                                "network-wireless-signal-excellent"
                            } else if signal > 50 {
                                "network-wireless-signal-good"
                            } else if signal > 25 {
                                "network-wireless-signal-ok"
                            } else {
                                "network-wireless-signal-weak"
                            });

                            item.metadata.ssid = Some(ssid.to_string());
                            item.metadata.signal_strength = Some(signal);
                            item.metadata.secured = !security.is_empty() && security != "--";
                            item.metadata.connected = in_use;

                            items.push(item);
                        }
                    }
                }
            }
        }

        // Filter by query
        if !query.is_empty() {
            items.retain(|item| {
                item.name.to_lowercase().contains(&query)
                    || item
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query))
                        .unwrap_or(false)
            });
        }

        items
    }

    pub fn connect(&self, ssid: &str) {
        let _ = Command::new("nmcli")
            .args(["device", "wifi", "connect", ssid])
            .output();
    }

    pub fn disconnect(&self) {
        let _ = Command::new("nmcli")
            .args(["device", "disconnect", "wlan0"])
            .output();
    }

    pub fn toggle_wifi(&self) {
        // Check current state
        if let Ok(output) = Command::new("nmcli")
            .args(["radio", "wifi"])
            .output()
        {
            if output.status.success() {
                let state = String::from_utf8_lossy(&output.stdout);
                if state.trim() == "enabled" {
                    let _ = Command::new("nmcli")
                        .args(["radio", "wifi", "off"])
                        .output();
                } else {
                    let _ = Command::new("nmcli")
                        .args(["radio", "wifi", "on"])
                        .output();
                }
            }
        }
    }

    pub fn scan(&self) {
        let _ = Command::new("nmcli")
            .args(["device", "wifi", "rescan"])
            .output();
    }

    pub fn execute_action(&self, action_id: &str) {
        match action_id {
            "wifi:scan" => self.scan(),
            "wifi:toggle" => self.toggle_wifi(),
            "wifi:disconnect" => self.disconnect(),
            _ => {}
        }
    }
}

impl Default for NetworkManager {
    fn default() -> Self {
        Self::new()
    }
}
