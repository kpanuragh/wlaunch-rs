//! Window manager integration for listing and focusing windows.
//!
//! Supports multiple window managers:
//! - i3/Sway (via `i3-msg`)
//! - Hyprland (via `hyprctl`)
//! - X11 WMs like GNOME, KDE, XFCE (via `wmctrl`)

use crate::core::{Item, ItemType};
use serde::Deserialize;
use std::process::Command;

/// Supported window manager types.
#[derive(Debug, Clone, PartialEq)]
enum WMType {
    /// i3 or Sway window manager
    I3Sway,
    /// Hyprland compositor
    Hyprland,
    /// Generic X11 window managers via wmctrl
    X11Wmctrl,
    /// No supported window manager detected
    Unknown,
}

/// i3/Sway tree node structure for JSON deserialization.
#[derive(Debug, Deserialize)]
struct I3Node {
    id: i64,
    name: Option<String>,
    #[serde(rename = "type")]
    node_type: String,
    #[allow(dead_code)]
    focused: bool,
    #[serde(default)]
    nodes: Vec<I3Node>,
    #[serde(default)]
    floating_nodes: Vec<I3Node>,
    window_properties: Option<I3WindowProperties>,
    #[allow(dead_code)]
    #[serde(default)]
    num: Option<i32>,
}

/// i3/Sway window properties.
#[derive(Debug, Deserialize)]
struct I3WindowProperties {
    class: Option<String>,
    #[allow(dead_code)]
    instance: Option<String>,
    title: Option<String>,
}

/// Hyprland client (window) structure for JSON deserialization.
#[derive(Debug, Deserialize)]
struct HyprlandClient {
    address: String,
    title: String,
    class: String,
    workspace: HyprlandWorkspace,
}

/// Hyprland workspace information.
#[derive(Debug, Deserialize)]
struct HyprlandWorkspace {
    name: String,
}

/// Manager for window listing and manipulation across different window managers.
///
/// Automatically detects the running window manager at initialization and uses
/// the appropriate backend for window operations.
pub struct WindowsManager {
    wm_type: WMType,
}

impl WindowsManager {
    /// Creates a new WindowsManager, auto-detecting the current window manager.
    ///
    /// Detection order:
    /// 1. Hyprland (checks `HYPRLAND_INSTANCE_SIGNATURE` env var and `hyprctl`)
    /// 2. i3/Sway (checks `i3-msg -t get_version`)
    /// 3. wmctrl (fallback for X11 window managers)
    pub fn new() -> Self {
        let wm_type = Self::detect_wm();
        log::debug!("Detected window manager: {:?}", wm_type);
        Self { wm_type }
    }

    fn detect_wm() -> WMType {
        // Check for Hyprland first (via HYPRLAND_INSTANCE_SIGNATURE env var)
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            if let Ok(output) = Command::new("hyprctl").arg("version").output() {
                if output.status.success() {
                    return WMType::Hyprland;
                }
            }
        }

        // Check for i3/Sway
        if let Ok(output) = Command::new("i3-msg").args(["-t", "get_version"]).output() {
            if output.status.success() {
                return WMType::I3Sway;
            }
        }

        // Check for wmctrl (works with most X11 WMs)
        if let Ok(output) = Command::new("wmctrl").arg("--version").output() {
            if output.status.success() {
                return WMType::X11Wmctrl;
            }
        }

