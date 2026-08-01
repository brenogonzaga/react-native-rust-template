import { requireNativeModule } from "expo-modules-core";

interface RustBridgeNative {
  callRust(command: string, payload: string): Promise<string>;
}

// Access native Expo module
const RustBridge: RustBridgeNative = requireNativeModule("RustBridge");

/**
 * Calls a Rust command asynchronously via the native Expo bridge.
 *
 * @param command The Rust command name (snake_case match)
 * @param payload Optional arguments for the command
 * @returns Parsed JSON response payload of type T
 */
export async function callRust<T = unknown>(
  command: string,
  payload?: unknown
): Promise<T> {
  try {
    const jsonPayload = payload !== undefined ? JSON.stringify(payload) : "";
    const responseJson = await RustBridge.callRust(command, jsonPayload);

    const parsed = JSON.parse(responseJson);

    if (parsed.status === "error") {
      throw new Error(parsed.message || "Unknown error inside Rust bridge");
    }

    return parsed.data as T;
  } catch (error) {
    throw new Error(
      `[RustBridge] Bridge call failed: ${
        error instanceof Error ? error.message : String(error)
      }`
    );
  }
}
