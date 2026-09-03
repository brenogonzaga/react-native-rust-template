const { withDangerousMod } = require("@expo/config-plugins");
const { execSync } = require("child_process");
const path = require("path");

// Both hooks delegate to scripts/setup.js, which is also what `npm run preios`
// and `npm run preandroid` call. Duplicating the cargo invocations here is how
// the plugin and the scripts drift apart.
function buildRust(projectRoot, action, label) {
  console.log(
    `\x1b[36m[Expo Plugin] Compiling Rust core for ${label}...\x1b[0m`,
  );
  try {
    execSync(`node ${path.join("scripts", "setup.js")} ${action}`, {
      cwd: projectRoot,
      stdio: "inherit",
    });
    console.log(`\x1b[32m[Expo Plugin] ${label} Rust library ready!\x1b[0m`);
  } catch (err) {
    console.error(
      `\x1b[31m[Expo Plugin Error] Failed to compile Rust core for ${label}:\x1b[0m`,
      err.message,
    );
    throw err;
  }
}

module.exports = function withRust(config) {
  const projectRoot = config._internal?.projectRoot || process.cwd();

  // EAS and explicit device builds need the arm64 device slice, not a simulator one.
  const isIosDevice =
    process.env.EAS_BUILD_PLATFORM === "ios" ||
    process.env.EXPO_BUILD_TARGET === "device";

  config = withDangerousMod(config, [
    "ios",
    async (config) => {
      if (process.platform !== "darwin") return config;
      buildRust(projectRoot, isIosDevice ? "ios-prod" : "ios", "iOS");
      return config;
    },
  ]);

  config = withDangerousMod(config, [
    "android",
    async (config) => {
      buildRust(projectRoot, "android", "Android");
      return config;
    },
  ]);

  return config;
};
