use crate::core::{Item, ItemType};
use chrono::{DateTime, Local};
use std::time::Duration;

#[derive(Debug, Clone)]
struct Timer {
    id: String,
    name: String,
    duration: Duration,
    started_at: DateTime<Local>,
    paused: bool,
    paused_remaining: Option<Duration>,
}

impl Timer {
    fn remaining(&self) -> Duration {
        if self.paused {
            self.paused_remaining.unwrap_or(Duration::ZERO)
        } else {
            let elapsed = Local::now()
                .signed_duration_since(self.started_at)
                .to_std()
                .unwrap_or(Duration::ZERO);

            self.duration.saturating_sub(elapsed)
        }
    }

    fn is_finished(&self) -> bool {
        !self.paused && self.remaining() == Duration::ZERO
    }
}

pub struct TimerManager {
    timers: Vec<Timer>,
    stopwatch: Option<DateTime<Local>>,
    stopwatch_paused: bool,
    stopwatch_elapsed: Duration,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            stopwatch: None,
            stopwatch_paused: false,
            stopwatch_elapsed: Duration::ZERO,
        }
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let mut items = Vec::new();

        // Parse timer duration from query
        if let Some(duration) = Self::parse_duration(query) {
            items.push(
                Item::new(
                    format!("timer:start:{}", duration.as_secs()),
                    format!("Start Timer: {}", Self::format_duration(duration)),
                    ItemType::TimerAction,
                )
                .with_description("Start a countdown timer")
                .with_icon("alarm"),
            );
        }

        // Stopwatch controls
        if self.stopwatch.is_some() {
            let elapsed = self.get_stopwatch_elapsed();
            let status = if self.stopwatch_paused {
                "Paused"
            } else {
                "Running"
            };

            items.push(
                Item::new(
                    "timer:stopwatch:toggle",
                    format!("Stopwatch: {} ({})", Self::format_duration(elapsed), status),
                    ItemType::Timer,
                )
                .with_description(if self.stopwatch_paused {
                    "Click to resume"
                } else {
                    "Click to pause"
                })
                .with_icon("chronometer"),
            );

            items.push(
                Item::new("timer:stopwatch:reset", "Reset Stopwatch", ItemType::TimerAction)
                    .with_description("Stop and reset the stopwatch")
                    .with_icon("view-refresh"),
            );
        } else {
            items.push(
                Item::new("timer:stopwatch:start", "Start Stopwatch", ItemType::TimerAction)
                    .with_description("Start a new stopwatch")
                    .with_icon("chronometer"),
            );
        }

        // Active timers
        for timer in &self.timers {
            let remaining = timer.remaining();
            let status = if timer.paused {
                "Paused"
            } else if timer.is_finished() {
                "Finished!"
            } else {
                "Running"
            };

            let mut item = Item::new(
                format!("timer:{}", timer.id),
                format!("{}: {} ({})", timer.name, Self::format_duration(remaining), status),
                ItemType::Timer,
            )
            .with_description(if timer.paused {
                "Click to resume"
            } else {
                "Click to pause"
            })
            .with_icon(if timer.is_finished() {
                "dialog-warning"
            } else {
                "alarm"
            });

            item.metadata.duration = Some(timer.duration.as_secs());
            item.metadata.remaining = Some(remaining.as_secs());

            items.push(item);

            // Add cancel action
            items.push(
                Item::new(
                    format!("timer:cancel:{}", timer.id),
                    format!("Cancel: {}", timer.name),
                    ItemType::TimerAction,
                )
                .with_description("Remove this timer")
                .with_icon("process-stop"),
            );
        }

        // Common presets if no query
        if query.is_empty() && self.timers.is_empty() {
            items.push(
                Item::new("timer:preset:5m", "5 minute timer", ItemType::TimerAction)
                    .with_description("Start a 5 minute countdown")
                    .with_icon("alarm"),
            );
            items.push(
                Item::new("timer:preset:10m", "10 minute timer", ItemType::TimerAction)
                    .with_description("Start a 10 minute countdown")
                    .with_icon("alarm"),
            );
            items.push(
                Item::new("timer:preset:25m", "25 minute timer (Pomodoro)", ItemType::TimerAction)
                    .with_description("Start a 25 minute Pomodoro session")
                    .with_icon("alarm"),
            );
        }

        items
    }

    pub fn tick(&mut self) {
        // Check for finished timers and send notifications
        for timer in &self.timers {
            if timer.is_finished() {
                let _ = notify_rust::Notification::new()
                    .summary("Timer Finished")
                    .body(&format!("{} has completed!", timer.name))
                    .show();
            }
        }

        // Remove finished timers
        self.timers.retain(|t| !t.is_finished());
    }

    pub fn start_timer(&mut self, duration: Duration, name: Option<&str>) {
        let id = uuid::Uuid::new_v4().to_string();
        let timer = Timer {
            id,
            name: name
                .unwrap_or(&Self::format_duration(duration))
                .to_string(),
            duration,
            started_at: Local::now(),
            paused: false,
            paused_remaining: None,
        };
        self.timers.push(timer);
    }

    pub fn toggle_timer(&mut self, id: &str) {
        if let Some(timer) = self.timers.iter_mut().find(|t| t.id == id) {
            if timer.paused {
                // Resume
                let remaining = timer.paused_remaining.unwrap_or(Duration::ZERO);
                timer.started_at = Local::now();
                timer.duration = remaining;
                timer.paused = false;
                timer.paused_remaining = None;
            } else {
                // Pause
                timer.paused_remaining = Some(timer.remaining());
                timer.paused = true;
            }
        }
    }

    pub fn cancel_timer(&mut self, id: &str) {
        self.timers.retain(|t| t.id != id);
    }

    pub fn start_stopwatch(&mut self) {
        self.stopwatch = Some(Local::now());
        self.stopwatch_paused = false;
        self.stopwatch_elapsed = Duration::ZERO;
    }

    pub fn toggle_stopwatch(&mut self) {
        if self.stopwatch_paused {
            // Resume
            self.stopwatch = Some(Local::now());
            self.stopwatch_paused = false;
        } else if self.stopwatch.is_some() {
            // Pause
            self.stopwatch_elapsed = self.get_stopwatch_elapsed();
            self.stopwatch_paused = true;
        }
    }

    pub fn reset_stopwatch(&mut self) {
        self.stopwatch = None;
        self.stopwatch_paused = false;
        self.stopwatch_elapsed = Duration::ZERO;
    }

    fn get_stopwatch_elapsed(&self) -> Duration {
        if self.stopwatch_paused {
            self.stopwatch_elapsed
        } else if let Some(started) = self.stopwatch {
            let current = Local::now()
                .signed_duration_since(started)
                .to_std()
                .unwrap_or(Duration::ZERO);
            self.stopwatch_elapsed + current
        } else {
            Duration::ZERO
        }
    }

    pub fn execute_action(&mut self, action_id: &str) {
        if action_id.starts_with("timer:start:") {
            if let Ok(secs) = action_id.strip_prefix("timer:start:").unwrap().parse::<u64>() {
                self.start_timer(Duration::from_secs(secs), None);
            }
        } else if action_id.starts_with("timer:cancel:") {
            let id = action_id.strip_prefix("timer:cancel:").unwrap();
            self.cancel_timer(id);
        } else if action_id.starts_with("timer:") && !action_id.contains(':') {
            // Toggle timer
            let id = action_id.strip_prefix("timer:").unwrap();
            self.toggle_timer(id);
        } else {
            match action_id {
                "timer:stopwatch:start" => self.start_stopwatch(),
                "timer:stopwatch:toggle" => self.toggle_stopwatch(),
                "timer:stopwatch:reset" => self.reset_stopwatch(),
                "timer:preset:5m" => self.start_timer(Duration::from_secs(5 * 60), Some("5 minutes")),
                "timer:preset:10m" => {
                    self.start_timer(Duration::from_secs(10 * 60), Some("10 minutes"))
                }
                "timer:preset:25m" => {
                    self.start_timer(Duration::from_secs(25 * 60), Some("Pomodoro"))
                }
                _ => {}
            }
        }
    }

    fn parse_duration(input: &str) -> Option<Duration> {
        let input = input.trim().to_lowercase();
        if input.is_empty() {
            return None;
        }

        // Try parsing different formats
        // "5m", "10m30s", "1h", "90s", "1:30", "01:30:00"

        // Format: Xh Xm Xs
        let mut total_secs = 0u64;
        let mut current_num = String::new();

        for c in input.chars() {
            if c.is_ascii_digit() {
                current_num.push(c);
            } else if !current_num.is_empty() {
                let num: u64 = current_num.parse().ok()?;
                match c {
                    'h' => total_secs += num * 3600,
                    'm' => total_secs += num * 60,
                    's' => total_secs += num,
                    ':' => {
                        // Will be handled below
                        break;
                    }
                    _ => return None,
                }
                current_num.clear();
            }
        }

        // Handle remaining number (e.g., "5" means 5 minutes)
        if !current_num.is_empty() && total_secs == 0 {
            let num: u64 = current_num.parse().ok()?;
            // If contains ':', parse as time format
            if input.contains(':') {
                let parts: Vec<&str> = input.split(':').collect();
                match parts.len() {
                    2 => {
                        // MM:SS
                        let mins: u64 = parts[0].parse().ok()?;
                        let secs: u64 = parts[1].parse().ok()?;
                        total_secs = mins * 60 + secs;
                    }
                    3 => {
                        // HH:MM:SS
                        let hours: u64 = parts[0].parse().ok()?;
                        let mins: u64 = parts[1].parse().ok()?;
                        let secs: u64 = parts[2].parse().ok()?;
                        total_secs = hours * 3600 + mins * 60 + secs;
                    }
                    _ => return None,
                }
            } else {
                // Default to minutes
                total_secs = num * 60;
            }
        }

        if total_secs > 0 {
            Some(Duration::from_secs(total_secs))
        } else {
            None
        }
    }

    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}
