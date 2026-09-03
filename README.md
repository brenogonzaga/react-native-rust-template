# React Native Rust Template

A minimal, production-ready Expo template for calling Rust code from React Native on iOS and Android.

<p align="center">
  <img src="./image.png" alt="React Native Rust Template Screenshot" width="340" />
</p>

## Overview

This template shows how to run shared Rust business logic inside a React Native app. It uses Expo Native Modules, C FFI for iOS, and JNI for Android.

### Highlights

- **Multi-Crate Workspace**: Decouples domain logic (`crates/app_core`) from mobile bridge code (`rust-core`).
- **Off-Thread by Default**: Calls run on the platform's background queue (Expo's serial `AsyncFunction` queue on iOS, `Dispatchers.IO` on Android), so the JS thread never blocks. No async runtime is bundled — handlers are synchronous.
- **Panic Protection**: Intercepts Rust panics with `catch_unwind` and returns typed errors (`panic = "unwind"` is pinned in the workspace release profile, which `catch_unwind` depends on).
- **Injection-Proof Boundary**: Command and args cross as one `JSON.stringify`d envelope that Rust parses once. Nothing is ever concatenated into JSON at the FFI layer.
- **Automated Builds**: Lifecycle hooks (`preios`/`preandroid`) via `scripts/setup.js` and Expo Config Plugin (`plugins/withRust.js`) handle target compilation and binary placement automatically.

---

## Workspace Structure

```text
.
├── Cargo.toml                    # Root Cargo workspace
├── package.json                  # App dependencies & build scripts
├── scripts/
│   └── setup.js                  # Unified setup & Rust compilation script
├── plugins/
│   └── withRust.js               # Expo Config Plugin for build automation
├── crates/
│   └── app_core/                 # Core domain logic (pure Rust, no FFI)
│       └── src/lib.rs            # Business logic, models, & errors
├── rust-core/                    # FFI bridge layer
│   ├── src/
│   │   ├── lib.rs                # C FFI & Android JNI entry points
│   │   ├── dispatcher.rs         # Command router & JSON serializer
│   │   ├── error.rs              # BridgeError mapping
│   │   ├── state.rs              # Global state instance
│   │   ├── safe_int.rs           # SafeInt<T>: wire-safe 64-bit+ integers
│   │   ├── wire.rs               # Wire::encode, the one place a response is encoded
│   │   ├── mock_export.rs        # Re-exports app_core types into rust-core/bindings
│   │   └── handlers/             # Modular request handlers (system, math, user)
│   └── tests/                    # Guards every #[ts(export)] type in this crate
├── modules/
│   └── rust-bridge/              # Expo Native Module
│       ├── ios/RustBridgeModule.swift
│       ├── android/.../RustBridgeModule.kt
│       └── src/index.ts          # TypeScript client
├── src/
│   ├── i18n/                     # Locale catalogs & BridgeError translation
│   ├── hooks/useRustBridge.ts    # Bridge call + logging hook
│   └── utils/describeError.ts    # Error -> user-facing string (+ .test.ts)
└── App.tsx                       # UI demo
```

---

## How Rust is Linked in Expo

Expo CLI (`npx expo start`) bundles JavaScript, while Expo Prebuild (`npx expo run:android` / `npx expo run:ios`) compiles native code.

To call Rust from React Native:

1. Rust is cross-compiled for target architectures (`aarch64-linux-android` / `aarch64-apple-ios-sim`) with `IPHONEOS_DEPLOYMENT_TARGET=16.0`.
2. Compiled binaries are placed in `modules/rust-bridge/android/src/main/jniLibs` and `modules/rust-bridge/ios/lib`.
3. Expo prebuild links the Rust library into the native iOS/Android binaries.
4. Heavy binary files (`*.a` / `*.so`) are ignored in `.gitignore`, while `.gitkeep` files keep the target folder structure tracked for clean git clones and EAS Build pipelines.

NPM lifecycle hooks (`preios` and `preandroid`) execute `scripts/setup.js` automatically prior to `expo run`.

---

## Getting Started

### Prerequisites

- Node.js 18+ and npm
- Expo SDK 57 (React Native 0.86)
- Rust toolchain (`rustup`)
- `cargo-ndk` for Android builds:
  ```bash
  cargo install cargo-ndk
  ```

### Setup

```bash
# Add target toolchains and install npm dependencies
npm run setup
```

