#!/usr/bin/env node
const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const os = require("os");

const action = process.argv[2] || "setup";
const isMac = os.platform() === "darwin";

// `mobile-dev` (root Cargo.toml) for local iteration; full `release` (fat LTO)
// on EAS, or locally with `RUST_PROFILE=release npm run ios`.
const PROFILE =
  process.env.RUST_PROFILE ||
  (process.env.EAS_BUILD === "true" ? "release" : "mobile-dev");
// cargo names the output dir after the profile, except `dev` -> `debug`.
const PROFILE_DIR = PROFILE === "dev" ? "debug" : PROFILE;

// rustup installs into ~/.cargo/bin. On EAS the `eas` action below installs it
// in the pre-install hook, and that PATH change doesn't reach later steps
// (e.g. the config plugin calling back into this script during prebuild).
process.env.PATH = [
  path.join(os.homedir(), ".cargo", "bin"),
  process.env.PATH,
].join(path.delimiter);

const ROOT_DIR = path.resolve(__dirname, "..");
const RUST_CORE_DIR = path.join(ROOT_DIR, "rust-core");
const IOS_XCFRAMEWORK = path.join(
  ROOT_DIR,
  "modules",
  "rust-bridge",
  "ios",
  "RustBridge.xcframework",
);
const ANDROID_JNILIBS_DIR = path.join(
  ROOT_DIR,
  "modules",
  "rust-bridge",
  "android",
  "src",
  "main",
  "jniLibs",
);

// Must equal `platforms` in modules/rust-bridge/rust-bridge.podspec.
const IOS_DEPLOYMENT_TARGET = "16.4";
const IOS_DEVICE_TARGET = "aarch64-apple-ios";
const IOS_SIM_TARGETS = ["aarch64-apple-ios-sim", "x86_64-apple-ios"];

// Must stay in sync with `ndk { abiFilters }` in the module's build.gradle.
const ANDROID_ABIS = ["arm64-v8a", "armeabi-v7a", "x86_64"];
const ANDROID_TARGETS = [
  "aarch64-linux-android",
  "armv7-linux-androideabi",
  "x86_64-linux-android",
];

const q = (p) => JSON.stringify(p);

function run(cmd, cwd = ROOT_DIR, env = {}) {
  console.log(`\x1b[36m> [setup] ${cmd}\x1b[0m`);
  try {
    execSync(cmd, { cwd, stdio: [0, 1, 1], env: { ...process.env, ...env } });
  } catch {
    fail(`Command failed: ${cmd}`);
  }
}

function fail(msg) {
  console.error(`\x1b[31m[setup] ${msg}\x1b[0m`);
  process.exit(1);
}

function hasCommand(cmd) {
  try {
    execSync(`${cmd} --version`, { stdio: "ignore" });
    return true;
  } catch {
    return false;
  }
}

// Fail loudly here rather than at link time in Xcode / Gradle, where a missing
// library surfaces as an unrelated "undefined symbol _call_rust".
function requireFile(file) {
  if (!fs.existsSync(file)) fail(`Missing compiled Rust library: ${file}`);
  return file;
}

function buildIos() {
  // Local builds target the active simulator arch only (as `expo run:ios`
  // does); release builds also carry the other one, for generic destinations.
  const simTargets =
    PROFILE === "release"
      ? IOS_SIM_TARGETS
      : [os.arch() === "arm64" ? "aarch64-apple-ios-sim" : "x86_64-apple-ios"];
  const targets = [IOS_DEVICE_TARGET, ...simTargets];
  run(
    `cargo build -p rust-core --profile ${PROFILE} ${targets.map((t) => `--target ${t}`).join(" ")}`,
    ROOT_DIR,
    { IPHONEOS_DEPLOYMENT_TARGET: IOS_DEPLOYMENT_TARGET },
  );

  const lib = (target) =>
    requireFile(
      path.join(ROOT_DIR, "target", target, PROFILE_DIR, "librust_bridge.a"),
    );
  let simLib = lib(simTargets[0]);
  if (simTargets.length > 1) {
    // An xcframework holds one library per platform, so the simulator
    // architectures are merged into a single fat library first.
    simLib = path.join(
      ROOT_DIR,
      "target",
      "ios-sim-universal",
      PROFILE_DIR,
      "librust_bridge.a",
    );
    fs.mkdirSync(path.dirname(simLib), { recursive: true });
    run(
      `lipo -create ${simTargets.map((t) => q(lib(t))).join(" ")} -output ${q(simLib)}`,
    );
  }

  fs.rmSync(IOS_XCFRAMEWORK, { recursive: true, force: true });
  run(
    `xcodebuild -create-xcframework -library ${q(lib(IOS_DEVICE_TARGET))} -library ${q(simLib)} -output ${q(IOS_XCFRAMEWORK)}`,
  );
}

function buildAndroid() {
  // Cleared first so a build that silently skips an ABI can't leave a stale
  // .so from an older build behind for the check below to accept.
  for (const abi of ANDROID_ABIS) {
    fs.rmSync(path.join(ANDROID_JNILIBS_DIR, abi), {
      recursive: true,
      force: true,
    });
  }
  const outDir = path
    .relative(RUST_CORE_DIR, ANDROID_JNILIBS_DIR)
    .replace(/\\/g, "/");
  run(
    `cargo ndk ${ANDROID_ABIS.map((abi) => `-t ${abi}`).join(" ")} -o ${outDir} build --profile ${PROFILE}`,
    RUST_CORE_DIR,
  );
  for (const abi of ANDROID_ABIS) {
    requireFile(path.join(ANDROID_JNILIBS_DIR, abi, "librust_bridge.so"));
  }
}

switch (action) {
  case "setup": {
    const targets = isMac
      ? [...ANDROID_TARGETS, IOS_DEVICE_TARGET, ...IOS_SIM_TARGETS]
      : ANDROID_TARGETS;
    run(`rustup target add ${targets.join(" ")}`);
    if (!hasCommand("cargo ndk")) {
      console.warn(
        "\x1b[33m[setup] cargo-ndk not found; Android builds need it: cargo install cargo-ndk\x1b[0m",
      );
    }
    run("npm install");
    break;
  }
  // `eas-build-pre-install` hook: EAS images ship without a Rust toolchain.
  case "eas": {
    if (!hasCommand("rustup")) {
      run(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal",
      );
    }
    const platform = process.env.EAS_BUILD_PLATFORM;
    const targets =
      platform === "ios"
        ? [IOS_DEVICE_TARGET, ...IOS_SIM_TARGETS]
        : ANDROID_TARGETS;
    run(`rustup target add ${targets.join(" ")}`);
    if (platform === "android" && !hasCommand("cargo ndk")) {
      run("cargo install cargo-ndk --locked");
    }
    break;
  }
  case "ios":
    if (isMac) buildIos();
    break;
  case "android":
    buildAndroid();
    break;
  default:
    fail(`Unknown action "${action}" (expected setup, eas, ios or android)`);
}
