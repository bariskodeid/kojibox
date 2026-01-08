use std::collections::HashMap;

#[derive(Default)]
pub struct SecretsStore {
    values: HashMap<String, String>,
}

impl SecretsStore {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.values.get(key).cloned()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }
}
