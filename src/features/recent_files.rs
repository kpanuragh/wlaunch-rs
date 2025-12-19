use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RecentFile {
    path: PathBuf,
    name: String,
    accessed: String,
}

pub struct RecentFilesManager {
    files: Vec<RecentFile>,
    max_files: usize,
}

impl RecentFilesManager {
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        let files = Self::load().unwrap_or_default();

        Self {
            files,
            max_files: config.max_recent_files(),
        }
    }

    fn data_path() -> PathBuf {
        Config::data_path("recent_files.json")
    }

    fn load() -> Result<Vec<RecentFile>> {
        let path = Self::data_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let files: Vec<RecentFile> = serde_json::from_str(&content)?;
            Ok(files)
        } else {
            Ok(Vec::new())
        }
    }

    fn save(&self) -> Result<()> {
        let path = Self::data_path();
        fs::create_dir_all(path.parent().unwrap())?;
        let content = serde_json::to_string_pretty(&self.files)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query_lower = query.to_lowercase();

        self.files
            .iter()
            .filter(|f| {
                query_lower.is_empty()
                    || f.name.to_lowercase().contains(&query_lower)
                    || f.path.to_string_lossy().to_lowercase().contains(&query_lower)
            })
            .filter(|f| f.path.exists())
            .map(|f| {
                let mime = mime_guess::from_path(&f.path)
                    .first()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                let icon = Self::get_icon_for_mime(&mime);

                let mut item = Item::new(
                    format!("recent:{}", f.path.display()),
                    &f.name,
                    ItemType::RecentFile,
                )
                .with_description(format!("{} | {}", f.path.display(), f.accessed))
                .with_icon(icon);

                item.metadata.path = Some(f.path.clone());
                item.metadata.mime_type = Some(mime);

                item
            })
            .collect()
    }

    pub fn add_file(&mut self, path: &PathBuf) {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Remove if already exists
        self.files.retain(|f| f.path != *path);

        // Add to front
        self.files.insert(
            0,
            RecentFile {
                path: path.clone(),
                name,
                accessed: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            },
        );

        // Trim to max size
        if self.files.len() > self.max_files {
            self.files.truncate(self.max_files);
        }

        let _ = self.save();
    }

    pub fn clear(&mut self) {
        self.files.clear();
        let _ = self.save();
    }

    fn get_icon_for_mime(mime: &str) -> &'static str {
        let main_type = mime.split('/').next().unwrap_or("");
        match main_type {
            "image" => "image-x-generic",
            "video" => "video-x-generic",
            "audio" => "audio-x-generic",
            "text" => "text-x-generic",
            "application" => {
                if mime.contains("pdf") {
                    "application-pdf"
                } else if mime.contains("zip") || mime.contains("tar") {
                    "package-x-generic"
                } else {
                    "application-x-executable"
                }
            }
            _ => "text-x-generic",
        }
    }
}

impl Default for RecentFilesManager {
    fn default() -> Self {
        Self::new()
    }
}
