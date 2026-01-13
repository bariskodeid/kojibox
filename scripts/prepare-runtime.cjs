#!/usr/bin/env node
/**
 * scripts/prepare-runtime.js
 * Fetches and prepares runtime binaries for the current platform.
 */
const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const ROOT = path.join(__dirname, "..");
const RUNTIME_DIR = path.join(ROOT, "runtime");
const BIN_DIR = path.join(RUNTIME_DIR, "bin");
const CACHE_DIR = path.join(RUNTIME_DIR, "cache");

// Mock URLs - in a real scenario, these would point to actual binary distributions
// or a proxy/bucket managed by Kojibox.
const SOURCES = {
  php: "8.3.2",
  node: "20.11.1",
  postgres: "16.2",
  mariadb: "10.11.6",
  mailpit: "1.15.0",
};

function getPlatformTag() {
  const os = process.platform === "win32" ? "windows" : process.platform === "darwin" ? "macos" : "linux";
  const arch = process.arch === "x64" ? "x64" : "arm64";
  return `${os}-${arch}`;
}

function ensureDirs() {
  [RUNTIME_DIR, BIN_DIR, CACHE_DIR].forEach((d) => {
    if (!fs.existsSync(d)) fs.mkdirSync(d, { recursive: true });
  });
}

async function main() {
  console.log("Preparing runtime binaries...");
  ensureDirs();

  const platform = getPlatformTag();
  console.log(`Platform detected: ${platform}`);

  // This script currently just ensures the structure exists.
  // In a real implementation, it would download zip files for the current platform,
  // extract them to runtime/bin/{name}/{version}/{platform}/,
  // and update runtime/manifest.json.

  // For this prototype, we'll create dummy binaries if they don't exist
  // to allow Tauri bundling to succeed.
  for (const [name, version] of Object.entries(SOURCES)) {
    const targetDir = path.join(BIN_DIR, name, version, platform);
    if (!fs.existsSync(targetDir)) fs.mkdirSync(targetDir, { recursive: true });

    const ext = process.platform === "win32" ? ".exe" : "";
    const binFile = path.join(targetDir, `${name}${ext}`);

    if (!fs.existsSync(binFile)) {
      console.log(`Creating placeholder for ${name} v${version}`);
      // On real build, we'd download here.
      const isWin = process.platform === "win32";
      const scriptContent = isWin 
        ? `@echo off\r\necho Placeholder running for ${name}...\r\n:loop\r\ntimeout /t 5 >nul\r\ngoto loop`
        : `#!/bin/sh\necho "Placeholder running for ${name}..."\nwhile true; do sleep 5; done`;
      
      fs.writeFileSync(binFile, scriptContent);
      if (!isWin) {
          fs.chmodSync(binFile, 0o755);
      }
    }
  }

  console.log("Runtime preparation complete.");
}

main().catch(err => {
  console.error(err);
  process.exit(1);
});
