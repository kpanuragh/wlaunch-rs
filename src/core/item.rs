use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    // Core
    Application,
    Script,

    // Window management
    Window,

    // System
    WifiNetwork,
    WifiAction,
    BluetoothDevice,
    BluetoothAction,
    AudioSink,
    AudioAction,

    // Files
    File,
    RecentFile,
    Folder,

    // Clipboard
    ClipboardEntry,

    // Productivity
    Note,
    NoteAction,
    Snippet,
    SnippetAction,
    Todo,
    TodoAction,

    // Connections
    SshConnection,
    SshAction,
    DockerContainer,
    DockerAction,

    // Utilities
    Process,
    Emoji,
    Timer,
    TimerAction,
    Calculator,
    Converter,

    // Web
    WebSearch,

    // Password
    BitwardenItem,
    BitwardenAction,

    // AI
    AiQuery,
    AiResponse,

    // Commands
    Command,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub icon: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub exec: Option<String>,
    pub keywords: Vec<String>,
    pub metadata: ItemMetadata,
}

#[derive(Debug, Clone, Default)]
pub struct ItemMetadata {
    // Application
    pub desktop_file: Option<PathBuf>,
    pub terminal: bool,

    // Window
    pub window_id: Option<i64>,
    pub workspace: Option<String>,

    // WiFi
    pub ssid: Option<String>,
    pub signal_strength: Option<i32>,
    pub connected: bool,
    pub secured: bool,

    // Bluetooth
    pub mac_address: Option<String>,
    pub paired: bool,

    // Audio
    pub volume: Option<u32>,
    pub muted: bool,
    pub sink_id: Option<String>,

    // File
    pub path: Option<PathBuf>,
    pub size: Option<u64>,
    pub modified: Option<String>,
    pub mime_type: Option<String>,

    // Process
    pub pid: Option<u32>,
    pub cpu: Option<f32>,
    pub memory: Option<f32>,

    // Clipboard
    pub clipboard_content: Option<String>,
    pub timestamp: Option<String>,

    // SSH
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,

    // Docker
    pub container_id: Option<String>,
    pub container_status: Option<String>,
    pub image: Option<String>,

    // Note/Snippet/Todo
    pub content: Option<String>,
    pub created: Option<String>,
    pub completed: bool,

    // Timer
    pub duration: Option<u64>,
    pub remaining: Option<u64>,

    // Bitwarden
    pub username: Option<String>,
    pub password: Option<String>,
    pub totp: Option<String>,
    pub uri: Option<String>,

    // Web search
    pub search_engine: Option<String>,
    pub query: Option<String>,
    pub url: Option<String>,
}

impl Item {
    pub fn new(id: impl Into<String>, name: impl Into<String>, item_type: ItemType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            item_type,
            icon: None,
            icon_path: None,
            exec: None,
            keywords: Vec::new(),
            metadata: ItemMetadata::default(),
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn with_icon_path(mut self, path: PathBuf) -> Self {
        self.icon_path = Some(path);
        self
    }

    pub fn with_exec(mut self, exec: impl Into<String>) -> Self {
        self.exec = Some(exec.into());
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        let name_lower = self.name.to_lowercase();

        if name_lower.contains(&query) {
            return true;
        }

        if let Some(desc) = &self.description {
            if desc.to_lowercase().contains(&query) {
                return true;
            }
        }

        for keyword in &self.keywords {
            if keyword.to_lowercase().contains(&query) {
                return true;
            }
        }

        false
    }

    pub fn fuzzy_score(&self, query: &str) -> i64 {
        use fuzzy_matcher::skim::SkimMatcherV2;
        use fuzzy_matcher::FuzzyMatcher;

        let matcher = SkimMatcherV2::default();
        let mut best_score = 0i64;

        if let Some(score) = matcher.fuzzy_match(&self.name, query) {
            best_score = best_score.max(score);
        }

        if let Some(desc) = &self.description {
            if let Some(score) = matcher.fuzzy_match(desc, query) {
                best_score = best_score.max(score / 2);
            }
        }

        for keyword in &self.keywords {
            if let Some(score) = matcher.fuzzy_match(keyword, query) {
                best_score = best_score.max(score / 2);
            }
        }

        best_score
    }
}
