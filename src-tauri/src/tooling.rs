use rcgen::{Certificate, CertificateParams, SanType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainMapping {
    pub domain: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "targetPort")]
    pub target_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRule {
    pub host: String,
    pub path: String,
    pub target: String,
    pub tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertMeta {
    pub name: String,
    pub path: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustResult {
    pub applied: bool,
    pub command: String,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct DomainConfig {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub mappings: Vec<DomainMapping>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ProxyConfig {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub rules: Vec<ProxyRule>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct CertIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub certs: Vec<CertMeta>,
}

pub fn domains_list(root: &PathBuf) -> Result<Vec<DomainMapping>, String> {
    let config = load_domains(root)?;
    Ok(config.mappings)
}

pub fn domains_upsert(root: &PathBuf, mapping: DomainMapping) -> Result<(), String> {
    let mut config = load_domains(root)?;
    if let Some(existing) = config
        .mappings
        .iter_mut()
        .find(|item| item.domain == mapping.domain)
    {
        *existing = mapping;
    } else {
        config.mappings.push(mapping);
    }
    save_domains(root, &config)
}

pub fn domains_remove(root: &PathBuf, domain: &str) -> Result<(), String> {
    let mut config = load_domains(root)?;
    config.mappings.retain(|item| item.domain != domain);
    save_domains(root, &config)
}

pub fn proxy_rules(root: &PathBuf) -> Result<Vec<ProxyRule>, String> {
    let config = load_proxy(root)?;
    Ok(config.rules)
}

pub fn proxy_apply(root: &PathBuf, rules: Vec<ProxyRule>) -> Result<(), String> {
    let mut config = load_proxy(root)?;
    config.rules = rules;
    save_proxy(root, &config)
}

pub fn hosts_apply(root: &PathBuf, mappings: Vec<DomainMapping>) -> Result<(), String> {
    let hosts_path = hosts_path()?;
    let backup_path = root.join("app/config/hosts.backup");
    if !backup_path.exists() {
        if let Some(parent) = backup_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(&hosts_path, &backup_path).map_err(|e| e.to_string())?;
    }
    let content = fs::read_to_string(&hosts_path).map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    remove_hosts_block(&mut lines);
    let mut block = Vec::new();
    block.push("# kojibox start".to_string());
    for mapping in mappings {
        block.push(format!("127.0.0.1 {}", mapping.domain));
    }
    block.push("# kojibox end".to_string());
    lines.extend(block);
    let updated = lines.join("\n") + "\n";
    fs::write(&hosts_path, updated).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn hosts_rollback(root: &PathBuf) -> Result<(), String> {
    let backup_path = root.join("app/config/hosts.backup");
    if !backup_path.exists() {
        return Err("hosts backup not found".to_string());
    }
    let hosts_path = hosts_path()?;
    fs::copy(&backup_path, &hosts_path).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn certs_generate(root: &PathBuf, domains: Vec<String>) -> Result<CertMeta, String> {
    if domains.is_empty() {
        return Err("domains required".to_string());
    }
    let mut params = CertificateParams::default();
    params.subject_alt_names = domains
        .iter()
        .map(|domain| SanType::DnsName(domain.clone()))
        .collect();
    let cert = Certificate::from_params(params).map_err(|e| e.to_string())?;
    let cert_pem = cert.serialize_pem().map_err(|e| e.to_string())?;
    let key_pem = cert.serialize_private_key_pem();

    let cert_dir = root.join("app/certs");
    fs::create_dir_all(&cert_dir).map_err(|e| e.to_string())?;
    let name = domains[0].replace('.', "-");
    let cert_path = cert_dir.join(format!("{name}.crt"));
    let key_path = cert_dir.join(format!("{name}.key"));
    fs::write(&cert_path, cert_pem).map_err(|e| e.to_string())?;
    fs::write(&key_path, key_pem).map_err(|e| e.to_string())?;

    let meta = CertMeta {
        name,
        path: cert_path.to_string_lossy().to_string(),
        expires_at: expires_in_days(365),
    };
    update_cert_index(root, meta.clone())?;
    Ok(meta)
}

pub fn certs_list(root: &PathBuf) -> Result<Vec<CertMeta>, String> {
    let path = root.join("app/certs/index.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let index: CertIndex = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(index.certs)
}

pub fn certs_trust(root: &PathBuf, cert_path: &str) -> Result<String, String> {
    let instructions = root.join("app/certs/trust-instructions.txt");
    if let Some(parent) = instructions.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let text = format!(
        "Trust this certificate manually in your OS keychain: {}",
        cert_path
    );
    fs::write(&instructions, text).map_err(|e| e.to_string())?;
    Ok(instructions.to_string_lossy().to_string())
}

pub fn certs_trust_os(cert_path: &str, apply: bool) -> TrustResult {
    if cfg!(target_os = "windows") {
        let command = format!("certutil -addstore -f \"Root\" \"{cert_path}\"");
        if apply {
            let result = run_command("certutil", &["-addstore", "-f", "Root", cert_path]);
            return TrustResult {
                applied: result.is_ok(),
                command,
                notes: vec!["Requires Administrator privileges.".to_string()],
                error: result.err(),
            };
        }
        return TrustResult {
            applied: false,
            command,
            notes: vec!["Run in an elevated PowerShell prompt.".to_string()],
            error: None,
        };
    }

    if cfg!(target_os = "macos") {
        let command = format!(
            "sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain \"{cert_path}\""
        );
        let osa_command = format!(
            "do shell script \"security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain \\\"{}\\\"\" with administrator privileges",
            cert_path
        );
        if apply {
            let result = run_command(
                "osascript",
                &[
                    "-e",
                    &osa_command,
                ],
            );
            return TrustResult {
                applied: result.is_ok(),
                command,
                notes: vec!["Triggered macOS native password prompt.".to_string()],
                error: result.err(),
            };
        }
        return TrustResult {
            applied: false,
            command,
            notes: vec!["Run the displayed command in a Terminal or use 'Apply Now' to see a password prompt.".to_string()],
            error: None,
        };
    }

    let (command, notes) = linux_trust_command(cert_path);
    if apply {
        let result = run_shell_command(&command);
        return TrustResult {
            applied: result.is_ok(),
            command,
            notes,
            error: result.err(),
        };
    }
    TrustResult {
        applied: false,
        command,
        notes,
        error: None,
    }
}

fn linux_trust_command(cert_path: &str) -> (String, Vec<String>) {
    let update_ca = which("update-ca-certificates");
    if update_ca.is_some() {
        let command = format!(
            "sudo cp \"{cert_path}\" /usr/local/share/ca-certificates/kojibox.crt && sudo update-ca-certificates"
        );
        return (
            command,
            vec!["Requires sudo privileges.".to_string()],
        );
    }
    let update_trust = which("update-ca-trust");
    if update_trust.is_some() {
        let command = format!(
            "sudo cp \"{cert_path}\" /etc/pki/ca-trust/source/anchors/kojibox.crt && sudo update-ca-trust extract"
        );
        return (
            command,
            vec!["Requires sudo privileges.".to_string()],
        );
    }
    (
        format!(
            "sudo cp \"{cert_path}\" /usr/local/share/ca-certificates/kojibox.crt && sudo update-ca-certificates"
        ),
        vec![
            "Could not detect update-ca-certificates or update-ca-trust.".to_string(),
            "Install ca-certificates tooling and rerun.".to_string(),
        ],
    )
}

fn which(command: &str) -> Option<String> {
    std::process::Command::new("which")
        .arg(command)
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                None
            }
        })
}

fn run_command(cmd: &str, args: &[&str]) -> Result<(), String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn run_shell_command(command: &str) -> Result<(), String> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn load_domains(root: &PathBuf) -> Result<DomainConfig, String> {
    let path = root.join("app/config/domains.json");
    if !path.exists() {
        return Ok(DomainConfig {
            schema_version: 1,
            mappings: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_domains(root: &PathBuf, config: &DomainConfig) -> Result<(), String> {
    let path = root.join("app/config/domains.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())
}

fn load_proxy(root: &PathBuf) -> Result<ProxyConfig, String> {
    let path = root.join("app/config/proxy.json");
    if !path.exists() {
        return Ok(ProxyConfig {
            schema_version: 1,
            rules: Vec::new(),
        });
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn save_proxy(root: &PathBuf, config: &ProxyConfig) -> Result<(), String> {
    let path = root.join("app/config/proxy.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())
}

fn update_cert_index(root: &PathBuf, meta: CertMeta) -> Result<(), String> {
    let path = root.join("app/certs/index.json");
    let mut index = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str::<CertIndex>(&raw).map_err(|e| e.to_string())?
    } else {
        CertIndex {
            schema_version: 1,
            certs: Vec::new(),
        }
    };
    index.certs.push(meta);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?;
    fs::write(&path, raw).map_err(|e| e.to_string())
}

fn hosts_path() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        Ok(PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts"))
    } else {
        Ok(PathBuf::from("/etc/hosts"))
    }
}

fn remove_hosts_block(lines: &mut Vec<String>) {
    let start = lines.iter().position(|line| line.contains("# kojibox start"));
    let end = lines.iter().position(|line| line.contains("# kojibox end"));
    if let (Some(start), Some(end)) = (start, end) {
        if end >= start {
            lines.drain(start..=end);
        }
    }
}

fn expires_in_days(days: u64) -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let expires = secs + days * 24 * 60 * 60;
    expires.to_string()
}
