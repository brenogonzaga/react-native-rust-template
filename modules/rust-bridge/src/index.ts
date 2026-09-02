import { requireNativeModule } from "expo-modules-core";
import type { BridgeError } from "@rust-core/BridgeError";

interface RustBridgeNative {
  callRust(command: string, payload: string): Promise<string>;
}

// Access native Expo module
const RustBridge: RustBridgeNative = requireNativeModule("RustBridge");

/**
 * Thrown when the Rust core returns a structured error. Carries the raw
 * `BridgeError` (kind + params) so callers can localize it via `src/i18n`
 * instead of matching on an English message string.
 */
export class RustBridgeError extends Error {
  constructor(public readonly bridgeError: BridgeError) {
    super(`[RustBridge] ${bridgeError.kind}`);
    this.name = "RustBridgeError";
  }
}

/**
 * Calls a Rust command asynchronously via the native Expo bridge.
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
    const jsonPayload = payload !== undefined ? JSON.stringify(payload) : "";
    const responseJson = await RustBridge.callRust(command, jsonPayload);

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
