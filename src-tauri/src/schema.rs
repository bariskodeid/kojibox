use serde_json::Value;

pub fn validate_app_config(value: &Value) -> Result<(), String> {
    let schema_version = value
        .get("schemaVersion")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "schemaVersion is required".to_string())?;
    if schema_version != 1 {
        return Err("unsupported schemaVersion".to_string());
    }
    let install_path = value
        .get("installPath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "installPath is required".to_string())?;
    if install_path.trim().is_empty() {
        return Err("installPath is required".to_string());
    }

    let update_channel = value
        .get("updateChannel")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "updateChannel is required".to_string())?;
    if update_channel != "stable" && update_channel != "beta" {
        return Err("updateChannel must be stable or beta".to_string());
    }

    let update_feed = value
        .get("updateFeedUrl")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "updateFeedUrl is required".to_string())?;
    if !(update_feed.starts_with("http://") || update_feed.starts_with("https://")) {
        return Err("updateFeedUrl must be a valid URL".to_string());
    }

    Ok(())
}

pub fn validate_project_config(value: &Value) -> Result<(), String> {
    let schema_version = value
        .get("schemaVersion")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "schemaVersion is required".to_string())?;
    if schema_version != 1 {
        return Err("unsupported schemaVersion".to_string());
    }
    for field in ["id", "name", "path", "domain", "stack"] {
        let val = value
            .get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("{field} is required"))?;
        if val.trim().is_empty() {
            return Err(format!("{field} is required"));
        }
    }
    Ok(())
}

pub fn validate_port_registry(value: &Value) -> Result<(), String> {
    let schema_version = value
        .get("schemaVersion")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "schemaVersion is required".to_string())?;
    if schema_version != 1 {
        return Err("unsupported schemaVersion".to_string());
    }
    let ranges = value
        .get("ranges")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "ranges is required".to_string())?;
    for (key, range) in ranges {
        let from = range
            .get("from")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("invalid range for {key}"))?;
        let to = range
            .get("to")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| format!("invalid range for {key}"))?;
        if from == 0 || to == 0 || from > to {
            return Err(format!("invalid range for {key}"));
        }
    }
    Ok(())
}

pub fn validate_service_config(value: &Value) -> Result<(), String> {
    let schema_version = value
        .get("schemaVersion")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| "schemaVersion is required".to_string())?;
    if schema_version != 1 {
        return Err("unsupported schemaVersion".to_string());
    }
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "id is required".to_string())?;
    if id.trim().is_empty() {
        return Err("id is required".to_string());
    }
    let ports = value
        .get("ports")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "ports is required".to_string())?;
    for (key, port) in ports {
        let port = port.as_u64().ok_or_else(|| format!("invalid port for {key}"))?;
        if port > u16::MAX as u64 {
            return Err(format!("port out of range for {key}"));
        }
    }
    let env = value
        .get("env")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "env is required".to_string())?;
    for (key, value) in env {
        if !is_valid_env_key(key) {
            return Err(format!("invalid env key: {key}"));
        }
        let value = value
            .as_str()
            .ok_or_else(|| format!("invalid env value for {key}"))?;
        if value.trim().is_empty() {
            return Err(format!("env value required for {key}"));
        }
    }
    value
        .get("args")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "args is required".to_string())?;
    Ok(())
}

fn is_valid_env_key(key: &str) -> bool {
    let mut chars = key.chars();
    match chars.next() {
        Some(c) if c.is_ascii_uppercase() || c == '_' => {}
        _ => return false,
    }
    for c in chars {
        if !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_') {
            return false;
        }
    }
    true
}
