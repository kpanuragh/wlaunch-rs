use crate::core::{Config, Item, ItemType};
use anyhow::Result;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct Indexer {
    apps: Vec<Item>,
    scripts: Vec<Item>,
}

impl Indexer {
    pub fn new() -> Self {
        Self {
            apps: Vec::new(),
            scripts: Vec::new(),
        }
    }

    pub fn index(&mut self) -> Result<()> {
        self.index_applications()?;
        self.index_scripts()?;
        Ok(())
    }

    pub fn apps(&self) -> &[Item] {
        &self.apps
    }

    pub fn scripts(&self) -> &[Item] {
        &self.scripts
    }

    pub fn all_items(&self) -> Vec<Item> {
        let mut items = self.apps.clone();
        items.extend(self.scripts.clone());
        items
    }

    fn index_applications(&mut self) -> Result<()> {
        let mut seen_names: HashSet<String> = HashSet::new();
        self.apps.clear();

        // Get XDG data directories
        let data_dirs = Self::get_xdg_data_dirs();

        for data_dir in data_dirs {
            let apps_dir = data_dir.join("applications");
            if !apps_dir.exists() {
                continue;
            }

            if let Ok(entries) = fs::read_dir(&apps_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                        if let Some(item) = self.parse_desktop_file(&path, &mut seen_names) {
                            self.apps.push(item);
                        }
                    }
                }
            }
        }

        // Sort by name
        self.apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(())
    }

    fn get_xdg_data_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // User data dir
        if let Some(data_home) = dirs::data_local_dir() {
            dirs.push(data_home);
        }

        // System data dirs
        if let Ok(xdg_data_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in xdg_data_dirs.split(':') {
                dirs.push(PathBuf::from(dir));
            }
        } else {
            dirs.push(PathBuf::from("/usr/local/share"));
            dirs.push(PathBuf::from("/usr/share"));
        }

        // Flatpak exports
        if let Some(data_home) = dirs::data_local_dir() {
            dirs.push(data_home.join("flatpak/exports/share"));
        }
        dirs.push(PathBuf::from("/var/lib/flatpak/exports/share"));

        // Snap applications
        dirs.push(PathBuf::from("/var/lib/snapd/desktop"));

        dirs
    }

    fn parse_desktop_file(&self, path: &PathBuf, seen_names: &mut HashSet<String>) -> Option<Item> {
        let content = fs::read_to_string(path).ok()?;

        let mut name: Option<String> = None;
        let mut exec: Option<String> = None;
        let mut icon: Option<String> = None;
        let mut comment: Option<String> = None;
        let mut no_display = false;
        let mut hidden = false;
        let mut terminal = false;
        let mut keywords: Vec<String> = Vec::new();

        let mut in_desktop_entry = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with('[') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "Name" if name.is_none() => name = Some(value.to_string()),
                    "Exec" => exec = Some(value.to_string()),
                    "Icon" => icon = Some(value.to_string()),
                    "Comment" if comment.is_none() => comment = Some(value.to_string()),
                    "NoDisplay" => no_display = value.to_lowercase() == "true",
                    "Hidden" => hidden = value.to_lowercase() == "true",
                    "Terminal" => terminal = value.to_lowercase() == "true",
                    "Keywords" => {
                        keywords = value.split(';').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                    }
                    _ => {}
                }
            }
        }

        // Skip hidden entries
        if no_display || hidden {
            return None;
        }

        let name = name?;

        // Skip duplicates
        if seen_names.contains(&name) {
            return None;
        }
        seen_names.insert(name.clone());

        let mut item = Item::new(
            format!("app:{}", name),
            &name,
            ItemType::Application,
        );

        if let Some(comment) = comment {
            item = item.with_description(comment);
        }

        if let Some(ref icon_name) = icon {
            item = item.with_icon(icon_name);
            if let Some(icon_path) = Self::find_icon(icon_name) {
                item = item.with_icon_path(icon_path);
            }
        }

        if let Some(exec) = exec {
            item = item.with_exec(exec);
        }

        item = item.with_keywords(keywords);
        item.metadata.desktop_file = Some(path.clone());
        item.metadata.terminal = terminal;

        Some(item)
    }

    fn index_scripts(&mut self) -> Result<()> {
        self.scripts.clear();
        let scripts_dir = Config::scripts_dir();

        if !scripts_dir.exists() {
            fs::create_dir_all(&scripts_dir)?;
            return Ok(());
        }

        for entry in fs::read_dir(&scripts_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let mut item = Item::new(
                    format!("script:{}", name),
                    &name,
                    ItemType::Script,
                )
                .with_description(format!("Script: {}", path.display()))
                .with_exec(path.to_string_lossy().to_string())
                .with_icon("application-x-executable");

                item.metadata.path = Some(path);
                self.scripts.push(item);
            }
        }

        self.scripts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(())
    }

    fn find_icon(icon_name: &str) -> Option<PathBuf> {
        // Check if it's already an absolute path
        let path = PathBuf::from(icon_name);
        if path.is_absolute() && path.exists() {
            return Some(path);
        }

        // Try common icon directories
        let icon_dirs = [
            "/usr/share/icons/hicolor/48x48/apps",
            "/usr/share/icons/hicolor/64x64/apps",
            "/usr/share/icons/hicolor/128x128/apps",
            "/usr/share/icons/hicolor/scalable/apps",
            "/usr/share/pixmaps",
            "/usr/share/icons/Adwaita/48x48/apps",
            "/usr/share/icons/Adwaita/64x64/apps",
        ];

        let extensions = ["png", "svg", "xpm"];

        for dir in &icon_dirs {
            for ext in &extensions {
                let icon_path = PathBuf::from(dir).join(format!("{}.{}", icon_name, ext));
                if icon_path.exists() {
                    return Some(icon_path);
                }
            }
        }

        None
    }
}

impl Default for Indexer {
    fn default() -> Self {
        Self::new()
    }
}
