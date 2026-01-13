use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use once_cell::sync::Lazy;
use crate::runtime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub event: String,
    pub ts: String,
    #[serde(rename = "appVersion")]
    pub app_version: String,
    pub os: String,
    pub payload: serde_json::Value,
}

struct TelemetryState {
    queue: Vec<TelemetryEvent>,
    enabled: bool,
}

static STATE: Lazy<Mutex<TelemetryState>> = Lazy::new(|| {
    Mutex::new(TelemetryState {
        queue: Vec::new(),
        enabled: false,
    })
});

pub fn set_enabled(enabled: bool) {
    let mut state = STATE.lock().unwrap();
    state.enabled = enabled;
}

pub fn track_event(event_name: &str, payload: serde_json::Value) {
    let mut state = STATE.lock().unwrap();
    if !state.enabled {
        return;
    }

    let (os, _) = runtime::os_arch_tag();
    let ev = TelemetryEvent {
        event: event_name.to_string(),
        ts: now_iso(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os,
        payload,
    };

    println!("[Telemetry] Tracking: {} - {:?}", event_name, ev.payload);
    state.queue.push(ev);

    if state.queue.len() >= 10 {
        flush_internal(&mut state);
    }
}

#[allow(dead_code)]
pub fn flush() {
    let mut state = STATE.lock().unwrap();
    flush_internal(&mut state);
}

fn flush_internal(state: &mut TelemetryState) {
    if state.queue.is_empty() {
        return;
    }
    
    let _events = state.queue.drain(..).collect::<Vec<_>>();
    println!("[Telemetry] Flushed {} events", _events.len());
    
    // In a real app, we would send these to an endpoint:
    // let client = reqwest::blocking::Client::new();
    // let _ = client.post("https://telemetry.kojibox.dev/v1/batch")
    //     .json(&_events)
    //     .send();
}

fn now_iso() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple mock ISO string
    format!("{}-01-01T00:00:00Z", now) // In real app use chrono
}

// Helper tracking functions
pub fn track_app_start(channel: &str, services: Vec<String>) {
    track_event("app_start", serde_json::json!({
        "channel": channel,
        "servicesEnabled": services
    }));
}

pub fn track_service_start(service: &str, success: bool) {
    track_event("service_start", serde_json::json!({
        "service": service,
        "success": success
    }));
}

pub fn track_service_error(service: &str, error_code: &str) {
    track_event("service_error", serde_json::json!({
        "service": service,
        "errorCode": error_code
    }));
}
