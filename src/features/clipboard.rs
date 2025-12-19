use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use arboard::Clipboard;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardEntry {
    content: String,
    timestamp: String,
}

pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
    history: Vec<ClipboardEntry>,
    max_size: usize,
}

impl ClipboardManager {
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let history = Self::load_history().unwrap_or_default();

        Self {
            clipboard: Clipboard::new().ok(),
            history,
            max_size: config.clipboard_history_size(),
        }
    }

    fn history_path() -> std::path::PathBuf {
        Config::data_path("clipboard_history.json")
    }

    fn load_history() -> Result<Vec<ClipboardEntry>> {
        let path = Self::history_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let history: Vec<ClipboardEntry> = serde_json::from_str(&content)?;
            Ok(history)
        } else {
            Ok(Vec::new())
        }
    }

    fn save_history(&self) -> Result<()> {
        let path = Self::history_path();
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(&self.history)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn copy(&mut self, text: &str) -> Result<()> {
        if let Some(clipboard) = &mut self.clipboard {
            clipboard.set_text(text)?;

            // Add to history
            let entry = ClipboardEntry {
                content: text.to_string(),
                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            };

            // Remove duplicates
            self.history.retain(|e| e.content != text);
            self.history.insert(0, entry);

            // Trim to max size
            if self.history.len() > self.max_size {
                self.history.truncate(self.max_size);
            }

            let _ = self.save_history();
        }
        Ok(())
    }

    pub fn get_text(&mut self) -> Option<String> {
        self.clipboard.as_mut()?.get_text().ok()
    }

    pub fn add_to_history(&mut self, text: &str) {
        let entry = ClipboardEntry {
            content: text.to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.history.retain(|e| e.content != text);
        self.history.insert(0, entry);

        if self.history.len() > self.max_size {
            self.history.truncate(self.max_size);
        }

        let _ = self.save_history();
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query = query.to_lowercase();

        self.history
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                query.is_empty() || entry.content.to_lowercase().contains(&query)
            })
            .map(|(i, entry)| {
                let preview = if entry.content.len() > 60 {
                    format!("{}...", &entry.content[..57])
                } else {
                    entry.content.clone()
                };

                let mut item = Item::new(
                    format!("clipboard:{}", i),
                    preview,
                    ItemType::ClipboardEntry,
                )
                .with_description(format!("Copied: {}", entry.timestamp))
                .with_icon("edit-paste");

                item.metadata.clipboard_content = Some(entry.content.clone());
                item.metadata.timestamp = Some(entry.timestamp.clone());
                item
            })
            .collect()
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
        let _ = self.save_history();
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}
