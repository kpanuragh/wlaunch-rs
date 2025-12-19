use crate::core::{Item, ItemType};
use std::process::Command;

pub struct BluetoothManager;

impl BluetoothManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query = query.to_lowercase();
        let mut items = Vec::new();

        // Add actions
        items.push(
            Item::new("bt:scan", "Scan Devices", ItemType::BluetoothAction)
                .with_description("Scan for Bluetooth devices")
                .with_icon("bluetooth"),
        );
        items.push(
            Item::new("bt:toggle", "Toggle Bluetooth", ItemType::BluetoothAction)
                .with_description("Enable or disable Bluetooth")
                .with_icon("bluetooth"),
        );

        // Get paired devices
        if let Ok(output) = Command::new("bluetoothctl")
            .args(["devices", "Paired"])
            .output()
        {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        // Format: Device XX:XX:XX:XX:XX:XX Name
                        if let Some(rest) = line.strip_prefix("Device ") {
                            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
                            if parts.len() >= 2 {
                                let mac = parts[0];
                                let name = parts[1];

                                let connected = self.is_connected(mac);

                                let mut item = Item::new(
                                    format!("bt:{}", mac),
                                    name,
                                    ItemType::BluetoothDevice,
                                )
                                .with_description(format!(
                                    "{} | {}",
                                    mac,
                                    if connected { "Connected" } else { "Paired" }
                                ))
                                .with_icon(if connected {
                                    "bluetooth-active"
                                } else {
                                    "bluetooth"
                                });

                                item.metadata.mac_address = Some(mac.to_string());
                                item.metadata.paired = true;
                                item.metadata.connected = connected;

                                items.push(item);
                            }
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

    fn is_connected(&self, mac: &str) -> bool {
        if let Ok(output) = Command::new("bluetoothctl")
            .args(["info", mac])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout.contains("Connected: yes");
            }
        }
        false
    }

    pub fn connect(&self, mac: &str) {
        let _ = Command::new("bluetoothctl")
            .args(["connect", mac])
            .output();
    }

    pub fn disconnect(&self, mac: &str) {
        let _ = Command::new("bluetoothctl")
            .args(["disconnect", mac])
            .output();
    }

    pub fn pair(&self, mac: &str) {
        let _ = Command::new("bluetoothctl")
            .args(["pair", mac])
            .output();
    }

    pub fn toggle_power(&self) {
        if let Ok(output) = Command::new("bluetoothctl")
            .args(["show"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("Powered: yes") {
                    let _ = Command::new("bluetoothctl")
                        .args(["power", "off"])
                        .output();
                } else {
                    let _ = Command::new("bluetoothctl")
                        .args(["power", "on"])
                        .output();
                }
            }
        }
    }

    pub fn scan_start(&self) {
        let _ = Command::new("bluetoothctl")
            .args(["scan", "on"])
            .output();
    }

    pub fn execute_action(&self, action_id: &str) {
        match action_id {
            "bt:scan" => self.scan_start(),
            "bt:toggle" => self.toggle_power(),
            id if id.starts_with("bt:") => {
                let mac = id.strip_prefix("bt:").unwrap();
                if self.is_connected(mac) {
                    self.disconnect(mac);
                } else {
                    self.connect(mac);
                }
            }
            _ => {}
        }
    }
}

impl Default for BluetoothManager {
    fn default() -> Self {
        Self::new()
    }
}
