use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TodoItem {
    id: String,
    text: String,
    completed: bool,
    created: String,
}

pub struct TodosManager {
    todos: Vec<TodoItem>,
}

impl TodosManager {
    pub fn new() -> Self {
        let todos = Self::load().unwrap_or_default();
        Self { todos }
    }

    fn data_path() -> std::path::PathBuf {
        Config::data_path("todos.json")
    }

    fn load() -> Result<Vec<TodoItem>> {
        let path = Self::data_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let todos: Vec<TodoItem> = serde_json::from_str(&content)?;
            Ok(todos)
        } else {
            Ok(Vec::new())
        }
    }

    fn save(&self) -> Result<()> {
        let path = Self::data_path();
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(&self.todos)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query_lower = query.to_lowercase();
        let mut items = Vec::new();

        // Add action to create todo
        if query.starts_with("add ") || query.starts_with("new ") {
            let text = query.splitn(2, ' ').nth(1).unwrap_or("");
            if !text.is_empty() {
                items.push(
                    Item::new(
                        format!("todo:add:{}", text),
                        format!("Add Todo: {}", text),
                        ItemType::TodoAction,
                    )
                    .with_description("Create a new todo item")
                    .with_icon("checkbox"),
                );
            }
        }

        // List existing todos (incomplete first, then completed)
        let incomplete: Vec<_> = self.todos.iter().filter(|t| !t.completed).collect();
        let completed: Vec<_> = self.todos.iter().filter(|t| t.completed).collect();

        for todo in incomplete.iter().chain(completed.iter()) {
            if query_lower.is_empty() || todo.text.to_lowercase().contains(&query_lower) {
                let prefix = if todo.completed { "✓ " } else { "○ " };

                let mut item = Item::new(
                    format!("todo:{}", todo.id),
                    format!("{}{}", prefix, todo.text),
                    ItemType::Todo,
                )
                .with_description(format!(
                    "Created: {} | {}",
                    todo.created,
                    if todo.completed {
                        "Completed"
                    } else {
                        "Pending"
                    }
                ))
                .with_icon(if todo.completed {
                    "checkbox-checked"
                } else {
                    "checkbox"
                });

                item.metadata.content = Some(todo.text.clone());
                item.metadata.completed = todo.completed;
                item.metadata.created = Some(todo.created.clone());

                items.push(item);
            }
        }

        // Add clear actions
        if !self.todos.is_empty() && query.is_empty() {
            items.push(
                Item::new(
                    "todo:action:clear_completed",
                    "Clear Completed",
                    ItemType::TodoAction,
                )
                .with_description("Remove all completed todos")
                .with_icon("edit-delete"),
            );
            items.push(
                Item::new("todo:action:clear_all", "Clear All", ItemType::TodoAction)
                    .with_description("Remove all todos")
                    .with_icon("edit-delete"),
            );
        }

        items
    }

    pub fn add_todo(&mut self, text: &str) {
        let todo = TodoItem {
            id: Uuid::new_v4().to_string(),
            text: text.to_string(),
            completed: false,
            created: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.todos.insert(0, todo);
        let _ = self.save();
    }

    pub fn toggle_todo(&mut self, id: &str) {
        let id = id.strip_prefix("todo:").unwrap_or(id);
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
            let _ = self.save();
        }
    }

    pub fn delete_todo(&mut self, id: &str) {
        self.todos.retain(|t| t.id != id);
        let _ = self.save();
    }

    pub fn clear_completed(&mut self) {
        self.todos.retain(|t| !t.completed);
        let _ = self.save();
    }

    pub fn clear_all(&mut self) {
        self.todos.clear();
        let _ = self.save();
    }

    pub fn execute_action(&mut self, action_id: &str, query: &str) {
        if action_id.starts_with("todo:add:") {
            let text = action_id.strip_prefix("todo:add:").unwrap();
            self.add_todo(text);
        } else if action_id == "todo:action:clear_completed" {
            self.clear_completed();
        } else if action_id == "todo:action:clear_all" {
            self.clear_all();
        } else if action_id.starts_with("todo:delete:") {
            let id = action_id.strip_prefix("todo:delete:").unwrap();
            self.delete_todo(id);
        } else if query.starts_with("add ") || query.starts_with("new ") {
            let text = query.splitn(2, ' ').nth(1).unwrap_or("");
            if !text.is_empty() {
                self.add_todo(text);
            }
        }
    }
}

impl Default for TodosManager {
    fn default() -> Self {
        Self::new()
    }
}
