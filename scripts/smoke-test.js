import { execSync } from "node:child_process";

execSync("cargo test --manifest-path src-tauri/Cargo.toml --test smoke", {
  stdio: "inherit",
});
