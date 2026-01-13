#!/usr/bin/env node
const fs = require("fs");
const path = require("path");

const ROOT = path.join(__dirname, "..");
const DIST_DIR = path.join(ROOT, "dist");
const RUNTIME_DIR = path.join(ROOT, "runtime");

function formatSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + " KB";
  return (bytes / (1024 * 1024)).toFixed(2) + " MB";
}

function getDirSize(dir) {
  if (!fs.existsSync(dir)) return 0;
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  let total = 0;
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      total += getDirSize(full);
    } else {
      total += fs.statSync(full).size;
    }
  }
  return total;
}

function main() {
  console.log("--- Bundle Size Audit ---");
  
  const frontendSize = getDirSize(DIST_DIR);
  console.log(`Frontend Assets: ${formatSize(frontendSize)}`);
  
  const runtimeSize = getDirSize(RUNTIME_DIR);
  console.log(`Runtime Binaries: ${formatSize(runtimeSize)}`);
  
  const totalRaw = frontendSize + runtimeSize;
  console.log(`Estimated Resource Payload: ${formatSize(totalRaw)}`);
  
  if (runtimeSize > 200 * 1024 * 1024) {
      console.warn("WARNING: Runtime size exceeds 200MB. Consider using dynamic download for database binaries.");
  }

  // Check specific large directories
  const binDir = path.join(RUNTIME_DIR, "bin");
  if (fs.existsSync(binDir)) {
      const bins = fs.readdirSync(binDir);
      for (const bin of bins) {
          const size = getDirSize(path.join(binDir, bin));
          console.log(`  - ${bin}: ${formatSize(size)}`);
      }
  }
}

main();