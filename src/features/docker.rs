use crate::core::{Item, ItemType};
use std::process::Command;

pub struct DockerManager {
    runtime: DockerRuntime,
}

enum DockerRuntime {
    Docker,
    Podman,
    None,
}

impl DockerManager {
    pub fn new() -> Self {
        // Detect runtime
        let runtime = if Command::new("docker").arg("--version").output().is_ok() {
            DockerRuntime::Docker
        } else if Command::new("podman").arg("--version").output().is_ok() {
            DockerRuntime::Podman
        } else {
            DockerRuntime::None
        };

        Self { runtime }
    }

    fn runtime_cmd(&self) -> Option<&str> {
        match self.runtime {
            DockerRuntime::Docker => Some("docker"),
            DockerRuntime::Podman => Some("podman"),
            DockerRuntime::None => None,
        }
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query_lower = query.to_lowercase();
        let mut items = Vec::new();

        let Some(cmd) = self.runtime_cmd() else {
            items.push(
                Item::new(
                    "docker:not_found",
                    "Docker/Podman not found",
                    ItemType::DockerAction,
                )
                .with_description("Install Docker or Podman to use this feature")
                .with_icon("dialog-warning"),
            );
            return items;
        };

        // Get containers (all, including stopped)
        if let Ok(output) = Command::new(cmd)
            .args(["ps", "-a", "--format", "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}"])
            .output()
        {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line.split('\t').collect();
                        if parts.len() >= 4 {
                            let id = parts[0];
                            let name = parts[1];
                            let image = parts[2];
                            let status = parts[3];

                            let running = status.starts_with("Up");

                            if query_lower.is_empty()
                                || name.to_lowercase().contains(&query_lower)
                                || image.to_lowercase().contains(&query_lower)
                            {
                                let mut item = Item::new(
                                    format!("docker:{}", id),
                                    name,
                                    ItemType::DockerContainer,
                                )
                                .with_description(format!("{} | {}", image, status))
                                .with_icon(if running {
                                    "media-playback-start"
                                } else {
                                    "media-playback-stop"
                                });

                                item.metadata.container_id = Some(id.to_string());
                                item.metadata.container_status = Some(status.to_string());
                                item.metadata.image = Some(image.to_string());

                                items.push(item);
                            }
                        }
                    }
                }
            }
        }

        // Add actions
        if query_lower.is_empty() {
            items.push(
                Item::new("docker:action:prune", "Prune Containers", ItemType::DockerAction)
                    .with_description("Remove stopped containers")
                    .with_icon("edit-clear-all"),
            );
            items.push(
                Item::new("docker:action:prune_all", "Prune All", ItemType::DockerAction)
                    .with_description("Remove unused containers, images, and volumes")
                    .with_icon("edit-delete"),
            );
        }

        items
    }

    pub fn start_container(&self, container_id: &str) {
        if let Some(cmd) = self.runtime_cmd() {
            let _ = Command::new(cmd)
                .args(["start", container_id])
                .output();
        }
    }

    pub fn stop_container(&self, container_id: &str) {
        if let Some(cmd) = self.runtime_cmd() {
            let _ = Command::new(cmd)
                .args(["stop", container_id])
                .output();
        }
    }

    pub fn remove_container(&self, container_id: &str) {
        if let Some(cmd) = self.runtime_cmd() {
            let _ = Command::new(cmd)
                .args(["rm", "-f", container_id])
                .output();
        }
    }

    pub fn toggle_container(&self, container_id: &str) {
        if let Some(cmd) = self.runtime_cmd() {
            // Check if running
            if let Ok(output) = Command::new(cmd)
                .args(["inspect", "-f", "{{.State.Running}}", container_id])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.trim() == "true" {
                        self.stop_container(container_id);
                    } else {
                        self.start_container(container_id);
                    }
                }
            }
        }
    }

    pub fn prune_containers(&self) {
        if let Some(cmd) = self.runtime_cmd() {
            let _ = Command::new(cmd)
                .args(["container", "prune", "-f"])
                .output();
        }
    }

    pub fn prune_all(&self) {
        if let Some(cmd) = self.runtime_cmd() {
            let _ = Command::new(cmd)
                .args(["system", "prune", "-af"])
                .output();
        }
    }

    pub fn execute_action(&self, action_id: &str) {
        match action_id {
            "docker:action:prune" => self.prune_containers(),
            "docker:action:prune_all" => self.prune_all(),
            id if id.starts_with("docker:start:") => {
                let container_id = id.strip_prefix("docker:start:").unwrap();
                self.start_container(container_id);
            }
            id if id.starts_with("docker:stop:") => {
                let container_id = id.strip_prefix("docker:stop:").unwrap();
                self.stop_container(container_id);
            }
            id if id.starts_with("docker:remove:") => {
                let container_id = id.strip_prefix("docker:remove:").unwrap();
                self.remove_container(container_id);
            }
            _ => {}
        }
    }
}

impl Default for DockerManager {
    fn default() -> Self {
        Self::new()
    }
}
