use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snippet {
    id: String,
    name: String,
    content: String,
    language: Option<String>,
    created: String,
}

pub struct SnippetsManager {
    snippets: Vec<Snippet>,
}

impl SnippetsManager {
    pub fn new() -> Self {
        let snippets = Self::load().unwrap_or_default();
        Self { snippets }
    }

    fn data_path() -> std::path::PathBuf {
        Config::data_path("snippets.json")
    }

    fn load() -> Result<Vec<Snippet>> {
        let path = Self::data_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let snippets: Vec<Snippet> = serde_json::from_str(&content)?;
            Ok(snippets)
        } else {
            Ok(Vec::new())
        }
    }

    fn save(&self) -> Result<()> {
        let path = Self::data_path();
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(&self.snippets)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query_lower = query.to_lowercase();
        let mut items = Vec::new();

        // Add action to create snippet
        if query.starts_with("add ") || query.starts_with("new ") {
            let rest = query.splitn(2, ' ').nth(1).unwrap_or("");
            if !rest.is_empty() {
                // Parse "name: content" format
                if let Some(idx) = rest.find(':') {
                    let name = rest[..idx].trim();
                    let content = rest[idx + 1..].trim();
                    if !name.is_empty() && !content.is_empty() {
                        items.push(
                            Item::new(
                                format!("snippet:add:{}:{}", name, content),
                                format!("Add Snippet: {}", name),
                                ItemType::SnippetAction,
                            )
                            .with_description(format!("Content: {}", content))
                            .with_icon("document-new"),
                        );
                    }
                }
            }
        }

        // List existing snippets
        for snippet in &self.snippets {
            if query_lower.is_empty()
                || snippet.name.to_lowercase().contains(&query_lower)
                || snippet.content.to_lowercase().contains(&query_lower)
            {
                let preview = if snippet.content.len() > 50 {
                    format!("{}...", &snippet.content[..47])
                } else {
                    snippet.content.clone()
                };

                let mut item = Item::new(
                    format!("snippet:{}", snippet.id),
                    &snippet.name,
                    ItemType::Snippet,
                )
                .with_description(preview)
                .with_icon("text-x-script");

                item.metadata.content = Some(snippet.content.clone());
                item.metadata.created = Some(snippet.created.clone());

                items.push(item);
            }
        }

        // Add clear action
        if !self.snippets.is_empty() && query.is_empty() {
            items.push(
                Item::new("snippet:action:clear", "Clear All Snippets", ItemType::SnippetAction)
                    .with_description("Delete all snippets")
                    .with_icon("edit-delete"),
            );
        }

        items
    }

    pub fn add_snippet(&mut self, name: &str, content: &str) {
        let snippet = Snippet {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            content: content.to_string(),
            language: None,
            created: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.snippets.insert(0, snippet);
        let _ = self.save();
    }

    pub fn delete_snippet(&mut self, id: &str) {
        self.snippets.retain(|s| s.id != id);
        let _ = self.save();
    }

    pub fn clear_snippets(&mut self) {
        self.snippets.clear();
        let _ = self.save();
    }

    pub fn execute_action(&mut self, action_id: &str, query: &str) {
        if action_id.starts_with("snippet:add:") {
            let rest = action_id.strip_prefix("snippet:add:").unwrap();
            if let Some(idx) = rest.find(':') {
                let name = &rest[..idx];
                let content = &rest[idx + 1..];
                self.add_snippet(name, content);
            }
        } else if action_id == "snippet:action:clear" {
            self.clear_snippets();
        } else if action_id.starts_with("snippet:delete:") {
            let id = action_id.strip_prefix("snippet:delete:").unwrap();
            self.delete_snippet(id);
        } else if query.starts_with("add ") || query.starts_with("new ") {
            let rest = query.splitn(2, ' ').nth(1).unwrap_or("");
            if let Some(idx) = rest.find(':') {
                let name = rest[..idx].trim();
                let content = rest[idx + 1..].trim();
                if !name.is_empty() && !content.is_empty() {
                    self.add_snippet(name, content);
                }
            }
        }
    }
}

impl Default for SnippetsManager {
    fn default() -> Self {
        Self::new()
    }
}
