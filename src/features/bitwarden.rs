use crate::core::{Config, Item, ItemType};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct BwItem {
    id: String,
    name: String,
    login: Option<BwLogin>,
    notes: Option<String>,
    #[serde(rename = "type")]
    item_type: i32,
}

#[derive(Debug, Deserialize)]
struct BwLogin {
    username: Option<String>,
    password: Option<String>,
    totp: Option<String>,
    uris: Option<Vec<BwUri>>,
}

#[derive(Debug, Deserialize)]
struct BwUri {
    uri: Option<String>,
}

pub struct BitwardenManager {
    session: Option<String>,
    server: Option<String>,
}

impl BitwardenManager {
    pub fn new(config: &Config) -> Self {
        Self {
            session: None,
            server: config.bitwarden_server.clone(),
        }
    }

    fn is_bw_installed(&self) -> bool {
        Command::new("bw").arg("--version").output().is_ok()
    }

    fn get_status(&self) -> String {
        if let Ok(output) = Command::new("bw").args(["status"]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("\"status\":\"unlocked\"") {
                    return "unlocked".to_string();
                } else if stdout.contains("\"status\":\"locked\"") {
                    return "locked".to_string();
                } else if stdout.contains("\"status\":\"unauthenticated\"") {
                    return "unauthenticated".to_string();
                }
            }
        }
        "unknown".to_string()
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let mut items = Vec::new();

        if !self.is_bw_installed() {
            items.push(
                Item::new(
                    "bw:not_installed",
                    "Bitwarden CLI not installed",
                    ItemType::BitwardenAction,
                )
                .with_description("Install with: npm install -g @bitwarden/cli")
                .with_icon("dialog-warning"),
            );
            return items;
        }

        let status = self.get_status();

        match status.as_str() {
            "unauthenticated" => {
                items.push(
                    Item::new("bw:login", "Login to Bitwarden", ItemType::BitwardenAction)
                        .with_description("Run: bw login")
                        .with_icon("dialog-password"),
                );
            }
            "locked" => {
                items.push(
                    Item::new("bw:unlock", "Unlock Vault", ItemType::BitwardenAction)
                        .with_description("Enter master password to unlock")
                        .with_icon("dialog-password"),
                );
            }
            "unlocked" => {
                // Show vault items
                items.extend(self.search_vault(query));

                // Add actions
                if query.is_empty() {
                    items.push(
                        Item::new("bw:lock", "Lock Vault", ItemType::BitwardenAction)
                            .with_description("Lock the Bitwarden vault")
                            .with_icon("system-lock-screen"),
                    );
                    items.push(
                        Item::new("bw:sync", "Sync Vault", ItemType::BitwardenAction)
                            .with_description("Sync with Bitwarden server")
                            .with_icon("view-refresh"),
                    );
                    items.push(
                        Item::new("bw:generate", "Generate Password", ItemType::BitwardenAction)
                            .with_description("Generate a new secure password")
                            .with_icon("dialog-password"),
                    );
                }
            }
            _ => {
                items.push(
                    Item::new("bw:status", "Check Bitwarden Status", ItemType::BitwardenAction)
                        .with_description("Unable to determine vault status")
                        .with_icon("dialog-question"),
                );
            }
        }

        items
    }

    fn search_vault(&self, query: &str) -> Vec<Item> {
        let mut items = Vec::new();

        let args = if query.is_empty() {
            vec!["list", "items"]
        } else {
            vec!["list", "items", "--search", query]
        };

        if let Ok(output) = Command::new("bw").args(&args).output() {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(bw_items) = serde_json::from_str::<Vec<BwItem>>(&stdout) {
                        for bw_item in bw_items.iter().take(20) {
                            let type_str = match bw_item.item_type {
                                1 => "Login",
                                2 => "Secure Note",
                                3 => "Card",
                                4 => "Identity",
                                _ => "Item",
                            };

                            let description = if let Some(login) = &bw_item.login {
                                login.username.clone().unwrap_or_default()
                            } else {
                                type_str.to_string()
                            };

                            let mut item = Item::new(
                                format!("bw:item:{}", bw_item.id),
                                &bw_item.name,
                                ItemType::BitwardenItem,
                            )
                            .with_description(description)
                            .with_icon("dialog-password");

                            if let Some(login) = &bw_item.login {
                                item.metadata.username = login.username.clone();
                                item.metadata.password = login.password.clone();
                                item.metadata.totp = login.totp.clone();
                                if let Some(uris) = &login.uris {
                                    if let Some(first_uri) = uris.first() {
                                        item.metadata.uri = first_uri.uri.clone();
                                    }
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

    pub fn lock(&self) {
        let _ = Command::new("bw").args(["lock"]).output();
    }

    pub fn sync(&self) {
        let _ = Command::new("bw").args(["sync"]).output();
    }

    pub fn generate_password(&self) -> Option<String> {
        if let Ok(output) = Command::new("bw")
            .args(["generate", "-ulns", "--length", "20"])
            .output()
        {
            if output.status.success() {
                return String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string());
            }
        }
        None
    }

    pub fn get_totp(&self, item_id: &str) -> Option<String> {
        if let Ok(output) = Command::new("bw")
            .args(["get", "totp", item_id])
            .output()
        {
            if output.status.success() {
                return String::from_utf8(output.stdout).ok().map(|s| s.trim().to_string());
            }
        }
        None
    }

    pub fn execute_action(&self, action_id: &str) {
        match action_id {
            "bw:lock" => self.lock(),
            "bw:sync" => self.sync(),
            "bw:generate" => {
                if let Some(password) = self.generate_password() {
                    // Copy to clipboard
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&password);
                    }
                    let _ = notify_rust::Notification::new()
                        .summary("Password Generated")
                        .body("New password copied to clipboard")
                        .show();
                }
            }
            id if id.starts_with("bw:totp:") => {
                let item_id = id.strip_prefix("bw:totp:").unwrap();
                if let Some(totp) = self.get_totp(item_id) {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&totp);
                    }
                    let _ = notify_rust::Notification::new()
                        .summary("TOTP Copied")
                        .body(&format!("Code: {}", totp))
                        .show();
                }
            }
            _ => {}
        }
    }
}

impl Clone for BitwardenManager {
    fn clone(&self) -> Self {
        Self {
            session: self.session.clone(),
            server: self.server.clone(),
        }
    }
}

impl Default for BitwardenManager {
    fn default() -> Self {
        Self::new(&Config::default())
    }
}