        WMType::Unknown
    }

    /// Returns a list of open windows matching the query.
    ///
    /// # Arguments
    /// * `query` - Filter string to match against window titles and classes
    ///
    /// # Returns
    /// Vector of `Item` representing each matching window
    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query = query.to_lowercase();
        let mut items = match self.wm_type {
            WMType::I3Sway => self.get_i3_windows(),
            WMType::Hyprland => self.get_hyprland_windows(),
            WMType::X11Wmctrl => self.get_wmctrl_windows(),
            WMType::Unknown => {
                // Return a helpful message
                vec![Item::new(
                    "window:no-wm",
                    "No supported window manager detected",
                    ItemType::Window,
                )
                .with_description("Install wmctrl, or use i3/Sway/Hyprland")]
            }
        };

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

    // ==================== i3/Sway ====================
    fn get_i3_windows(&self) -> Vec<Item> {
        let mut items = Vec::new();

        if let Ok(output) = Command::new("i3-msg").args(["-t", "get_tree"]).output() {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(tree) = serde_json::from_str::<I3Node>(&stdout) {
                        self.collect_i3_windows(&tree, &mut items, None);
                    }
                }
            }
        }

        items
    }

    fn collect_i3_windows(&self, node: &I3Node, items: &mut Vec<Item>, workspace: Option<&str>) {
        let current_workspace = if node.node_type == "workspace" {
            node.name.as_deref()
        } else {
            workspace
        };

        if node.node_type == "con" && node.window_properties.is_some() {
            if let Some(props) = &node.window_properties {
                let title = props
                    .title
                    .as_deref()
                    .or(node.name.as_deref())
                    .unwrap_or("Unknown");
                let class = props.class.as_deref().unwrap_or("Unknown");

                let mut item = Item::new(
                    format!("window:{}", node.id),
                    title,
                    ItemType::Window,
                )
                .with_description(format!("{} ({})", class, current_workspace.unwrap_or("?")))
                .with_icon("window");

                item.metadata.window_id = Some(node.id);
                item.metadata.workspace = current_workspace.map(String::from);

                items.push(item);
            }
        }

        for child in &node.nodes {
            self.collect_i3_windows(child, items, current_workspace);
        }
        for child in &node.floating_nodes {
            self.collect_i3_windows(child, items, current_workspace);
        }
    }

    // ==================== Hyprland ====================
    fn get_hyprland_windows(&self) -> Vec<Item> {
        let mut items = Vec::new();

        if let Ok(output) = Command::new("hyprctl").args(["clients", "-j"]).output() {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(clients) = serde_json::from_str::<Vec<HyprlandClient>>(&stdout) {
                        for client in clients {
                            let mut item = Item::new(
                                format!("window:{}", client.address),
                                &client.title,
                                ItemType::Window,
                            )
                            .with_description(format!(
                                "{} ({})",
                                client.class, client.workspace.name
                            ))
                            .with_icon("window");

                            // Store address as string in metadata for Hyprland
                            // We'll parse it back when focusing
                            item.metadata.workspace = Some(client.workspace.name);
                            // Convert hex address to i64 for window_id
                            if let Some(addr) = client.address.strip_prefix("0x") {
                                if let Ok(id) = i64::from_str_radix(addr, 16) {
                                    item.metadata.window_id = Some(id);
                                }
                            }

                            items.push(item);
                        }
                    }
                }
            }
        }

        items
    }

    // ==================== wmctrl (X11) ====================
    fn get_wmctrl_windows(&self) -> Vec<Item> {
        let mut items = Vec::new();

        // wmctrl -l -x output format:
        // 0x04000003  0 instance.class  hostname Window Title
        if let Ok(output) = Command::new("wmctrl").args(["-l", "-x"]).output() {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.splitn(5, ' ').filter(|s| !s.is_empty()).collect();
                        if parts.len() >= 5 {
                            let window_id_hex = parts[0];
                            let desktop = parts[1];
                            let class = parts[2];
                            // parts[3] is hostname, parts[4..] is title
                            let title = parts[4..].join(" ");
                            let title = if title.is_empty() {
                                class.to_string()
                            } else {
                                title
                            };

                            // Parse window ID from hex
                            let window_id = if let Some(hex) = window_id_hex.strip_prefix("0x") {
                                i64::from_str_radix(hex, 16).unwrap_or(0)
                            } else {
                                0
                            };

                            if window_id == 0 {
                                continue;
                            }

                            // Parse class (format: instance.class)
                            let class_name = class.split('.').last().unwrap_or(class);

                            let workspace = if desktop == "-1" {
                                "sticky".to_string()
                            } else {
                                format!("Desktop {}", desktop)
                            };

                            let mut item = Item::new(
                                format!("window:{}", window_id),
                                &title,
                                ItemType::Window,
                            )
                            .with_description(format!("{} ({})", class_name, workspace))
                            .with_icon("window");

                            item.metadata.window_id = Some(window_id);
                            item.metadata.workspace = Some(workspace);

                            items.push(item);
                        }
                    }
                }
            }
        }

        items
    }

    /// Focuses a window by its ID.
    ///
    /// # Arguments
    /// * `window_id` - The window ID to focus (format depends on WM)
    ///
    /// Uses the appropriate command for the detected window manager:
    /// - i3/Sway: `i3-msg [con_id=<id>] focus`
    /// - Hyprland: `hyprctl dispatch focuswindow address:<addr>`
    /// - wmctrl: `wmctrl -i -a <id>`
    pub fn focus_window(&self, window_id: i64) {
        let result = match self.wm_type {
            WMType::I3Sway => {
                Command::new("i3-msg")
                    .arg(format!("[con_id={}] focus", window_id))
                    .output()
            }
            WMType::Hyprland => {
                let address = format!("0x{:x}", window_id);
                Command::new("hyprctl")
                    .args(["dispatch", "focuswindow", &format!("address:{}", address)])
                    .output()
            }
            WMType::X11Wmctrl => {
                Command::new("wmctrl")
                    .args(["-i", "-a", &format!("0x{:08x}", window_id)])
                    .output()
            }
            WMType::Unknown => return,
        };

        if let Err(e) = result {
            log::debug!("Failed to focus window {}: {}", window_id, e);
        } else if let Ok(output) = result {
            if !output.status.success() {
                log::debug!(
                    "Focus window command failed for {}: {}",
                    window_id,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    /// Closes a window by its ID.
    ///
    /// # Arguments
    /// * `window_id` - The window ID to close (format depends on WM)
    ///
    /// Uses the appropriate command for the detected window manager:
    /// - i3/Sway: `i3-msg [con_id=<id>] kill`
    /// - Hyprland: `hyprctl dispatch closewindow address:<addr>`
    /// - wmctrl: `wmctrl -i -c <id>`
    #[allow(dead_code)]
    pub fn close_window(&self, window_id: i64) {
        let result = match self.wm_type {
            WMType::I3Sway => {
                Command::new("i3-msg")
                    .arg(format!("[con_id={}] kill", window_id))
                    .output()
            }
            WMType::Hyprland => {
                let address = format!("0x{:x}", window_id);
                Command::new("hyprctl")
                    .args(["dispatch", "closewindow", &format!("address:{}", address)])
                    .output()
            }
            WMType::X11Wmctrl => {
                Command::new("wmctrl")
                    .args(["-i", "-c", &format!("0x{:08x}", window_id)])
                    .output()
            }
            WMType::Unknown => return,
        };

        if let Err(e) = result {
            log::debug!("Failed to close window {}: {}", window_id, e);
        } else if let Ok(output) = result {
            if !output.status.success() {
                log::debug!(
                    "Close window command failed for {}: {}",
                    window_id,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

impl Default for WindowsManager {
    fn default() -> Self {
        Self::new()
    }
}
