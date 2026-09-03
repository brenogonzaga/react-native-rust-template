import { requireNativeModule } from "expo-modules-core";
import type { BridgeError } from "@rust-core/BridgeError";

interface RustBridgeNative {
  callRust(envelopeJson: string): Promise<string>;
}

// Resolved lazily (on first call) so importing this module — e.g. just for
// the `RustBridgeError` class — doesn't require the native module to exist,
// which keeps pure logic that depends on it unit-testable outside Expo.
let nativeModule: RustBridgeNative | undefined;
function getNativeModule(): RustBridgeNative {
  if (nativeModule) return nativeModule;
  const resolved = requireNativeModule<RustBridgeNative>("RustBridge");
  nativeModule = resolved;
  return resolved;
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
 * The command and its arguments cross as a single `JSON.stringify`d envelope.
 * Rust parses that document once and never concatenates strings into JSON, so
 * a command name containing quotes cannot forge the `args` it is paired with.
 *
 * @param command The Rust command name (snake_case match)
 * @param payload Optional arguments for the command
 * @returns Parsed JSON response payload of type T
 */
export async function callRust<T = unknown>(
  command: string,
  payload?: unknown,
): Promise<T> {
  try {
    const envelope =
      payload !== undefined
        ? { cmd: command, args: payload }
        : { cmd: command };
    const responseJson = await getNativeModule().callRust(
      JSON.stringify(envelope),
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