### Running the App

```bash
# Run on Android (compiles Rust via cargo-ndk -> runs expo run:android)
npm run android

# Run on iOS Simulator (macOS only, compiles Rust -> runs expo run:ios)
npm run ios
```

### Standalone Rust Compilation

If you want to build the Rust binaries without launching the emulator:

```bash
node scripts/setup.js android
node scripts/setup.js ios
```

---

## How to Call Rust from TypeScript

```typescript
import { callRust } from "rust-bridge";

// Call a handler defined in rust-core/src/handlers/
async function calculateFactorial() {
  try {
    // Result is a string: 64-bit values cross the wire as decimal strings —
    // JS numbers are f64 and would round anything past 2^53.
    const result = await callRust<string>("math", {
      type: "factorial",
      n: 5,
    });
    console.log("Result:", result); // "120"
  } catch (error) {
    console.error("Rust Error:", error);
  }
}
```

---

## Adding a New Rust Command

1. **Domain logic** (`crates/app_core/src/lib.rs`): write the pure Rust function/struct. Derive `#[derive(TS)]` + `#[ts(export)]` on any type you want mirrored in TypeScript.
2. **Errors** (`rust-core/src/error.rs`): if the operation can fail, add a variant to `CoreError` and map it in the `From<CoreError> for BridgeError` impl.
3. **Handler** (`rust-core/src/handlers/`): add or extend a `<Name>Command` enum (`#[serde(tag = "type")]`) and its `dispatch()` match arm, calling into the domain logic via `CORE_SERVICE` and returning `Wire::encode(&value)` (`rust-core/src/wire.rs`). `Wire::encode` only accepts types implementing `SafeForWire`, which is what forces a 64-bit+ value through `SafeInt<T>` (`rust-core/src/safe_int.rs`) instead of a bare `u64`/`i64`.
4. **Dispatcher** (`rust-core/src/dispatcher.rs`): add the handler's command enum as a variant of `BridgeCommand` (`#[serde(tag = "cmd", content = "args")]`) and route it in `BridgeDispatcher::run()`. New handler file? Add `pub mod <name>;` to `handlers/mod.rs`.
5. **Regenerate TS types**: run `cargo test --workspace` — any `#[ts(export)]` type is (re)written to `rust-core/bindings/` or `crates/app_core/bindings/` as a side effect of the test run.
6. **Call it from TypeScript**:
   ```typescript
   const result = await callRust<ReturnType>("<command>", {
     type: "<variant>",
     /* args */
   });
   ```
7. **Localize errors**: if the command can return a `BridgeError`, add its `kind` string to `src/i18n/locales/en.json` and `pt.json` under `"errors"` — `translateBridgeError()` looks it up by `kind` and interpolates the remaining fields.

---

## Testing & Linting

- `npm test` / `npm run lint` / `npm run typecheck` / `npm run format:check` — Jest (`jest-expo` preset), ESLint, `tsc --noEmit`, Prettier.
- `cargo test --workspace` — Rust unit tests; also regenerates the ts-rs bindings (see step 5 above).
- `cargo clippy --workspace --all-targets -- -D warnings` / `cargo fmt --check` — Rust lint & format.

All of the above run in CI on every push/PR (`.github/workflows/ci.yml`). A Husky pre-commit hook (`npx lint-staged`) formats staged files automatically.

---

## Customizing Package Name / App Namespace

If you change the Android package name (e.g., from `com.myapp.rustbridge` to `com.yourcompany.app`), you **must** update the JNI function signature in `rust-core/src/lib.rs`:

1. **Android JNI Function (`rust-core/src/lib.rs`)**:
   Rename `Java_com_myapp_rustbridge_RustBridgeModule_callRustNative` to match your new Android package path (replacing dots with underscores):

   ```rust
   // For package com.yourcompany.app:
   pub extern "system" fn Java_com_yourcompany_app_RustBridgeModule_callRustNative(...)
   ```

   Works whether `callRustNative` is declared directly on the module class (as here) or as `@JvmStatic external` inside a `companion object` — Kotlin emits the native method on the outer class either way (`javap -s` on the compiled class confirms it).

2. **Android Package Namespace (`modules/rust-bridge/android/build.gradle`)**:
   Update `namespace` in the `android {}` block to match your target package:
   ```groovy
   android {
     namespace "com.yourcompany.app"
   }
   ```

---

## License

MIT
