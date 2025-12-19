use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Note {
    id: String,
    title: String,
    content: String,
    created: String,
    updated: String,
}

pub struct NotesManager {
    notes: Vec<Note>,
}

impl NotesManager {
    pub fn new() -> Self {
        let notes = Self::load().unwrap_or_default();
        Self { notes }
    }

    fn data_path() -> std::path::PathBuf {
        Config::data_path("notes.json")
    }

    fn load() -> Result<Vec<Note>> {
        let path = Self::data_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let notes: Vec<Note> = serde_json::from_str(&content)?;
            Ok(notes)
        } else {
            Ok(Vec::new())
        }
    }

    fn save(&self) -> Result<()> {
        let path = Self::data_path();
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(&self.notes)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query_lower = query.to_lowercase();
        let mut items = Vec::new();

        // Add action to create note
        if query.starts_with("add ") || query.starts_with("new ") {
            let content = query.splitn(2, ' ').nth(1).unwrap_or("");
            if !content.is_empty() {
                items.push(
                    Item::new(
                        format!("note:add:{}", content),
                        format!("Add Note: {}", content),
                        ItemType::NoteAction,
                    )
                    .with_description("Create a new note")
                    .with_icon("document-new"),
                );
            }
        }

        // List existing notes
        for note in &self.notes {
            if query_lower.is_empty()
                || note.title.to_lowercase().contains(&query_lower)
                || note.content.to_lowercase().contains(&query_lower)
            {
                let preview = if note.content.len() > 50 {
                    format!("{}...", &note.content[..47])
                } else {
                    note.content.clone()
                };

                let mut item = Item::new(
                    format!("note:{}", note.id),
                    &note.title,
                    ItemType::Note,
                )
                .with_description(preview)
                .with_icon("text-x-generic");

                item.metadata.content = Some(note.content.clone());
                item.metadata.created = Some(note.created.clone());

                items.push(item);
            }
        }

        // Add delete actions if we have notes and specific query
        if !self.notes.is_empty() && query.is_empty() {
            items.push(
                Item::new("note:action:clear", "Clear All Notes", ItemType::NoteAction)
                    .with_description("Delete all notes")
                    .with_icon("edit-delete"),
            );
        }

        items
    }

    pub fn add_note(&mut self, content: &str) {
        let title = if content.len() > 30 {
            format!("{}...", &content[..27])
        } else {
            content.to_string()
        };

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let note = Note {
            id: Uuid::new_v4().to_string(),
            title,
            content: content.to_string(),
            created: now.clone(),
            updated: now,
        };

        self.notes.insert(0, note);
        let _ = self.save();
    }

    pub fn delete_note(&mut self, id: &str) {
        self.notes.retain(|n| n.id != id);
        let _ = self.save();
    }

    pub fn clear_notes(&mut self) {
        self.notes.clear();
        let _ = self.save();
    }

    pub fn execute_action(&mut self, action_id: &str, query: &str) {
        if action_id.starts_with("note:add:") {
            let content = action_id.strip_prefix("note:add:").unwrap();
            self.add_note(content);
        } else if action_id == "note:action:clear" {
            self.clear_notes();
        } else if action_id.starts_with("note:delete:") {
            let id = action_id.strip_prefix("note:delete:").unwrap();
            self.delete_note(id);
        } else if query.starts_with("add ") || query.starts_with("new ") {
            let content = query.splitn(2, ' ').nth(1).unwrap_or("");
            if !content.is_empty() {
                self.add_note(content);
            }
        }
    }
}

impl Default for NotesManager {
    fn default() -> Self {
        Self::new()
    }
}
