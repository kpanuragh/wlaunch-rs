use crate::core::{Item, ItemType};
use std::process::Command;

pub struct AudioManager {
    backend: AudioBackend,
}

enum AudioBackend {
    PipeWire,
    PulseAudio,
}

impl AudioManager {
    pub fn new() -> Self {
        // Detect backend
        let backend = if Command::new("wpctl").arg("--version").output().is_ok() {
            AudioBackend::PipeWire
        } else {
            AudioBackend::PulseAudio
        };

        Self { backend }
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query = query.to_lowercase();
        let mut items = Vec::new();

        // Volume controls
        let volume = self.get_volume();
        let muted = self.is_muted();

        items.push(
            Item::new(
                "audio:volume",
                format!("Volume: {}%{}", volume, if muted { " (Muted)" } else { "" }),
                ItemType::AudioAction,
            )
            .with_description("Current volume level")
            .with_icon(if muted {
                "audio-volume-muted"
            } else if volume > 66 {
                "audio-volume-high"
            } else if volume > 33 {
                "audio-volume-medium"
            } else {
                "audio-volume-low"
            }),
        );

        items.push(
            Item::new("audio:mute", "Toggle Mute", ItemType::AudioAction)
                .with_description("Mute or unmute audio")
                .with_icon("audio-volume-muted"),
        );

        items.push(
            Item::new("audio:up", "Volume Up (+10%)", ItemType::AudioAction)
                .with_description("Increase volume by 10%")
                .with_icon("audio-volume-high"),
        );

        items.push(
            Item::new("audio:down", "Volume Down (-10%)", ItemType::AudioAction)
                .with_description("Decrease volume by 10%")
                .with_icon("audio-volume-low"),
        );

        // Get sinks
        let sinks = self.get_sinks();
        for sink in sinks {
            let mut item = Item::new(
                format!("audio:sink:{}", sink.id),
                &sink.name,
                ItemType::AudioSink,
            )
            .with_description(format!(
                "{}{}",
                sink.description,
                if sink.default { " (Default)" } else { "" }
            ))
            .with_icon("audio-card");

            item.metadata.sink_id = Some(sink.id.clone());
            item.metadata.volume = Some(sink.volume);
            item.metadata.muted = sink.muted;

            items.push(item);
        }

        // Filter by query
        if !query.is_empty() {
            // Check if it's a volume command
            if let Ok(vol) = query.parse::<u32>() {
                items.clear();
                items.push(
                    Item::new(
                        format!("audio:set:{}", vol),
                        format!("Set Volume to {}%", vol),
                        ItemType::AudioAction,
                    )
                    .with_description("Set volume level")
                    .with_icon("audio-volume-medium"),
                );
            } else {
                items.retain(|item| {
                    item.name.to_lowercase().contains(&query)
                        || item
                            .description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query))
                            .unwrap_or(false)
                });
            }
        }

        items
    }

    fn get_volume(&self) -> u32 {
        match self.backend {
            AudioBackend::PipeWire => {
                if let Ok(output) = Command::new("wpctl")
                    .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        // Format: Volume: X.XX [MUTED]
                        if let Some(vol_str) = stdout.split_whitespace().nth(1) {
                            if let Ok(vol) = vol_str.parse::<f32>() {
                                return (vol * 100.0) as u32;
                            }
                        }
                    }
                }
            }
            AudioBackend::PulseAudio => {
                if let Ok(output) = Command::new("pactl")
                    .args(["get-sink-volume", "@DEFAULT_SINK@"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        // Parse percentage
                        if let Some(idx) = stdout.find('%') {
                            let start = stdout[..idx].rfind(' ').unwrap_or(0) + 1;
                            if let Ok(vol) = stdout[start..idx].parse::<u32>() {
                                return vol;
                            }
                        }
                    }
                }
            }
        }
        50
    }

    fn is_muted(&self) -> bool {
        match self.backend {
            AudioBackend::PipeWire => {
                if let Ok(output) = Command::new("wpctl")
                    .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        return stdout.contains("[MUTED]");
                    }
                }
            }
            AudioBackend::PulseAudio => {
                if let Ok(output) = Command::new("pactl")
                    .args(["get-sink-mute", "@DEFAULT_SINK@"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        return stdout.contains("yes");
                    }
                }
            }
        }
        false
    }

    fn get_sinks(&self) -> Vec<AudioSink> {
        let mut sinks = Vec::new();

        match self.backend {
            AudioBackend::PipeWire => {
                if let Ok(output) = Command::new("wpctl")
                    .args(["status"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let mut in_sinks = false;

                        for line in stdout.lines() {
                            if line.contains("Sinks:") {
                                in_sinks = true;
                                continue;
                            }
                            if in_sinks && (line.contains("Sources:") || line.trim().is_empty()) {
                                break;
                            }
                            if in_sinks && line.contains('.') {
                                // Parse sink line
                                let default = line.contains('*');
                                let parts: Vec<&str> = line.split_whitespace().collect();
                                if parts.len() >= 2 {
                                    let id = parts[0].trim_matches(|c| c == '*' || c == '.');
                                    let name = parts[1..].join(" ");
                                    sinks.push(AudioSink {
                                        id: id.to_string(),
                                        name: name.clone(),
                                        description: name,
                                        volume: 100,
                                        muted: false,
                                        default,
                                    });
                                }
                            }
                        }
                    }
                }
            }
            AudioBackend::PulseAudio => {
                if let Ok(output) = Command::new("pactl")
                    .args(["list", "sinks", "short"])
                    .output()
                {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        for line in stdout.lines() {
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 2 {
                                sinks.push(AudioSink {
                                    id: parts[0].to_string(),
                                    name: parts[1].to_string(),
                                    description: parts[1].to_string(),
                                    volume: 100,
                                    muted: false,
                                    default: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        sinks
    }

    pub fn set_volume(&self, volume: u32) {
        let vol_str = format!("{}%", volume.min(150));
        match self.backend {
            AudioBackend::PipeWire => {
                let _ = Command::new("wpctl")
                    .args(["set-volume", "@DEFAULT_AUDIO_SINK@", &vol_str])
                    .output();
            }
            AudioBackend::PulseAudio => {
                let _ = Command::new("pactl")
                    .args(["set-sink-volume", "@DEFAULT_SINK@", &vol_str])
                    .output();
            }
        }
    }

    pub fn toggle_mute(&self) {
        match self.backend {
            AudioBackend::PipeWire => {
                let _ = Command::new("wpctl")
                    .args(["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])
                    .output();
            }
            AudioBackend::PulseAudio => {
                let _ = Command::new("pactl")
                    .args(["set-sink-mute", "@DEFAULT_SINK@", "toggle"])
                    .output();
            }
        }
    }

    pub fn set_default_sink(&self, sink_id: &str) {
        match self.backend {
            AudioBackend::PipeWire => {
                let _ = Command::new("wpctl")
                    .args(["set-default", sink_id])
                    .output();
            }
            AudioBackend::PulseAudio => {
                let _ = Command::new("pactl")
                    .args(["set-default-sink", sink_id])
                    .output();
            }
        }
    }

    pub fn execute_action(&self, action_id: &str, query: &str) {
        match action_id {
            "audio:mute" => self.toggle_mute(),
            "audio:up" => {
                let current = self.get_volume();
                self.set_volume(current + 10);
            }
            "audio:down" => {
                let current = self.get_volume();
                self.set_volume(current.saturating_sub(10));
            }
            id if id.starts_with("audio:set:") => {
                if let Ok(vol) = id.strip_prefix("audio:set:").unwrap().parse::<u32>() {
                    self.set_volume(vol);
                }
            }
            id if id.starts_with("audio:sink:") => {
                let sink_id = id.strip_prefix("audio:sink:").unwrap();
                self.set_default_sink(sink_id);
            }
            _ => {
                // Try parsing query as volume
                if let Ok(vol) = query.parse::<u32>() {
                    self.set_volume(vol);
                }
            }
        }
    }
}

struct AudioSink {
    id: String,
    name: String,
    description: String,
    volume: u32,
    muted: bool,
    default: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
