use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
pub struct UpdateProgress {
    pub phase: String,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatus {
    pub available: bool,
    pub version: String,
}

pub struct Updater;

impl Updater {
    pub fn new() -> Self {
        Self
    }

    pub fn check(&self) -> UpdateStatus {
        UpdateStatus {
            available: false,
            version: "".to_string(),
        }
    }

    pub fn apply(&self, progress: Arc<Mutex<UpdateProgress>>) {
        std::thread::spawn(move || {
            {
                let mut state = progress.lock().expect("update progress lock");
                state.phase = "downloading".to_string();
                state.progress = 0.0;
            }
            for i in 1..=10 {
                std::thread::sleep(std::time::Duration::from_millis(250));
                let mut state = progress.lock().expect("update progress lock");
                state.progress = i as f32 / 10.0;
                if i == 10 {
                    state.phase = "complete".to_string();
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            let mut state = progress.lock().expect("update progress lock");
            state.phase = "idle".to_string();
            state.progress = 0.0;
        });
    }
}

pub fn default_progress() -> UpdateProgress {
    UpdateProgress {
        phase: "idle".to_string(),
        progress: 0.0,
    }
}
