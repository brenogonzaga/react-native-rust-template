#!/usr/bin/env node
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");

const action = process.argv[2] || "setup";
const isArm64 = os.arch() === "arm64";
const isMac = os.platform() === "darwin";

const ROOT_DIR = path.resolve(__dirname, "..");
const RUST_CORE_DIR = path.join(ROOT_DIR, "rust-core");
const IOS_LIB_DIR = path.join(ROOT_DIR, "modules", "rust-bridge", "ios", "lib");
const ANDROID_JNILIBS_DIR = path.join(ROOT_DIR, "modules", "rust-bridge", "android", "src", "main", "jniLibs");

function run(cmd, cwd = ROOT_DIR, env = {}) {
  console.log(`\x1b[36m> [setup] ${cmd}\x1b[0m`);
  try {
    execSync(cmd, { cwd, stdio: [0, 1, 1], env: { ...process.env, ...env } });
  } catch (err) {
    console.error(`\x1b[31m[setup] Command failed: ${cmd}\x1b[0m`);
    process.exit(1);
  }
}

function ensureDir(dir) {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
}

switch (action) {
  case "setup": {
    const targets = ["aarch64-linux-android", "armv7-linux-androideabi", "x86_64-linux-android"];
    if (isMac) targets.push("aarch64-apple-ios-sim", "x86_64-apple-ios", "aarch64-apple-ios");
    run(`rustup target add ${targets.join(" ")}`);
    run("npm install");
    break;
  }
  case "ios": {
    if (!isMac) break;
    const target = isArm64 ? "aarch64-apple-ios-sim" : "x86_64-apple-ios";
    run(`cargo build --release -p rust-core --target ${target}`, ROOT_DIR, { IPHONEOS_DEPLOYMENT_TARGET: "16.0" });
    ensureDir(IOS_LIB_DIR);
    fs.copyFileSync(
      path.join(ROOT_DIR, "target", target, "release", "librust_bridge.a"),
      path.join(IOS_LIB_DIR, "librust_bridge.a")
    );
    break;
  }
  case "ios-prod": {
    if (!isMac) break;
    run(`cargo build --release -p rust-core --target aarch64-apple-ios`, ROOT_DIR, { IPHONEOS_DEPLOYMENT_TARGET: "16.0" });
    ensureDir(IOS_LIB_DIR);
    fs.copyFileSync(
      path.join(ROOT_DIR, "target", "aarch64-apple-ios", "release", "librust_bridge.a"),
      path.join(IOS_LIB_DIR, "librust_bridge.a")
    );
    break;
  }
  case "android": {
    ensureDir(ANDROID_JNILIBS_DIR);
    const outDir = path.relative(RUST_CORE_DIR, ANDROID_JNILIBS_DIR).replace(/\\/g, "/");
    run(`cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ${outDir} build --release`, RUST_CORE_DIR);
    break;
  }
}
