use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, VerifyingKey};
use reqwest::blocking::Client;
use semver::Version;

use crate::config_store::AppConfig;
use crate::installer;

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

#[derive(Debug)]
pub struct Updater;

impl Updater {
    pub fn new() -> Self {
        Self
    }

    pub fn check(&self, app: &AppConfig) -> Result<UpdateStatus, String> {
        let feed = fetch_feed(&app.update_feed_url)?;
        verify_feed_signature(&feed, &app.update_public_keys)?;
        let latest = feed.version.clone();
        let current = Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or_else(|_| {
            Version::new(0, 0, 0)
        });
        let available = Version::parse(&latest)
            .map(|version| version > current)
            .unwrap_or_else(|_| latest != current.to_string());
        Ok(UpdateStatus {
            available,
            version: latest,
        })
    }

    #[allow(dead_code)]
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

    pub fn apply_with_archive(
        &self,
        progress: Arc<Mutex<UpdateProgress>>,
        root: PathBuf,
        archive: PathBuf,
        checksum: String,
    ) {
        std::thread::spawn(move || {
            {
                let mut state = progress.lock().expect("update progress lock");
                state.phase = "staging".to_string();
                state.progress = 0.0;
            }
            let result = installer::apply_archive_with_rollback(&root, &archive, &checksum);
            let mut state = progress.lock().expect("update progress lock");
            match result {
                Ok(_) => {
                    state.phase = "complete".to_string();
                    state.progress = 1.0;
                }
                Err(err) => {
                    state.phase = format!("error: {err}");
                    state.progress = 0.0;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
            state.phase = "idle".to_string();
            state.progress = 0.0;
        });
    }

    pub fn apply_latest(
        &self,
        progress: Arc<Mutex<UpdateProgress>>,
        app: AppConfig,
        root: PathBuf,
    ) {
        std::thread::spawn(move || {
            {
                let mut state = progress.lock().expect("update progress lock");
                state.phase = "fetching".to_string();
                state.progress = 0.0;
            }
            let result = fetch_feed(&app.update_feed_url)
                .and_then(|feed| {
                    verify_feed_signature(&feed, &app.update_public_keys)?;
                    let (os, arch) = current_platform();
                    let platform = feed
                        .platforms
                        .iter()
                        .find(|entry| entry.os == os && entry.arch == arch)
                        .ok_or_else(|| "no update for current platform".to_string())?;
                    let cache_dir = root.join("runtime/cache/updates");
                    std::fs::create_dir_all(&cache_dir).map_err(|e| e.to_string())?;
                    let archive_path = cache_dir.join(format!("update-{}.zip", feed.version));
                    installer::download_with_resume(&platform.url, &archive_path, &platform.checksum)?;
                    installer::apply_archive_with_rollback(&root, &archive_path, &platform.checksum)?;
                    Ok(())
                });
            let mut state = progress.lock().expect("update progress lock");
            match result {
                Ok(_) => {
                    state.phase = "complete".to_string();
                    state.progress = 1.0;
                }
                Err(err) => {
                    state.phase = format!("error: {err}");
                    state.progress = 0.0;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateFeed {
    pub version: String,
    #[serde(rename = "pubDate")]
    pub pub_date: Option<String>,
    pub notes: Option<String>,
    pub platforms: Vec<UpdatePlatform>,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdatePlatform {
    pub os: String,
    pub arch: String,
    pub url: String,
    pub checksum: String,
}

fn fetch_feed(url: &str) -> Result<UpdateFeed, String> {
    let client = Client::new();
    let response = client.get(url).send().map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("update feed error: {}", response.status()));
    }
    response.json::<UpdateFeed>().map_err(|e| e.to_string())
}

fn verify_feed_signature(feed: &UpdateFeed, public_keys: &[String]) -> Result<(), String> {
    let signature = feed
        .signature
        .clone()
        .ok_or_else(|| "missing update signature".to_string())?;
    let signature_bytes = general_purpose::STANDARD
        .decode(&signature)
        .map_err(|e| e.to_string())?;
    let signature = Signature::from_slice(&signature_bytes).map_err(|e| e.to_string())?;
    let payload_value = serde_json::to_value(feed).map_err(|e| e.to_string())?;
    let mut payload_map = payload_value
        .as_object()
        .ok_or_else(|| "invalid feed payload".to_string())?
        .clone();
    payload_map.remove("signature");
    let payload = serde_json::to_string(&payload_map).map_err(|e| e.to_string())?;

    for key_b64 in public_keys {
        let key_bytes = match general_purpose::STANDARD.decode(key_b64) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        let key_array: [u8; 32] = match key_bytes.try_into() {
            Ok(arr) => arr,
            Err(_) => continue,
        };
        if let Ok(key) = VerifyingKey::from_bytes(&key_array) {
            if key.verify_strict(payload.as_bytes(), &signature).is_ok() {
                return Ok(());
            }
        }
    }
    Err("update signature invalid".to_string())
}

fn current_platform() -> (String, String) {
    let os = match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        "linux" => "linux",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    (os.to_string(), arch.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use ed25519_dalek::Signer;
    use rand::rngs::OsRng;

    #[test]
    fn verify_feed_signature_ok() {
        let mut csprng = OsRng;
        let mut bytes = [0u8; 32];
        csprng.fill_bytes(&mut bytes);
        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();
        let public_key = general_purpose::STANDARD.encode(verifying_key.to_bytes());

        let mut feed = UpdateFeed {
            version: "1.2.3".to_string(),
            pub_date: Some("2025-01-01".to_string()),
            notes: Some("notes".to_string()),
            platforms: vec![UpdatePlatform {
                os: "linux".to_string(),
                arch: "x64".to_string(),
                url: "https://example.com/update.zip".to_string(),
                checksum: "sha256:abc".to_string(),
            }],
            signature: None,
        };

        let payload_value = serde_json::to_value(&feed).unwrap();
        let mut payload_map = payload_value.as_object().unwrap().clone();
        payload_map.remove("signature");
        let payload = serde_json::to_string(&payload_map).unwrap();
        let signature = signing_key.sign(payload.as_bytes());
        feed.signature = Some(general_purpose::STANDARD.encode(signature.to_bytes()));

        let result = verify_feed_signature(&feed, &[public_key]);
        assert!(result.is_ok());
    }
}
