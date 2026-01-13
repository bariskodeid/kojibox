use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::{engine::general_purpose, Engine as _};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
struct SecretsFile {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub entries: HashMap<String, SecretEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SecretEntry {
    pub nonce: String,
    pub value: String,
}

#[derive(Debug)]
pub struct SecretsStore {
    key: [u8; 32],
    root: PathBuf,
    in_memory: bool,
}

impl SecretsStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, String> {
        let root = root.into();
        let key = load_or_create_key(&root)?;
        Ok(Self {
            root,
            key,
            in_memory: false,
        })
    }

    pub fn new_in_memory() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self {
            root: PathBuf::from("."),
            key,
            in_memory: true,
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let store = self.load_store().ok()?;
        let entry = store.entries.get(key)?;
        decrypt_value(&self.key, entry).ok()
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        let mut store = self.load_store()?;
        let entry = encrypt_value(&self.key, value)?;
        store.entries.insert(key.to_string(), entry);
        self.save_store(&store)
    }

    fn store_path(&self) -> PathBuf {
        self.root.join("app/config/secrets.json")
    }

    fn load_store(&self) -> Result<SecretsFile, String> {
        if self.in_memory {
            return Ok(SecretsFile::default());
        }
        let path = self.store_path();
        if !path.exists() {
            return Ok(SecretsFile {
                schema_version: 1,
                entries: HashMap::new(),
            });
        }
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let store: SecretsFile = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        Ok(store)
    }

    fn save_store(&self, store: &SecretsFile) -> Result<(), String> {
        if self.in_memory {
            return Ok(());
        }
        let path = self.store_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let raw = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
        fs::write(path, raw).map_err(|e| e.to_string())
    }
}

fn load_or_create_key(root: &PathBuf) -> Result<[u8; 32], String> {
    let path = root.join("app/config/secrets.key");
    if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let bytes = hex::decode(raw.trim()).map_err(|e| e.to_string())?;
        if bytes.len() != 32 {
            return Err("invalid secrets key length".to_string());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        return Ok(key);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    fs::write(&path, hex::encode(key)).map_err(|e| e.to_string())?;
    Ok(key)
}

fn encrypt_value(key: &[u8; 32], value: &str) -> Result<SecretEntry, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, value.as_bytes())
        .map_err(|_| "encrypt failed".to_string())?;
    Ok(SecretEntry {
        nonce: general_purpose::STANDARD.encode(nonce_bytes),
        value: general_purpose::STANDARD.encode(ciphertext),
    })
}

fn decrypt_value(key: &[u8; 32], entry: &SecretEntry) -> Result<String, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes = general_purpose::STANDARD
        .decode(&entry.nonce)
        .map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = general_purpose::STANDARD
        .decode(&entry.value)
        .map_err(|e| e.to_string())?;
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "decrypt failed".to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
