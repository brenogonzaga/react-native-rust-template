import { RustBridgeError } from "rust-bridge";
import { translateBridgeError, type Locale } from "../i18n";

export function describeError(err: unknown, locale: Locale): string {
  if (err instanceof RustBridgeError)
    return translateBridgeError(err.bridgeError, locale);
  if (err instanceof Error) return err.message;
  return String(err);
}
