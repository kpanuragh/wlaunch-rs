use crate::core::{Config, Indexer, Item, ItemType};
use crate::features::*;
use crate::ui::theme;
use iced::widget::{
    button, column, container, horizontal_space, image, row, scrollable, svg, text, text_input,
    Column, Row,
};

// Scrollable ID for auto-scrolling
fn results_scrollable_id() -> scrollable::Id {
    scrollable::Id::new("results_list")
}
use iced::{
    event, keyboard, window, Element, Event, Length, Subscription, Task,
};
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Mutex;

// Message types for the application
#[derive(Debug, Clone)]
pub enum Message {
    // Search
    SearchChanged(String),
    SearchSubmit,

    // Navigation
    SelectNext,
    SelectPrevious,
    SelectItem(usize),
    ExecuteSelected,
    ExecuteItem(usize),

    // Indexing
    IndexingComplete(Vec<Item>),

    // Feature messages
    ClipboardUpdated(Vec<Item>),
    ProcessesUpdated(Vec<Item>),
    WindowsUpdated(Vec<Item>),
    NetworkUpdated(Vec<Item>),
    BluetoothUpdated(Vec<Item>),
    AudioUpdated(Vec<Item>),
    NotesUpdated(Vec<Item>),
    TodosUpdated(Vec<Item>),
    SnippetsUpdated(Vec<Item>),
    SshUpdated(Vec<Item>),
    DockerUpdated(Vec<Item>),
    RecentFilesUpdated(Vec<Item>),
    FilesSearchResult(Vec<Item>),
    AiResponse(String),
    TimerTick,

    // Actions
    CopyToClipboard(String),
    OpenUrl(String),
    ShowNotification(String),

    // Window
    CloseWindow,
    Escape,
    WindowUnfocused,

    // Keyboard
    KeyPressed(keyboard::Key, keyboard::Modifiers),

    // Events
    EventOccurred(Event),
}

// Application mode based on search prefix
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Apps,
    Windows,
    Processes,
    Wifi,
    Bluetooth,
    Audio,
    Clipboard,
    Notes,
    Snippets,
    Todos,
    Ssh,
    Docker,
    Timer,
    Emoji,
    Files,
    RecentFiles,
    Bitwarden,
    Ai,
    WebSearch,
    Calculator,
    Converter,
}

impl Mode {
    fn from_query(query: &str) -> (Self, String) {
        let query = query.trim();
        let parts: Vec<&str> = query.splitn(2, ' ').collect();
        let prefix = parts.first().unwrap_or(&"").to_lowercase();
        let remainder = parts.get(1).map(|s| s.to_string()).unwrap_or_default();

        match prefix.as_str() {
            "w" | "window" | "windows" => (Mode::Windows, remainder),
            "ps" | "proc" | "process" => (Mode::Processes, remainder),
            "wifi" | "network" => (Mode::Wifi, remainder),
            "bt" | "bluetooth" => (Mode::Bluetooth, remainder),
            "vol" | "volume" | "audio" => (Mode::Audio, remainder),
            "cb" | "clip" | "clipboard" => (Mode::Clipboard, remainder),
            "note" | "notes" => (Mode::Notes, remainder),
            "snip" | "snippet" | "snippets" => (Mode::Snippets, remainder),
            "todo" | "todos" | "task" | "tasks" => (Mode::Todos, remainder),
            "ssh" => (Mode::Ssh, remainder),
            "docker" | "container" | "containers" => (Mode::Docker, remainder),
            "timer" | "stopwatch" => (Mode::Timer, remainder),
            "e" | "emoji" => (Mode::Emoji, remainder),
            "f" | "find" | "file" | "files" => (Mode::Files, remainder),
            "r" | "recent" => (Mode::RecentFiles, remainder),
            "bw" | "bitwarden" | "pass" | "password" => (Mode::Bitwarden, remainder),
            "ask" | "ai" | "?" => (Mode::Ai, remainder),
            "g" | "google" => (Mode::WebSearch, format!("google {}", remainder)),
            "gh" | "github" => (Mode::WebSearch, format!("github {}", remainder)),
            "yt" | "youtube" => (Mode::WebSearch, format!("youtube {}", remainder)),
            _ => {
                // Check for calculator or converter
                if is_math_expression(query) {
                    (Mode::Calculator, query.to_string())
                } else if is_conversion(query) {
                    (Mode::Converter, query.to_string())
                } else {
                    (Mode::Apps, query.to_string())
                }
            }
        }
    }
}

