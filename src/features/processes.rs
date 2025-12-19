use crate::core::{Item, ItemType};
use sysinfo::{System, ProcessesToUpdate};
use std::process::Command;

pub struct ProcessManager {
    system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_processes(ProcessesToUpdate::All, true);
    }

    pub fn get_items(&mut self, query: &str) -> Vec<Item> {
        self.refresh();

        let query = query.to_lowercase();
        let mut items: Vec<Item> = self
            .system
            .processes()
            .iter()
            .filter(|(_, process)| {
                let name = process.name().to_string_lossy().to_lowercase();
                query.is_empty() || name.contains(&query)
            })
            .map(|(pid, process)| {
                let name = process.name().to_string_lossy().to_string();
                let cpu = process.cpu_usage();
                let memory = process.memory() as f32 / 1024.0 / 1024.0; // MB

                let mut item = Item::new(
                    format!("process:{}", pid),
                    &name,
                    ItemType::Process,
                )
                .with_description(format!(
                    "PID: {} | CPU: {:.1}% | Mem: {:.1} MB",
                    pid, cpu, memory
                ))
                .with_icon("application-x-executable");

                item.metadata.pid = Some(pid.as_u32());
                item.metadata.cpu = Some(cpu);
                item.metadata.memory = Some(memory);
                item
            })
            .collect();

        // Sort by CPU usage
        items.sort_by(|a, b| {
            let cpu_a = a.metadata.cpu.unwrap_or(0.0);
            let cpu_b = b.metadata.cpu.unwrap_or(0.0);
            cpu_b.partial_cmp(&cpu_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Limit to top 50
        items.truncate(50);
        items
    }

    pub fn kill_process(&self, pid: u32) {
        let _ = Command::new("kill")
            .arg("-9")
            .arg(pid.to_string())
            .output();
    }

    pub fn kill_process_graceful(&self, pid: u32) {
        let _ = Command::new("kill")
            .arg("-15")
            .arg(pid.to_string())
            .output();
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
