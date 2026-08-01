const { withDangerousMod } = require("@expo/config-plugins");
const { execSync } = require("child_process");
const path = require("path");
const fs = require("fs");
const os = require("os");

module.exports = function withRust(config) {
  const projectRoot = config._internal?.projectRoot || process.cwd();
  const isArm64 = os.arch() === "arm64";
  const isMac = os.platform() === "darwin";

  const isIosDevice = process.env.EAS_BUILD_PLATFORM === "ios" || process.env.EXPO_BUILD_TARGET === "device";
  const iosTarget = isIosDevice ? "aarch64-apple-ios" : (isArm64 ? "aarch64-apple-ios-sim" : "x86_64-apple-ios");

  // iOS Plugin Hook: Compiles Rust core before CocoaPods / Xcode
  config = withDangerousMod(config, [
    "ios",
    async (config) => {
      if (!isMac) return config;

      console.log(`\x1b[36m[Expo Plugin] Compiling Rust core for iOS (${iosTarget})...\x1b[0m`);
      const rustCoreDir = path.join(projectRoot, "rust-core");
      const iosLibDir = path.join(projectRoot, "modules", "rust-bridge", "ios", "lib");
      
      if (!fs.existsSync(iosLibDir)) {
        fs.mkdirSync(iosLibDir, { recursive: true });
      }

      try {
        execSync(`cargo build --release -p rust-core --target ${iosTarget}`, {
          cwd: projectRoot,
          stdio: "inherit",
          env: { ...process.env, IPHONEOS_DEPLOYMENT_TARGET: "16.0" },
        });

        const src = path.join(projectRoot, "target", iosTarget, "release", "librust_bridge.a");
        const dest = path.join(iosLibDir, "librust_bridge.a");
        fs.copyFileSync(src, dest);

        if (!fs.existsSync(dest)) {
          throw new Error(`[Expo Plugin Error] Missing compiled iOS Rust library at ${dest}`);
        }
        console.log("\x1b[32m[Expo Plugin] iOS Rust library ready!\x1b[0m");
      } catch (err) {
        console.error("\x1b[31m[Expo Plugin Error] Failed to compile iOS Rust library:\x1b[0m", err.message);
        throw err;
      }

      return config;
    },
  ]);

  // Android Plugin Hook: Compiles Rust core before Gradle
  config = withDangerousMod(config, [
    "android",
    async (config) => {
      console.log(`\x1b[36m[Expo Plugin] Compiling Rust core for Android (arm64-v8a, armeabi-v7a, x86_64)...\x1b[0m`);
      const rustCoreDir = path.join(projectRoot, "rust-core");
      const androidJniDir = path.join(projectRoot, "modules", "rust-bridge", "android", "src", "main", "jniLibs");
      
      const outDir = path.relative(rustCoreDir, androidJniDir).replace(/\\/g, "/");

      try {
        execSync(`cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -o ${outDir} build --release`, {
          cwd: rustCoreDir,
          stdio: "inherit",
        });

        const requiredAbis = ["arm64-v8a", "armeabi-v7a", "x86_64"];
        const missing = requiredAbis.filter(abi => !fs.existsSync(path.join(androidJniDir, abi, "librust_bridge.so")));
        if (missing.length > 0) {
          throw new Error(`Missing compiled Android Rust libraries for ABIs: ${missing.join(", ")}`);
        }

        console.log("\x1b[32m[Expo Plugin] Android Rust library ready!\x1b[0m");
      } catch (err) {
        console.error("\x1b[31m[Expo Plugin Error] Failed to compile Android Rust core:\x1b[0m", err.message);
        throw err;
      }

      return config;
    },
  ]);

  return config;
};
