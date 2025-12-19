use crate::core::{Item, ItemType};
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct FileManager {
    search_paths: Vec<PathBuf>,
}

impl FileManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        Self {
            search_paths: vec![
                home.join("Documents"),
                home.join("Downloads"),
                home.join("Pictures"),
                home.join("Videos"),
                home.join("Music"),
                home.join("Desktop"),
            ],
        }
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        if query.len() < 2 {
            return vec![Item::new(
                "file:hint",
                "Type at least 2 characters to search",
                ItemType::Command,
            )
            .with_description("Search in Documents, Downloads, Pictures, Videos, Music, Desktop")
            .with_icon("system-search")];
        }

        let query_lower = query.to_lowercase();
        let mut items = Vec::new();

        for search_path in &self.search_paths {
            if !search_path.exists() {
                continue;
            }

            for entry in WalkDir::new(search_path)
                .max_depth(4)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if file_name.to_lowercase().contains(&query_lower) {
                    let is_dir = path.is_dir();
                    let size = if is_dir {
                        None
                    } else {
                        path.metadata().ok().map(|m| m.len())
                    };

                    let mime = if is_dir {
                        "directory".to_string()
                    } else {
                        mime_guess::from_path(path)
                            .first()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "application/octet-stream".to_string())
                    };

                    let icon = Self::get_icon_for_mime(&mime);

                    let mut item = Item::new(
                        format!("file:{}", path.display()),
                        file_name,
                        if is_dir { ItemType::Folder } else { ItemType::File },
                    )
                    .with_description(path.display().to_string())
                    .with_icon(icon);

                    item.metadata.path = Some(path.to_path_buf());
                    item.metadata.size = size;
                    item.metadata.mime_type = Some(mime);

                    items.push(item);
                }

                // Limit results
                if items.len() >= 50 {
                    break;
                }
            }

            if items.len() >= 50 {
                break;
            }
        }

        items
    }

    fn get_icon_for_mime(mime: &str) -> &'static str {
        if mime == "directory" {
            return "folder";
        }

        let main_type = mime.split('/').next().unwrap_or("");
        match main_type {
            "image" => "image-x-generic",
            "video" => "video-x-generic",
            "audio" => "audio-x-generic",
            "text" => "text-x-generic",
            "application" => {
                if mime.contains("pdf") {
                    "application-pdf"
                } else if mime.contains("zip") || mime.contains("tar") || mime.contains("compressed") {
                    "package-x-generic"
                } else if mime.contains("spreadsheet") || mime.contains("excel") {
                    "x-office-spreadsheet"
                } else if mime.contains("document") || mime.contains("word") {
                    "x-office-document"
                } else if mime.contains("presentation") || mime.contains("powerpoint") {
                    "x-office-presentation"
                } else {
                    "application-x-executable"
                }
            }
            _ => "text-x-generic",
        }
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self::new()
    }
}
