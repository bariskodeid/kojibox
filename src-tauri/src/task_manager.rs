use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::runtime::RuntimeManager;

#[derive(Debug)]
pub struct TaskManager {
    runtime: RuntimeManager,
    processes: HashMap<String, std::process::Child>,
}

impl TaskManager {
    pub fn new(runtime: RuntimeManager) -> Self {
        Self {
            runtime,
            processes: HashMap::new(),
        }
    }

    pub fn list_scripts(&self, project_path: &str) -> Result<HashMap<String, String>, String> {
        let pkg_json_path = PathBuf::from(project_path).join("package.json");
        if !pkg_json_path.exists() {
            return Err("package.json not found".to_string());
        }
        let raw = std::fs::read_to_string(&pkg_json_path).map_err(|e| e.to_string())?;
        let json: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
        
        let scripts = json.get("scripts")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            })
            .unwrap_or_default();
            
        Ok(scripts)
    }

    pub fn run_script(&mut self, project_id: &str, project_path: &str, script: &str) -> Result<(), String> {
        if self.processes.contains_key(project_id) {
            return Err("Task already running for this project".to_string());
        }

        let node_bin = self.runtime.resolve_binary("node")
            .map_err(|_| "node binary not found".to_string())?;
        
        // We assume npm is near node or we use node to run npm cli.js if bundled?
        // Ideally we should bundle npm. Node usually comes with npm/npx symlinks.
        // In our portable setup, we might need to look for npm-cli.js or similar if not in PATH.
        // For this implementation, let's assume 'npm' is available in the same dir as 'node' executable
        // or we try to run `node_bin` path/to/npm/cli.js run script.
        // A simpler way for portable node is usually `npm` (batch/sh) next to node.exe.
        
        let npm_bin = node_bin.parent().unwrap().join(if cfg!(target_os="windows") { "npm.cmd" } else { "npm" });
        
        // Prepare PATH
        let path_env = self.runtime.scoped_path(&node_bin);

        let mut cmd = Command::new(&npm_bin);
        cmd.arg("run")
           .arg(script)
           .current_dir(project_path)
           .env("PATH", path_env)
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

        let child = cmd.spawn().map_err(|e| e.to_string())?;
        self.processes.insert(project_id.to_string(), child);
        Ok(())
    }

    pub fn stop_task(&mut self, project_id: &str) -> Result<(), String> {
        if let Some(mut child) = self.processes.remove(project_id) {
            child.kill().map_err(|e| e.to_string())?;
            return Ok(());
        }
        Err("No task running".to_string())
    }
}
