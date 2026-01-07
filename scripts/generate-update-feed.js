#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
const crypto = require("crypto");

function readFileSha256(filePath) {
  const data = fs.readFileSync(filePath);
  const hash = crypto.createHash("sha256").update(data).digest("hex");
  return `sha256:${hash}`;
}

function walk(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...walk(full));
    } else {
      files.push(full);
    }
  }
  return files;
}

function inferPlatform(filename) {
  const name = filename.toLowerCase();
  if (name.includes("windows") || name.endsWith(".msi") || name.endsWith(".exe") || name.endsWith(".msix")) {
    return { os: "windows", arch: "x64" };
  }
  if (name.includes("macos") || name.endsWith(".dmg") || name.endsWith(".app.tar.gz")) {
    return { os: "macos", arch: "arm64" };
  }
  if (name.includes("linux") || name.endsWith(".appimage") || name.endsWith(".deb") || name.endsWith(".rpm")) {
    return { os: "linux", arch: "x64" };
  }
  return null;
}

function main() {
  const args = process.argv.slice(2);
  const getArg = (flag) => {
    const idx = args.indexOf(flag);
    return idx >= 0 ? args[idx + 1] : null;
  };
  const version = getArg("--version");
  const channel = getArg("--channel");
  const distDir = getArg("--dist") || "dist";
  const outFile = getArg("--out") || path.join(distDir, "feed.json");
  const signKey = getArg("--sign-key");

  if (!version || !channel) {
    console.error("Missing --version or --channel");
    process.exit(1);
  }

  const files = walk(distDir).filter((f) => !f.endsWith("feed.json"));
  const platforms = [];

  for (const file of files) {
    const platform = inferPlatform(file);
    if (!platform) continue;
    const rel = path.relative(distDir, file).replace(/\\/g, "/");
    platforms.push({
      os: platform.os,
      arch: platform.arch,
      url: rel,
      checksum: readFileSha256(file),
    });
  }

  const feed = {
    version,
    pubDate: new Date().toISOString(),
    notes: `Release ${version} (${channel})`,
    platforms,
  };

  if (signKey) {
    const keyData = fs.readFileSync(signKey, "utf8").trim();
    const privateKey = Buffer.from(keyData, "base64");
    const payload = JSON.stringify(feed);
    const signature = crypto.sign(null, Buffer.from(payload), {
      key: privateKey,
      dsaEncoding: "ieee-p1363",
    });
    feed.signature = signature.toString("base64");
  }

  fs.writeFileSync(outFile, JSON.stringify(feed, null, 2));
  console.log(`Update feed written to ${outFile}`);
}

main();
