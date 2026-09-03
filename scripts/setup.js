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
const ANDROID_JNILIBS_DIR = path.join(
  ROOT_DIR,
  "modules",
  "rust-bridge",
  "android",
  "src",
  "main",
  "jniLibs",
);

// Must stay in sync with `ndk { abiFilters }` in the module's build.gradle.
const ANDROID_ABIS = ["arm64-v8a", "armeabi-v7a", "x86_64"];

function run(cmd, cwd = ROOT_DIR, env = {}) {
  console.log(`\x1b[36m> [setup] ${cmd}\x1b[0m`);
  try {
    execSync(cmd, { cwd, stdio: [0, 1, 1], env: { ...process.env, ...env } });
  } catch {
    console.error(`\x1b[31m[setup] Command failed: ${cmd}\x1b[0m`);
    process.exit(1);
  }
}

function ensureDir(dir) {
  if (!fs.existsSync(dir)) fs.mkdirSync(dir, { recursive: true });
}

// Fail loudly here rather than at link time in Xcode / Gradle, where a missing
// slice surfaces as an unrelated "undefined symbol _call_rust".
function copyIosLib(target) {
  const src = path.join(
    ROOT_DIR,
    "target",
    target,
    "release",
    "librust_bridge.a",
  );
  if (!fs.existsSync(src)) {
    console.error(
      `\x1b[31m[setup] Missing compiled iOS library: ${src}\x1b[0m`,
    );
    process.exit(1);
  }
  ensureDir(IOS_LIB_DIR);
  fs.copyFileSync(src, path.join(IOS_LIB_DIR, "librust_bridge.a"));
}

function verifyAndroidAbis() {
  const missing = ANDROID_ABIS.filter(
    (abi) =>
      !fs.existsSync(path.join(ANDROID_JNILIBS_DIR, abi, "librust_bridge.so")),
  );
  if (missing.length > 0) {
    console.error(
      `\x1b[31m[setup] Missing compiled Android libraries for ABIs: ${missing.join(", ")}\x1b[0m`,
    );
    process.exit(1);
  }
}

switch (action) {
  case "setup": {
    const targets = [
      "aarch64-linux-android",
      "armv7-linux-androideabi",
      "x86_64-linux-android",
    ];
    if (isMac)
      targets.push(
        "aarch64-apple-ios-sim",
        "x86_64-apple-ios",
        "aarch64-apple-ios",
      );
    run(`rustup target add ${targets.join(" ")}`);
    run("npm install");
    break;
  }
  case "ios": {
    if (!isMac) break;
    const target = isArm64 ? "aarch64-apple-ios-sim" : "x86_64-apple-ios";
    run(`cargo build --release -p rust-core --target ${target}`, ROOT_DIR, {
      IPHONEOS_DEPLOYMENT_TARGET: "16.0",
    });
    copyIosLib(target);
    break;
  }
  case "ios-prod": {
    if (!isMac) break;
    run(
      `cargo build --release -p rust-core --target aarch64-apple-ios`,
      ROOT_DIR,
      { IPHONEOS_DEPLOYMENT_TARGET: "16.0" },
    );
    copyIosLib("aarch64-apple-ios");
    break;
  }
  case "android": {
    ensureDir(ANDROID_JNILIBS_DIR);
    const outDir = path
      .relative(RUST_CORE_DIR, ANDROID_JNILIBS_DIR)
      .replace(/\\/g, "/");
    run(
      `cargo ndk ${ANDROID_ABIS.map((abi) => `-t ${abi}`).join(" ")} -o ${outDir} build --release`,
      RUST_CORE_DIR,
    );
    verifyAndroidAbis();
    break;
  }
}
