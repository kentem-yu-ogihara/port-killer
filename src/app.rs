use anyhow::Result;
use std::time::{Duration, Instant};

use crate::network::{get_listening_ports, kill_process, PortProcess};

pub enum ConfirmState {
    Waiting { pid: u32, port: u16, name: String },
}

pub struct App {
    pub connections: Vec<PortProcess>,
    pub selected: usize,
    pub confirm: Option<ConfirmState>,
    pub status_message: Option<String>,
    pub search_query: String,
    pub is_searching: bool,
    last_refresh: Instant,
}

impl App {
    pub fn new() -> Result<Self> {
        Ok(Self {
            connections: get_listening_ports()?,
            selected: 0,
            confirm: None,
            status_message: None,
            search_query: String::new(),
            is_searching: false,
            last_refresh: Instant::now(),
        })
    }

    pub fn filtered_connections(&self) -> Vec<&PortProcess> {
        if self.search_query.is_empty() {
            return self.connections.iter().collect();
        }
        let q = self.search_query.to_lowercase();
        self.connections
            .iter()
            .filter(|c| {
                c.port.to_string().contains(&q)
                    || c.process_name.to_lowercase().contains(&q)
                    || c.protocol.to_lowercase().contains(&q)
            })
            .collect()
    }

    pub fn select_next(&mut self) {
        let len = self.filtered_connections().len();
        if len == 0 {
            return;
        }
        self.selected = (self.selected + 1) % len;
    }

    pub fn select_prev(&mut self) {
        let len = self.filtered_connections().len();
        if len == 0 {
            return;
        }
        if self.selected == 0 {
            self.selected = len - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn request_kill(&mut self) {
        let filtered = self.filtered_connections();
        if let Some(conn) = filtered.get(self.selected) {
            self.confirm = Some(ConfirmState::Waiting {
                pid: conn.pid,
                port: conn.port,
                name: conn.process_name.clone(),
            });
        }
    }

    pub fn cancel_kill(&mut self) {
        self.confirm = None;
    }

    pub fn execute_kill(&mut self) -> Result<()> {
        if let Some(ConfirmState::Waiting { pid, port, name }) = self.confirm.take() {
            if kill_process(pid) {
                self.status_message =
                    Some(format!("Killed '{}' (PID {}) on port {}", name, pid, port));
            } else {
                self.status_message = Some(format!("Failed to kill '{}' (PID {})", name, pid));
            }
            self.refresh()?;
        }
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.connections = get_listening_ports()?;
        self.last_refresh = Instant::now();
        let filtered_len = self.filtered_connections().len();
        if filtered_len == 0 {
            self.selected = 0;
        } else if self.selected >= filtered_len {
            self.selected = filtered_len - 1;
        }
        Ok(())
    }

    pub fn should_auto_refresh(&self) -> bool {
        self.last_refresh.elapsed() > Duration::from_secs(5)
    }

    pub fn confirm_entry(&self) -> Option<(u16, u32, &str)> {
        if let Some(ConfirmState::Waiting { port, pid, name }) = &self.confirm {
            Some((*port, *pid, name.as_str()))
        } else {
            None
        }
    }

    pub fn enter_search(&mut self) {
        self.is_searching = true;
    }

    pub fn exit_search(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        self.selected = 0;
    }

    pub fn search_push(&mut self, c: char) {
        self.search_query.push(c);
        self.selected = 0;
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.selected = 0;
    }
}
