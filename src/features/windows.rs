use crate::core::{Item, ItemType};
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct I3Node {
    id: i64,
    name: Option<String>,
    #[serde(rename = "type")]
    node_type: String,
    focused: bool,
    #[serde(default)]
    nodes: Vec<I3Node>,
    #[serde(default)]
    floating_nodes: Vec<I3Node>,
    window_properties: Option<WindowProperties>,
    #[serde(default)]
    num: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct WindowProperties {
    class: Option<String>,
    instance: Option<String>,
    title: Option<String>,
}

pub struct WindowsManager;

impl WindowsManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query = query.to_lowercase();
        let mut items = Vec::new();

        // Try i3-msg first
        if let Ok(output) = Command::new("i3-msg").args(["-t", "get_tree"]).output() {
            if output.status.success() {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Ok(tree) = serde_json::from_str::<I3Node>(&stdout) {
                        self.collect_windows(&tree, &mut items, None);
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

    fn collect_windows(&self, node: &I3Node, items: &mut Vec<Item>, workspace: Option<&str>) {
        let current_workspace = if node.node_type == "workspace" {
            node.name.as_deref()
        } else {
            workspace
        };

        // Check if this is a window
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

        // Recurse into children
        for child in &node.nodes {
            self.collect_windows(child, items, current_workspace);
        }
        for child in &node.floating_nodes {
            self.collect_windows(child, items, current_workspace);
        }
    }

    pub fn focus_window(&self, window_id: i64) {
        let _ = Command::new("i3-msg")
            .args(["[con_id=", &window_id.to_string(), "]", "focus"])
            .output();
    }

    pub fn close_window(&self, window_id: i64) {
        let _ = Command::new("i3-msg")
            .args(["[con_id=", &window_id.to_string(), "]", "kill"])
            .output();
    }
}

impl Default for WindowsManager {
    fn default() -> Self {
        Self::new()
    }
}
