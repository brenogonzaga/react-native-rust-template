import { requireNativeModule } from "expo-modules-core";
import type { BridgeCommand } from "@bindings/BridgeCommand";
import type { BridgeError } from "@bindings/BridgeError";

export type { BridgeCommand };

interface RustBridgeNative {
  callRust(envelopeJson: string): Promise<string>;
}

// Resolved lazily (on first call) so importing this module — e.g. just for
// the `RustBridgeError` class — doesn't require the native module to exist,
// which keeps pure logic that depends on it unit-testable outside Expo.
let nativeModule: RustBridgeNative | undefined;
function getNativeModule(): RustBridgeNative {
  if (nativeModule) return nativeModule;
  try {
    nativeModule = requireNativeModule<RustBridgeNative>("RustBridge");
  } catch {
    throw new Error(
      "RustBridge native module not found. Expo Go and web don't include " +
        "custom native code: build the app with `npm run ios` / `npm run android`.",
    );
  }
  return nativeModule;
}

/**
 * Thrown when the Rust core returns a structured error. Carries the raw
 * `BridgeError` (kind + params) so callers can localize it via `src/i18n`
 * instead of matching on an English message string.
 */
export class RustBridgeError extends Error {
  readonly bridgeError: BridgeError;

  constructor(bridgeError: BridgeError) {
    super(`[RustBridge] ${bridgeError.kind}`);
    this.name = "RustBridgeError";
    this.bridgeError = bridgeError;
  }
}

/**
 * Calls a Rust command asynchronously via the native Expo bridge.
 *
 * `command` is typed by `BridgeCommand`, generated from the Rust enum of the
 * same name (rust-core/src/dispatcher.rs), so command names and args are
 * checked at compile time. It crosses as one `JSON.stringify`d envelope that
 * Rust parses once — nothing is concatenated into JSON on either side.
 *
 * @returns The command's `data` payload, typed as `T` by the caller.
 */
export async function callRust<T = unknown>(
  command: BridgeCommand,
): Promise<T> {
  try {
    const responseJson = await getNativeModule().callRust(
      JSON.stringify(command),
    );

    const parsed = JSON.parse(responseJson);

    if (parsed.status === "error") {
      throw new RustBridgeError(parsed as BridgeError);
    }

    return parsed.data as T;
  } catch (error) {
    if (error instanceof RustBridgeError) throw error;
    throw new Error(
      `[RustBridge] Bridge call failed: ${
        error instanceof Error ? error.message : String(error)
      }`,
    );
  }
}
