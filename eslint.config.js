// https://docs.expo.dev/guides/using-eslint/
const { defineConfig } = require("eslint/config");
const expoConfig = require("eslint-config-expo/flat");
const globals = require("globals");
const prettierConfig = require("eslint-config-prettier");

module.exports = defineConfig([
  expoConfig,
  {
    ignores: ["dist/*", "rust-core/bindings/**", "crates/app_core/bindings/**"],
  },
  {
    files: ["scripts/**/*.js", "plugins/**/*.js"],
    languageOptions: {
      globals: globals.node,
    },
  },
  prettierConfig,
]);
