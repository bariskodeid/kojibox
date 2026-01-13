use std::path::PathBuf;
use std::process::Command;
use crate::runtime::RuntimeManager;

#[derive(Debug)]
pub struct DatabaseManager {
    runtime: RuntimeManager,
}

impl DatabaseManager {
    pub fn new(runtime: RuntimeManager) -> Self {
        Self { runtime }
    }

    pub fn dump(&self, service: &str, db_name: &str, output: PathBuf) -> Result<String, String> {
        let (bin, args) = match service {
            "postgres" => {
                let bin = self.runtime.resolve_binary("postgres")
                    .map_err(|_| "postgres binary not found".to_string())?;
                // Assume pg_dump is in same dir
                let pg_dump = bin.parent().unwrap().join(if cfg!(target_os="windows") { "pg_dump.exe" } else { "pg_dump" });
                if !pg_dump.exists() {
                    return Err("pg_dump not found".to_string());
                }
                // Port needs to be looked up from config/running service, but for now assume default or 5432
                // Ideally we get port from ServiceManager. 
                // For simplicity MVP, we assume local 5432 or 3306 based on service id.
                // Or better, passing connection string?
                // Let's use default ports for now as we don't have easy access to ConfigStore here without passing it.
                (pg_dump, vec![
                    "-h".to_string(), "127.0.0.1".to_string(),
                    "-p".to_string(), "5432".to_string(),
                    "-U".to_string(), "postgres".to_string(),
                    "-f".to_string(), output.to_string_lossy().to_string(),
                    db_name.to_string()
                ])
            },
            "mariadb" => {
                let bin = self.runtime.resolve_binary("mariadb")
                    .map_err(|_| "mariadb binary not found".to_string())?;
                let mysqldump = bin.parent().unwrap().join(if cfg!(target_os="windows") { "mysqldump.exe" } else { "mysqldump" });
                if !mysqldump.exists() {
                    return Err("mysqldump not found".to_string());
                }
                (mysqldump, vec![
                    "-h".to_string(), "127.0.0.1".to_string(),
                    "-P".to_string(), "3306".to_string(),
                    "-u".to_string(), "root".to_string(),
                    // Pass empty password? mysql usually needs -p with empty if socket auth not used
                    // output redirection > is shell feature. Command::output doesn't do >.
                    // mysqldump -r output_file
                    "--result-file".to_string(), output.to_string_lossy().to_string(),
                    db_name.to_string()
                ])
            },
            _ => return Err("unsupported service for dump".to_string()),
        };

        let output = Command::new(&bin)
            .args(&args)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok("Dump successful".to_string())
    }
}
