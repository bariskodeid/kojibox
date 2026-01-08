use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct InstallerStatus {
    pub phase: String,
    pub progress: f32,
}

#[derive(Debug)]
pub struct Installer {
    status: Arc<Mutex<InstallerStatus>>,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(InstallerStatus {
                phase: "idle".to_string(),
                progress: 0.0,
            })),
        }
    }

    pub fn status(&self) -> InstallerStatus {
        self.status.lock().expect("installer lock").clone()
    }

    pub fn start(&self) {
        let status = Arc::clone(&self.status);
        std::thread::spawn(move || {
            {
                let mut state = status.lock().expect("installer lock");
                state.phase = "downloading".to_string();
                state.progress = 0.0;
            }
            for i in 1..=10 {
                std::thread::sleep(std::time::Duration::from_millis(300));
                let mut state = status.lock().expect("installer lock");
                state.progress = i as f32 / 10.0;
                if i == 10 {
                    state.phase = "complete".to_string();
                }
            }
            let _ = init_runtime_layout(".");
        });
    }
}

fn init_runtime_layout(root: &str) -> Result<(), String> {
    let base = PathBuf::from(root);
    let runtime = base.join("runtime");
    let manifest = runtime.join("manifest.json");
    fs::create_dir_all(runtime.join("bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("data")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("logs")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("config")).map_err(|e| e.to_string())?;
    fs::create_dir_all(runtime.join("temp")).map_err(|e| e.to_string())?;

    if !manifest.exists() {
        let stub = serde_json::json!({
            "version": "1",
            "services": [],
            "bundle": {
                "createdAt": "0",
                "source": "installer",
                "signature": ""
            }
        });
        let raw = serde_json::to_string_pretty(&stub).map_err(|e| e.to_string())?;
        fs::write(manifest, raw).map_err(|e| e.to_string())?;
    }
    Ok(())
}
