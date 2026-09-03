import { getLocales } from "expo-localization";
import { I18n } from "i18n-js";
import type { BridgeError } from "@rust-core/BridgeError";

import en from "./locales/en.json";
import pt from "./locales/pt.json";

const translations = { en, pt };

export type Locale = keyof typeof translations;
export const SUPPORTED_LOCALES = Object.keys(translations) as Locale[];

function isSupportedLocale(code: string | null | undefined): code is Locale {
  return !!code && (SUPPORTED_LOCALES as string[]).includes(code);
}

export function resolveInitialLocale(): Locale {
  try {
    const code = getLocales()[0]?.languageCode;
    return isSupportedLocale(code) ? code : "en";
  } catch {
    return "en";
  }
}

export const i18n = new I18n(translations);
i18n.defaultLocale = "en";
i18n.enableFallback = true;

export function t(
  key: string,
  locale: Locale,
  params?: Record<string, unknown>,
): string {
  return i18n.t(key, { locale, ...params });
}

/**
 * Translates a `BridgeError` from the Rust core using its `kind` as the i18n
 * key and its remaining fields as interpolation params (see locales/*.json).
 */
export function translateBridgeError(
  error: BridgeError,
  locale: Locale,
): string {
  const { kind, ...params } = error;
  return t(`errors.${kind}`, locale, params);
}
