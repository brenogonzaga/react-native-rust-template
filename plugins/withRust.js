const { withDangerousMod } = require("expo/config-plugins");
const { execSync } = require("child_process");
const path = require("path");

// Both hooks delegate to scripts/setup.js, which is also what `npm run preios`
// and `npm run preandroid` call — so prebuild (EAS, CI, or a plain
// `npx expo prebuild`) leaves the Rust library in place before `pod install`
// and Gradle look for it. setup.js picks the cargo profile (release on EAS).
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
  config = withDangerousMod(config, [
    "ios",
    async (config) => {
      if (process.platform !== "darwin") return config;
      buildRust(config.modRequest.projectRoot, "ios", "iOS");
      return config;
    },
  ]);

  config = withDangerousMod(config, [
    "android",
    async (config) => {
      buildRust(config.modRequest.projectRoot, "android", "Android");
      return config;
    },
  ]);

  return config;
};