fn is_math_expression(query: &str) -> bool {
    let has_operators = query.chars().any(|c| "+-*/^%()".contains(c));
    let has_numbers = query.chars().any(|c| c.is_ascii_digit());
    has_operators && has_numbers
}

fn is_conversion(query: &str) -> bool {
    let query_lower = query.to_lowercase();
    query_lower.contains(" to ") || query_lower.contains(" in ")
}

pub struct WLaunch {
    // Search state
    search_query: String,
    mode: Mode,
    mode_query: String,

    // Items
    all_items: Vec<Item>,
    filtered_items: Vec<Item>,
    selected_index: usize,

    // Managers
    indexer: Arc<Mutex<Indexer>>,
    clipboard_manager: ClipboardManager,
    process_manager: ProcessManager,
    windows_manager: WindowsManager,
    network_manager: NetworkManager,
    bluetooth_manager: BluetoothManager,
    audio_manager: AudioManager,
    notes_manager: NotesManager,
    todos_manager: TodosManager,
    snippets_manager: SnippetsManager,
    ssh_manager: SshManager,
    docker_manager: DockerManager,
    emoji_manager: EmojiManager,
    file_manager: FileManager,
    recent_files_manager: RecentFilesManager,
    bitwarden_manager: BitwardenManager,
    ai_manager: AiManager,
    websearch_manager: WebSearchManager,
    calculator: Calculator,
    converter: Converter,
    timer_manager: TimerManager,

    // Config
    config: Config,
}

impl WLaunch {
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::load().unwrap_or_default();
        let indexer = Arc::new(Mutex::new(Indexer::new()));

        let app = Self {
            search_query: String::new(),
            mode: Mode::Apps,
            mode_query: String::new(),
            all_items: Vec::new(),
            filtered_items: Vec::new(),
            selected_index: 0,
            indexer: indexer.clone(),
            clipboard_manager: ClipboardManager::new(),
            process_manager: ProcessManager::new(),
            windows_manager: WindowsManager::new(),
            network_manager: NetworkManager::new(),
            bluetooth_manager: BluetoothManager::new(),
            audio_manager: AudioManager::new(),
            notes_manager: NotesManager::new(),
            todos_manager: TodosManager::new(),
            snippets_manager: SnippetsManager::new(),
            ssh_manager: SshManager::new(),
            docker_manager: DockerManager::new(),
            emoji_manager: EmojiManager::new(),
            file_manager: FileManager::new(),
            recent_files_manager: RecentFilesManager::new(),
            bitwarden_manager: BitwardenManager::new(&config),
            ai_manager: AiManager::new(&config),
            websearch_manager: WebSearchManager::new(),
            calculator: Calculator::new(),
            converter: Converter::new(),
            timer_manager: TimerManager::new(),
            config,
        };

        // Start indexing in background
        let task = Task::perform(
            async move {
                let mut indexer = indexer.lock().await;
                let _ = indexer.index();
                indexer.all_items()
            },
            Message::IndexingComplete,
        );

