import { RustBridgeError } from "rust-bridge";
import { translateBridgeError } from "../i18n";

export function describeError(err: unknown): string {
  if (err instanceof RustBridgeError)
    return translateBridgeError(err.bridgeError);
  if (err instanceof Error) return err.message;
  return String(err);
}
