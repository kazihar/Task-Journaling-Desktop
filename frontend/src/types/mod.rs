use std::fmt::Display;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JournalProps {
    pub id: String,
    pub title: Option<String>,
    pub body: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJournalRequest {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SystemInfo {
    pub disk_usage: Vec<DiskInfo>,
    pub current_user: String,
    pub top_cpu_processes: Vec<ProcessInfo>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub used_percentage: f64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
}

#[derive(Debug, Clone, Default)]
pub enum Screen {
    #[default]
    MainMenu,
    Create,
    SysInfo,
}

#[derive(Clone, Copy)]
pub struct CurrentScreen {
    pub screen: Signal<Screen>,
}
