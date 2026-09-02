import { getLocales } from "expo-localization";
import { I18n } from "i18n-js";
import type { BridgeError } from "@rust-core/BridgeError";

import en from "./locales/en.json";
import pt from "./locales/pt.json";

export type Locale = "en" | "pt";

export const i18n = new I18n({ en, pt });
i18n.defaultLocale = "en";
i18n.enableFallback = true;
i18n.locale = getLocales()[0]?.languageCode ?? "en";

/**
 * Translates a `BridgeError` from the Rust core using its `kind` as the i18n
 * key and its remaining fields as interpolation params (see locales/*.json).
 */
export function translateBridgeError(error: BridgeError): string {
  const { kind, ...params } = error;
  return i18n.t(`errors.${kind}`, params);
}
