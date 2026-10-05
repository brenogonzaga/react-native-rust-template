import { requireNativeModule } from "expo-modules-core";
import type { BridgeCommand } from "@bindings/BridgeCommand";
import type { BridgeError } from "@bindings/BridgeError";
import type { BridgeResponses } from "@bindings/BridgeResponses";

export type { BridgeCommand, BridgeResponses };

export type ResponseOf<C extends BridgeCommand> = C extends {
  cmd: infer K extends keyof BridgeResponses;
  args: { type: infer V };
}
  ? V extends keyof BridgeResponses[K]
    ? BridgeResponses[K][V]
    : never
  : never;

interface RustBridgeNative {
  readonly dataDir: string;
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
 * The app's private, writable directory (Application Support on iOS,
 * `filesDir` on Android) — pass it to the `system` `init` command.
 */
export function dataDir(): string {
  return getNativeModule().dataDir;
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
 * checked at compile time, and the result is typed by `BridgeResponses`. It
 * crosses as one `JSON.stringify`d envelope that Rust parses once — nothing is
 * concatenated into JSON on either side.
 */
export async function callRust<C extends BridgeCommand>(
  command: C,
): Promise<ResponseOf<C>> {
  const label = `${command.cmd}.${command.args.type}`;
  const start = Date.now();
  try {
    const responseJson = await getNativeModule().callRust(
      JSON.stringify(command),
    );

    const { status, ...parsed } = JSON.parse(responseJson);

    if (status === "error") {
      throw new RustBridgeError(parsed as BridgeError);
    }

    logCall(label, start);
    return parsed.data as ResponseOf<C>;
  } catch (error) {
    const failure =
      error instanceof RustBridgeError
        ? error
        : new Error(
            `[RustBridge] Bridge call failed: ${
              error instanceof Error ? error.message : String(error)
            }`,
          );
    logCall(label, start, failure);
    throw failure;
  }
}

const SLOW_CALL_MS = 100;

function logCall(label: string, start: number, error?: Error) {
  if (!__DEV__) return;
  const ms = Date.now() - start;
  if (error instanceof RustBridgeError) {
    const { kind, ...detail } = error.bridgeError;
    const line = `[RustBridge] ${label} → ${kind} ${JSON.stringify(detail)} (${ms}ms)`;
    if (kind === "invalid_command") {
      console.warn(
        `${line}\nThe native library is older than this JS bundle: rebuild it with \`npm run ios\` / \`npm run android\`.`,
      );
    } else {
      console.info(line);
    }
  } else if (error) {
    console.warn(`[RustBridge] ${label} failed (${ms}ms): ${error.message}`);
  } else if (ms > SLOW_CALL_MS) {
    console.warn(`[RustBridge] ${label} took ${ms}ms`);
  } else {
    console.debug(`[RustBridge] ${label} (${ms}ms)`);
  }
}
