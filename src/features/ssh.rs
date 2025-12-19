use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SshConnection {
    id: String,
    name: String,
    host: String,
    user: String,
    port: u16,
    identity_file: Option<String>,
}

pub struct SshManager {
    connections: Vec<SshConnection>,
}

impl SshManager {
    pub fn new() -> Self {
        let mut connections = Self::load_saved().unwrap_or_default();
        connections.extend(Self::parse_ssh_config().unwrap_or_default());
        Self { connections }
    }

    fn data_path() -> PathBuf {
        Config::data_path("ssh_connections.json")
    }

    fn load_saved() -> Result<Vec<SshConnection>> {
        let path = Self::data_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let connections: Vec<SshConnection> = serde_json::from_str(&content)?;
            Ok(connections)
        } else {
            Ok(Vec::new())
        }
    }

    fn save(&self) -> Result<()> {
        // Only save custom connections, not ones from ssh config
        let custom: Vec<_> = self
            .connections
            .iter()
            .filter(|c| !c.id.starts_with("sshconfig:"))
            .cloned()
            .collect();

        let path = Self::data_path();
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(&custom)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn parse_ssh_config() -> Result<Vec<SshConnection>> {
        let ssh_config_path = dirs::home_dir()
            .map(|h| h.join(".ssh/config"))
            .unwrap_or_default();

        if !ssh_config_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(ssh_config_path)?;
        let mut connections = Vec::new();
        let mut current: Option<SshConnection> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, |c| c == ' ' || c == '=').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].to_lowercase();
            let value = parts[1].trim().trim_matches('"');

            match key.as_str() {
                "host" => {
                    if let Some(conn) = current.take() {
                        if !conn.host.is_empty() && conn.name != "*" {
                            connections.push(conn);
                        }
                    }
                    current = Some(SshConnection {
                        id: format!("sshconfig:{}", value),
                        name: value.to_string(),
                        host: String::new(),
                        user: "root".to_string(),
                        port: 22,
                        identity_file: None,
                    });
                }
                "hostname" => {
                    if let Some(ref mut conn) = current {
                        conn.host = value.to_string();
                    }
                }
                "user" => {
                    if let Some(ref mut conn) = current {
                        conn.user = value.to_string();
                    }
                }
                "port" => {
                    if let Some(ref mut conn) = current {
                        conn.port = value.parse().unwrap_or(22);
                    }
                }
                "identityfile" => {
                    if let Some(ref mut conn) = current {
                        conn.identity_file = Some(value.to_string());
                    }
                }
                _ => {}
            }
        }

        // Add the last connection
        if let Some(conn) = current {
            if !conn.host.is_empty() && conn.name != "*" {
                connections.push(conn);
            }
        }

        Ok(connections)
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query_lower = query.to_lowercase();
        let mut items = Vec::new();

        // Add action to add new connection
        if query.starts_with("add ") {
            let rest = query.splitn(2, ' ').nth(1).unwrap_or("");
            // Parse "user@host:port" format
            if !rest.is_empty() {
                let (user, host_port): (&str, &str) = if rest.contains('@') {
                    let parts: Vec<&str> = rest.splitn(2, '@').collect();
                    (parts[0], parts.get(1).copied().unwrap_or(""))
                } else {
                    ("root", rest)
                };

                let (host, port): (&str, u16) = if host_port.contains(':') {
                    let parts: Vec<&str> = host_port.splitn(2, ':').collect();
                    (parts[0], parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(22))
                } else {
                    (host_port, 22u16)
                };

                if !host.is_empty() {
                    items.push(
                        Item::new(
                            format!("ssh:add:{}:{}:{}", user, host, port),
                            format!("Add SSH: {}@{}:{}", user, host, port),
                            ItemType::SshAction,
                        )
                        .with_description("Save new SSH connection")
                        .with_icon("network-server"),
                    );
                }
            }
        }

        // List existing connections
        for conn in &self.connections {
            if query_lower.is_empty()
                || conn.name.to_lowercase().contains(&query_lower)
                || conn.host.to_lowercase().contains(&query_lower)
            {
                let source = if conn.id.starts_with("sshconfig:") {
                    " (from ~/.ssh/config)"
                } else {
                    ""
                };

                let mut item = Item::new(
                    format!("ssh:{}", conn.id),
                    &conn.name,
                    ItemType::SshConnection,
                )
                .with_description(format!(
                    "{}@{}:{}{}",
                    conn.user, conn.host, conn.port, source
                ))
                .with_icon("network-server");

                item.metadata.host = Some(conn.host.clone());
                item.metadata.user = Some(conn.user.clone());
                item.metadata.port = Some(conn.port);

                items.push(item);
            }
        }

        items
    }

    pub fn add_connection(&mut self, user: &str, host: &str, port: u16) {
        let conn = SshConnection {
            id: Uuid::new_v4().to_string(),
            name: format!("{}@{}", user, host),
            host: host.to_string(),
            user: user.to_string(),
            port,
            identity_file: None,
        };

        self.connections.push(conn);
        let _ = self.save();
    }

    pub fn delete_connection(&mut self, id: &str) {
        self.connections.retain(|c| c.id != id);
        let _ = self.save();
    }

    pub fn execute_action(&mut self, action_id: &str, query: &str) {
        if action_id.starts_with("ssh:add:") {
            let rest = action_id.strip_prefix("ssh:add:").unwrap();
            let parts: Vec<&str> = rest.splitn(3, ':').collect();
            if parts.len() == 3 {
                let user = parts[0];
                let host = parts[1];
                let port: u16 = parts[2].parse().unwrap_or(22);
                self.add_connection(user, host, port);
            }
        } else if action_id.starts_with("ssh:delete:") {
            let id = action_id.strip_prefix("ssh:delete:").unwrap();
            self.delete_connection(id);
        } else if query.starts_with("add ") {
            // Parse and add from query
            let rest = query.splitn(2, ' ').nth(1).unwrap_or("");
            let (user, host_port): (&str, &str) = if rest.contains('@') {
                let parts: Vec<&str> = rest.splitn(2, '@').collect();
                (parts[0], parts.get(1).copied().unwrap_or(""))
            } else {
                ("root", rest)
            };

            let (host, port): (&str, u16) = if host_port.contains(':') {
                let parts: Vec<&str> = host_port.splitn(2, ':').collect();
                (parts[0], parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(22))
            } else {
                (host_port, 22u16)
            };

            if !host.is_empty() {
                self.add_connection(user, host, port);
            }
        }
    }
}

impl Default for SshManager {
    fn default() -> Self {
        Self::new()
    }
}