        (app, task)
    }

    pub fn title(&self) -> String {
        "WLaunch".to_string()
    }

    pub fn theme(&self) -> iced::Theme {
        theme::Theme::custom()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SearchChanged(query) => {
                self.search_query = query.clone();
                let (mode, mode_query) = Mode::from_query(&query);
                self.mode = mode;
                self.mode_query = mode_query;
                self.filter_items();
                self.selected_index = 0;
                Task::none()
            }
            Message::SearchSubmit | Message::ExecuteSelected => {
                self.execute_selected()
            }
            Message::SelectNext => {
                if !self.filtered_items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
                }
                Task::none()
            }
            Message::SelectPrevious => {
                if !self.filtered_items.is_empty() {
                    if self.selected_index == 0 {
                        self.selected_index = self.filtered_items.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }
                Task::none()
            }
            Message::SelectItem(index) => {
                self.selected_index = index;
                Task::none()
            }
            Message::ExecuteItem(index) => {
                self.selected_index = index;
                self.execute_selected()
            }
            Message::IndexingComplete(items) => {
                self.all_items = items;
                self.filter_items();
                Task::none()
            }
            Message::ClipboardUpdated(items) => {
                if self.mode == Mode::Clipboard {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::ProcessesUpdated(items) => {
                if self.mode == Mode::Processes {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::WindowsUpdated(items) => {
                if self.mode == Mode::Windows {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::NetworkUpdated(items) => {
                if self.mode == Mode::Wifi {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::BluetoothUpdated(items) => {
                if self.mode == Mode::Bluetooth {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::AudioUpdated(items) => {
                if self.mode == Mode::Audio {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::NotesUpdated(items) => {
                if self.mode == Mode::Notes {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::TodosUpdated(items) => {
                if self.mode == Mode::Todos {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::SnippetsUpdated(items) => {
                if self.mode == Mode::Snippets {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::SshUpdated(items) => {
                if self.mode == Mode::Ssh {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::DockerUpdated(items) => {
                if self.mode == Mode::Docker {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::RecentFilesUpdated(items) => {
                if self.mode == Mode::RecentFiles {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::FilesSearchResult(items) => {
                if self.mode == Mode::Files {
                    self.filtered_items = items;
                }
                Task::none()
            }
            Message::AiResponse(_response) => {
                // Handle AI response - could show in details panel
                Task::none()
            }
            Message::TimerTick => {
                self.timer_manager.tick();
                Task::none()
            }
            Message::CopyToClipboard(content) => {
                let _ = self.clipboard_manager.copy(&content);
                window::get_latest().and_then(window::close)
            }
            Message::OpenUrl(url) => {
                let _ = Command::new("xdg-open").arg(&url).spawn();
                window::get_latest().and_then(window::close)
            }
            Message::ShowNotification(msg) => {
                let _ = notify_rust::Notification::new()
                    .summary("WLaunch")
                    .body(&msg)
                    .show();
                Task::none()
            }
            Message::CloseWindow | Message::Escape | Message::WindowUnfocused => {
                window::get_latest().and_then(window::close)
            }
            Message::KeyPressed(key, modifiers) => {
                self.handle_key(key, modifiers)
            }
            Message::EventOccurred(_event) => {
                // Focus handling disabled - was causing immediate close
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let search_input = text_input("Search apps, commands, files...", &self.search_query)
            .on_input(Message::SearchChanged)
            .on_submit(Message::SearchSubmit)
            .padding(15)
            .size(18)
            .id(text_input::Id::new("search"))
            .style(theme::search_input);

        let search_bar = container(search_input)
            .width(Length::Fill)
            .padding(10)
            .style(theme::search_container);

        // Results list
        let results: Element<Message> = if self.filtered_items.is_empty() {
            container(
                text("No results found")
                    .size(14)
                    .style(theme::secondary_text),
            )
            .padding(20)
            .center_x(Length::Fill)
            .into()
        } else {
            let items: Vec<Element<Message>> = self
                .filtered_items
                .iter()
                .enumerate()
                .map(|(i, item)| self.render_item(i, item))
                .collect();

            scrollable(Column::with_children(items).spacing(2))
                .id(results_scrollable_id())
                .height(Length::Fill)
                .style(theme::scrollable_style)
                .into()
        };

        let results_panel = container(results)
            .width(Length::FillPortion(6))
            .height(Length::Fill)
            .padding(5)
            .style(theme::results_container);

        // Details panel
        let details = self.render_details();
        let details_panel = container(details)
            .width(Length::FillPortion(4))
            .height(Length::Fill)
            .padding(15)
            .style(theme::details_container);

        let content = row![results_panel, details_panel].spacing(10);

        let main_content = column![search_bar, content]
            .spacing(10)
            .padding(10);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::main_container)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::on_key_press(|key, modifiers| {
                Some(Message::KeyPressed(key, modifiers))
            }),
            event::listen().map(Message::EventOccurred),
        ])
    }

    fn render_item(&self, index: usize, item: &Item) -> Element<'static, Message> {
        let selected = index == self.selected_index;

        let icon_element: Element<'static, Message> = if let Some(icon_path) = &item.icon_path {
            let path_str = icon_path.to_string_lossy();
            if path_str.ends_with(".svg") {
                svg(svg::Handle::from_path(icon_path))
                    .width(24)
                    .height(24)
                    .into()
            } else {
                image(image::Handle::from_path(icon_path))
                    .width(24)
                    .height(24)
                    .into()
            }
        } else {
            let icon_text = match &item.icon {
                Some(icon) => self.get_icon_char(icon),
                None => self.get_type_icon(&item.item_type),
            };
            text(icon_text).size(16).into()
        };

        let name = text(item.name.clone()).size(14);

        let description = item
            .description
            .clone()
            .map(|d| {
                text(if d.len() > 50 {
                    format!("{}...", &d[..47])
                } else {
                    d
                })
                .size(12)
                .style(theme::secondary_text)
            });

        let mut row_content: Row<'static, Message> = row![icon_element, name].spacing(10).align_y(iced::Alignment::Center);

        if let Some(desc) = description {
            row_content = row_content.push(horizontal_space()).push(desc);
        }

        let btn = button(row_content)
            .width(Length::Fill)
            .padding([8, 12])
            .on_press(Message::ExecuteItem(index))
            .style(move |theme, status| {
                match status {
                    button::Status::Hovered => theme::item_button_hover(theme),
                    _ => theme::item_button(theme, selected),
                }
            });

        btn.into()
    }

    fn render_details(&self) -> Element<Message> {
        if let Some(item) = self.filtered_items.get(self.selected_index) {
            let title = text(&item.name).size(20);

            let type_badge = text(format!("{:?}", item.item_type))
                .size(12)
                .style(theme::accent_text);

            let mut content = column![title, type_badge].spacing(10);

            if let Some(desc) = &item.description {
                content = content.push(
                    text(desc)
                        .size(14)
                        .style(theme::secondary_text),
                );
            }

            // Add metadata based on item type
            content = self.add_metadata_to_details(content, item);

            // Add action hint
            let hint = text("Press Enter to execute")
                .size(12)
                .style(theme::secondary_text);

            content = content.push(iced::widget::vertical_space()).push(hint);

            content.spacing(15).into()
        } else {
            column![
                text("No item selected").size(16).style(theme::secondary_text)
            ]
            .into()
        }
    }

    fn add_metadata_to_details<'a>(
        &self,
        mut content: Column<'a, Message>,
        item: &Item,
    ) -> Column<'a, Message> {
        match item.item_type {
            ItemType::Application => {
                if let Some(exec) = &item.exec {
                    content = content.push(
                        text(format!("Command: {}", exec))
                            .size(12)
                            .style(theme::secondary_text),
                    );
                }
                if item.metadata.terminal {
                    content = content.push(
                        text("Runs in terminal")
                            .size(12)
                            .style(theme::secondary_text),
                    );
                }
            }
            ItemType::Process => {
                if let Some(pid) = item.metadata.pid {
                    content = content.push(text(format!("PID: {}", pid)).size(12));
                }
                if let Some(cpu) = item.metadata.cpu {
                    content = content.push(text(format!("CPU: {:.1}%", cpu)).size(12));
                }
                if let Some(mem) = item.metadata.memory {
                    content = content.push(text(format!("Memory: {:.1}%", mem)).size(12));
                }
            }
            ItemType::File | ItemType::RecentFile => {
                if let Some(path) = &item.metadata.path {
                    content = content.push(
                        text(format!("Path: {}", path.display()))
                            .size(12)
                            .style(theme::secondary_text),
                    );
                }
                if let Some(size) = item.metadata.size {
                    content = content.push(text(format!("Size: {} bytes", size)).size(12));
                }
            }
            ItemType::WifiNetwork => {
                if let Some(signal) = item.metadata.signal_strength {
                    content = content.push(text(format!("Signal: {}%", signal)).size(12));
                }
                if item.metadata.secured {
                    content = content.push(text("Secured").size(12));
                }
                if item.metadata.connected {
                    content = content.push(
                        text("Connected")
                            .size(12)
                            .style(theme::accent_text),
                    );
                }
            }
            ItemType::Window => {
                if let Some(ws) = &item.metadata.workspace {
                    content = content.push(text(format!("Workspace: {}", ws)).size(12));
                }
            }
            ItemType::DockerContainer => {
                if let Some(status) = &item.metadata.container_status {
                    content = content.push(text(format!("Status: {}", status)).size(12));
                }
                if let Some(image) = &item.metadata.image {
                    content = content.push(text(format!("Image: {}", image)).size(12));
                }
            }
            ItemType::SshConnection => {
                if let Some(host) = &item.metadata.host {
                    content = content.push(text(format!("Host: {}", host)).size(12));
                }
                if let Some(user) = &item.metadata.user {
                    content = content.push(text(format!("User: {}", user)).size(12));
                }
            }
            _ => {}
        }

        content
    }

    fn filter_items(&mut self) {
        match self.mode {
            Mode::Apps => {
                let query = self.mode_query.to_lowercase();
                if query.is_empty() {
                    self.filtered_items = self.all_items.clone();
                } else {
                    let mut items: Vec<(Item, i64)> = self
                        .all_items
                        .iter()
                        .filter_map(|item| {
                            let score = item.fuzzy_score(&query);
                            if score > 0 {
                                Some((item.clone(), score))
                            } else {
                                None
                            }
                        })
                        .collect();

                    items.sort_by(|a, b| b.1.cmp(&a.1));
                    self.filtered_items = items.into_iter().map(|(item, _)| item).collect();
                }
            }
            Mode::Clipboard => {
                self.filtered_items = self.clipboard_manager.get_items(&self.mode_query);
            }
            Mode::Processes => {
                self.filtered_items = self.process_manager.get_items(&self.mode_query);
            }
            Mode::Windows => {
                self.filtered_items = self.windows_manager.get_items(&self.mode_query);
            }
            Mode::Wifi => {
                self.filtered_items = self.network_manager.get_items(&self.mode_query);
            }
            Mode::Bluetooth => {
                self.filtered_items = self.bluetooth_manager.get_items(&self.mode_query);
            }
            Mode::Audio => {
                self.filtered_items = self.audio_manager.get_items(&self.mode_query);
            }
            Mode::Notes => {
                self.filtered_items = self.notes_manager.get_items(&self.mode_query);
            }
            Mode::Todos => {
                self.filtered_items = self.todos_manager.get_items(&self.mode_query);
            }
            Mode::Snippets => {
                self.filtered_items = self.snippets_manager.get_items(&self.mode_query);
            }
            Mode::Ssh => {
                self.filtered_items = self.ssh_manager.get_items(&self.mode_query);
            }
            Mode::Docker => {
                self.filtered_items = self.docker_manager.get_items(&self.mode_query);
            }
            Mode::Timer => {
                self.filtered_items = self.timer_manager.get_items(&self.mode_query);
            }
            Mode::Emoji => {
                self.filtered_items = self.emoji_manager.get_items(&self.mode_query);
            }
            Mode::Files => {
                self.filtered_items = self.file_manager.get_items(&self.mode_query);
            }
            Mode::RecentFiles => {
                self.filtered_items = self.recent_files_manager.get_items(&self.mode_query);
            }
            Mode::Bitwarden => {
                self.filtered_items = self.bitwarden_manager.get_items(&self.mode_query);
            }
            Mode::Ai => {
                self.filtered_items = self.ai_manager.get_items(&self.mode_query);
            }
            Mode::WebSearch => {
                self.filtered_items = self.websearch_manager.get_items(&self.mode_query);
            }
            Mode::Calculator => {
                self.filtered_items = self.calculator.get_items(&self.mode_query);
            }
            Mode::Converter => {
                self.filtered_items = self.converter.get_items(&self.mode_query);
            }
        }
    }

    fn execute_selected(&mut self) -> Task<Message> {
        if let Some(item) = self.filtered_items.get(self.selected_index).cloned() {
            self.execute_item(&item)
        } else {
            Task::none()
        }
    }

    fn execute_item(&mut self, item: &Item) -> Task<Message> {
        match item.item_type {
            ItemType::Application | ItemType::Script => {
                if let Some(exec) = &item.exec {
                    // Clean up exec string (remove %f, %F, %u, %U, etc.)
                    let exec_clean = exec
                        .replace("%f", "")
                        .replace("%F", "")
                        .replace("%u", "")
                        .replace("%U", "")
                        .replace("%c", "")
                        .replace("%k", "")
                        .replace("%i", "")
                        .trim()
                        .to_string();

                    if item.metadata.terminal {
                        let _ = Command::new("x-terminal-emulator")
                            .arg("-e")
                            .arg(&exec_clean)
                            .spawn();
                    } else {
                        let _ = Command::new("sh").arg("-c").arg(&exec_clean).spawn();
                    }
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::Window => {
                if let Some(window_id) = item.metadata.window_id {
                    self.windows_manager.focus_window(window_id);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::Process => {
                if let Some(pid) = item.metadata.pid {
                    self.process_manager.kill_process(pid);
                    self.filter_items();
                }
                Task::none()
            }
            ItemType::WifiNetwork => {
                if let Some(ssid) = &item.metadata.ssid {
                    self.network_manager.connect(ssid);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::WifiAction => {
                self.network_manager.execute_action(&item.id);
                self.filter_items();
                Task::none()
            }
            ItemType::BluetoothDevice => {
                if let Some(mac) = &item.metadata.mac_address {
                    self.bluetooth_manager.connect(mac);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::BluetoothAction => {
                self.bluetooth_manager.execute_action(&item.id);
                self.filter_items();
                Task::none()
            }
            ItemType::AudioSink => {
                if let Some(sink_id) = &item.metadata.sink_id {
                    self.audio_manager.set_default_sink(sink_id);
                }
                Task::none()
            }
            ItemType::AudioAction => {
                self.audio_manager.execute_action(&item.id, &self.mode_query);
                self.filter_items();
                Task::none()
            }
            ItemType::File | ItemType::RecentFile => {
                if let Some(path) = &item.metadata.path {
                    let _ = Command::new("xdg-open").arg(path).spawn();
                    self.recent_files_manager.add_file(path);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::ClipboardEntry => {
                if let Some(content) = &item.metadata.clipboard_content {
                    let _ = self.clipboard_manager.copy(content);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::Note => {
                if let Some(content) = &item.metadata.content {
                    let _ = self.clipboard_manager.copy(content);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::NoteAction => {
                self.notes_manager.execute_action(&item.id, &self.mode_query);
                self.filter_items();
                Task::none()
            }
            ItemType::Snippet => {
                if let Some(content) = &item.metadata.content {
                    let _ = self.clipboard_manager.copy(content);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::SnippetAction => {
                self.snippets_manager.execute_action(&item.id, &self.mode_query);
                self.filter_items();
                Task::none()
            }
            ItemType::Todo => {
                self.todos_manager.toggle_todo(&item.id);
                self.filter_items();
                Task::none()
            }
            ItemType::TodoAction => {
                self.todos_manager.execute_action(&item.id, &self.mode_query);
                self.filter_items();
                Task::none()
            }
            ItemType::SshConnection => {
                if let Some(host) = &item.metadata.host {
                    let user = item.metadata.user.as_deref().unwrap_or("root");
                    let port = item.metadata.port.unwrap_or(22);
                    let _ = Command::new("x-terminal-emulator")
                        .arg("-e")
                        .arg(format!("ssh -p {} {}@{}", port, user, host))
                        .spawn();
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::SshAction => {
                self.ssh_manager.execute_action(&item.id, &self.mode_query);
                self.filter_items();
                Task::none()
            }
            ItemType::DockerContainer => {
                if let Some(container_id) = &item.metadata.container_id {
                    self.docker_manager.toggle_container(container_id);
                    self.filter_items();
                }
                Task::none()
            }
            ItemType::DockerAction => {
                self.docker_manager.execute_action(&item.id);
                self.filter_items();
                Task::none()
            }
            ItemType::Emoji => {
                let _ = self.clipboard_manager.copy(&item.name);
                window::get_latest().and_then(window::close)
            }
            ItemType::Timer => {
                self.timer_manager.execute_action(&item.id);
                self.filter_items();
                Task::none()
            }
            ItemType::TimerAction => {
                self.timer_manager.execute_action(&item.id);
                self.filter_items();
                Task::none()
            }
            ItemType::Calculator => {
                if let Some(content) = &item.metadata.content {
                    let _ = self.clipboard_manager.copy(content);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::Converter => {
                if let Some(content) = &item.metadata.content {
                    let _ = self.clipboard_manager.copy(content);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::WebSearch => {
                if let Some(url) = &item.metadata.url {
                    let _ = Command::new("xdg-open").arg(url).spawn();
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::BitwardenItem => {
                if let Some(password) = &item.metadata.password {
                    let _ = self.clipboard_manager.copy(password);
                }
                window::get_latest().and_then(window::close)
            }
            ItemType::BitwardenAction => {
                self.bitwarden_manager.execute_action(&item.id);
                self.filter_items();
                Task::none()
            }
            ItemType::AiQuery => {
                // Trigger AI query
                let query = self.mode_query.clone();
                let ai = self.ai_manager.clone();
                Task::perform(
                    async move { ai.query(&query).await },
                    |result| match result {
                        Ok(response) => Message::AiResponse(response),
                        Err(_) => Message::AiResponse("Error querying AI".to_string()),
                    },
                )
            }
            ItemType::AiResponse => {
                if let Some(content) = &item.metadata.content {
                    let _ = self.clipboard_manager.copy(content);
                }
                window::get_latest().and_then(window::close)
            }
            _ => Task::none(),
        }
    }

    fn handle_key(&mut self, key: keyboard::Key, modifiers: keyboard::Modifiers) -> Task<Message> {
        match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::Escape) => {
                window::get_latest().and_then(window::close)
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                if !self.filtered_items.is_empty() {
                    self.selected_index = (self.selected_index + 1) % self.filtered_items.len();
                }
                self.scroll_to_selected()
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                if !self.filtered_items.is_empty() {
                    if self.selected_index == 0 {
                        self.selected_index = self.filtered_items.len() - 1;
                    } else {
                        self.selected_index -= 1;
                    }
                }
                self.scroll_to_selected()
            }
            keyboard::Key::Named(keyboard::key::Named::Enter) => self.execute_selected(),
            keyboard::Key::Character(ref c) if modifiers.command() => {
                let ch = c.to_string();
                // Ctrl+J or Ctrl+N = next item
                if ch == "j" || ch == "n" {
                    if !self.filtered_items.is_empty() {
                        self.selected_index =
                            (self.selected_index + 1) % self.filtered_items.len();
                    }
                    self.scroll_to_selected()
                // Ctrl+K or Ctrl+P = previous item
                } else if ch == "k" || ch == "p" {
                    if !self.filtered_items.is_empty() {
                        if self.selected_index == 0 {
                            self.selected_index = self.filtered_items.len() - 1;
                        } else {
                            self.selected_index -= 1;
                        }
                    }
                    self.scroll_to_selected()
                } else {
                    Task::none()
                }
            }
            _ => Task::none(),
        }
    }

    fn scroll_to_selected(&self) -> Task<Message> {
        // Each item is approximately 42px (40px height + 2px spacing)
        const ITEM_HEIGHT: f32 = 42.0;
        let offset = self.selected_index as f32 * ITEM_HEIGHT;
        scrollable::scroll_to(
            results_scrollable_id(),
            scrollable::AbsoluteOffset { x: 0.0, y: offset },
        )
    }

    fn get_icon_char(&self, icon: &str) -> &'static str {
        // Map common icon names to simple text icons
        match icon {
            s if s.contains("firefox") => "[FF]",
            s if s.contains("chrome") || s.contains("chromium") => "[CR]",
            s if s.contains("terminal") || s.contains("console") => "[>_]",
            s if s.contains("file") || s.contains("folder") => "[D]",
            s if s.contains("code") || s.contains("editor") => "[<>]",
            s if s.contains("music") || s.contains("audio") => "[M]",
            s if s.contains("video") || s.contains("movie") => "[V]",
            s if s.contains("image") || s.contains("photo") => "[I]",
            s if s.contains("mail") || s.contains("email") => "[@]",
            s if s.contains("calendar") => "[C]",
            s if s.contains("settings") || s.contains("preferences") => "[S]",
            s if s.contains("network") || s.contains("wifi") => "[W]",
            s if s.contains("bluetooth") => "[B]",
            s if s.contains("lock") || s.contains("security") => "[L]",
            _ => "[*]",
        }
    }

    fn get_type_icon(&self, item_type: &ItemType) -> &'static str {
        match item_type {
            ItemType::Application => "[A]",
            ItemType::Script => "[#]",
            ItemType::Window => "[W]",
            ItemType::WifiNetwork | ItemType::WifiAction => "[~]",
            ItemType::BluetoothDevice | ItemType::BluetoothAction => "[B]",
            ItemType::AudioSink | ItemType::AudioAction => "[S]",
            ItemType::File | ItemType::Folder => "[F]",
            ItemType::RecentFile => "[R]",
            ItemType::ClipboardEntry => "[C]",
            ItemType::Note | ItemType::NoteAction => "[N]",
            ItemType::Snippet | ItemType::SnippetAction => "[<]",
            ItemType::Todo | ItemType::TodoAction => "[T]",
            ItemType::SshConnection | ItemType::SshAction => "[$]",
            ItemType::DockerContainer | ItemType::DockerAction => "[D]",
            ItemType::Process => "[P]",
            ItemType::Emoji => "[:)]",
            ItemType::Timer | ItemType::TimerAction => "[O]",
            ItemType::Calculator => "[=]",
            ItemType::Converter => "[>]",
            ItemType::WebSearch => "[?]",
            ItemType::BitwardenItem | ItemType::BitwardenAction => "[K]",
            ItemType::AiQuery | ItemType::AiResponse => "[AI]",
            ItemType::Command => "[>]",
        }
    }
}

impl Default for WLaunch {
    fn default() -> Self {
        Self::new().0
    }
}
