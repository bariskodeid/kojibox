use serde::Serialize;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::config_store::ConfigStore;

#[derive(Debug, Clone, Serialize)]
pub struct MetricsSnapshot {
    pub ts: String,
    #[serde(rename = "uptimeSec")]
    pub uptime_sec: u64,
    #[serde(rename = "portsInUse")]
    pub ports_in_use: Vec<u16>,
    #[serde(rename = "cpuPercent")]
    pub cpu_percent: f32,
    #[serde(rename = "memMB")]
    pub mem_mb: u64,
}

static START_TS: OnceLock<u64> = OnceLock::new();

pub fn init_start() {
    let _ = START_TS.set(now_secs());
}

pub fn snapshot(config: &ConfigStore) -> Result<MetricsSnapshot, String> {
    let start = *START_TS.get_or_init(now_secs);
    let now = now_secs();
    let uptime = now.saturating_sub(start);

    let mut system = System::new_with_specifics(
        RefreshKind::new().with_cpu(CpuRefreshKind::everything()).with_memory(MemoryRefreshKind::everything()),
    );
    system.refresh_cpu();
    system.refresh_memory();

    let cpu_percent = system.global_cpu_info().cpu_usage();
    let mem_mb = system.used_memory() / 1024;

    let registry = config.load_port_registry()?;
    let mut ports = registry.assigned.values().copied().collect::<Vec<_>>();
    ports.sort_unstable();
    ports.dedup();

    Ok(MetricsSnapshot {
        ts: now.to_string(),
        uptime_sec: uptime,
        ports_in_use: ports,
        cpu_percent,
        mem_mb,
    })
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
