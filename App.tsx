import { useState } from "react";
import {
  StyleSheet,
  Text,
  View,
  TouchableOpacity,
  ActivityIndicator,
  ScrollView,
  Platform,
} from "react-native";
import { SafeAreaProvider, SafeAreaView } from "react-native-safe-area-context";
import { StatusBar } from "expo-status-bar";
import type { User } from "@app_core/User";
import type { AppVersion } from "@app_core/AppVersion";
import { t, resolveInitialLocale, type Locale } from "./src/i18n";
import { useRustBridge } from "./src/hooks/useRustBridge";
import { LocaleSwitcher } from "./src/components/LocaleSwitcher";
import { ExecutionConsole } from "./src/components/ExecutionConsole";

export default function App() {
  const [locale, setLocale] = useState<Locale>(resolveInitialLocale());
  const { logs, loading, activeCmd, runBridge, clearLogs } =
    useRustBridge(locale);

  return (
    <SafeAreaProvider>
      <SafeAreaView style={styles.container}>
        <StatusBar style="dark" />

        <ScrollView
          contentContainerStyle={styles.scrollContent}
          showsVerticalScrollIndicator={false}
        >
          {/* Header */}
          <View style={styles.header}>
            <View style={styles.badge}>
              <View style={styles.badgeDot} />
              <Text style={styles.badgeText}>{t("app.badge", locale)}</Text>
            </View>
            <Text style={styles.title}>{t("app.title", locale)}</Text>
            <Text style={styles.subtitle}>{t("app.subtitle", locale)}</Text>
            <LocaleSwitcher locale={locale} onChange={setLocale} />
          </View>

          {/* 1. SYSTEM */}
          <Text style={styles.sectionHeader}>
            {t("app.sectionSystem", locale)}
          </Text>
          <View style={styles.row}>
            <TouchableOpacity
              style={[
                styles.btn,
                styles.btnIndigo,
                activeCmd === "ping" && styles.active,
              ]}
              onPress={() =>
                runBridge<string>(
                  "ping",
                  "system",
                  t("app.pingLogTitle", locale),
                  "system",
                  { type: "ping" },
                )
              }
              activeOpacity={0.7}
            >
              <Text style={styles.btnIcon}>📡</Text>
              <Text style={[styles.btnTitle, styles.textIndigo]}>
                {t("app.ping", locale)}
              </Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={[
                styles.btn,
                styles.btnSky,
                activeCmd === "version" && styles.active,
              ]}
              onPress={() =>
                runBridge<AppVersion>(
                  "version",
                  "system",
                  t("app.versionLogTitle", locale),
                  "system",
                  { type: "get_version" },
                )
              }
              activeOpacity={0.7}
            >
              <Text style={styles.btnIcon}>⚙️</Text>
              <Text style={[styles.btnTitle, styles.textSky]}>
                {t("app.version", locale)}
              </Text>
            </TouchableOpacity>
          </View>

          {/* 2. NATIVE */}
          <Text style={styles.sectionHeader}>
            {t("app.sectionNative", locale)}
          </Text>
          <TouchableOpacity
            style={[
              styles.btnFull,
              styles.btnTeal,
              activeCmd === "math" && styles.active,
            ]}
            onPress={() =>
              runBridge<string>(
                "math",
                "native",
                t("app.factorialTitle", locale),
                "math",
                { type: "factorial", n: 5 },
              )
            }
            activeOpacity={0.7}
          >
            <Text style={styles.btnIcon}>🧮</Text>
            <View style={styles.btnTextCol}>
              <Text style={[styles.btnTitle, styles.textTeal]}>
                {t("app.factorialTitle", locale)}
              </Text>
              <Text style={styles.btnSub}>{t("app.factorialSub", locale)}</Text>
            </View>
          </TouchableOpacity>

          <TouchableOpacity
            style={[
              styles.btnFull,
              styles.btnRose,
              activeCmd === "math-overflow" && styles.active,
            ]}
            onPress={() =>
              runBridge<string>(
                "math-overflow",
                "native",
                t("app.factorialOverflowTitle", locale),
                "math",
                { type: "factorial", n: 25 },
              )
            }
            activeOpacity={0.7}
          >
            <Text style={styles.btnIcon}>💥</Text>
            <View style={styles.btnTextCol}>
              <Text style={[styles.btnTitle, styles.textRose]}>
                {t("app.factorialOverflowTitle", locale)}
              </Text>
              <Text style={styles.btnSub}>
                {t("app.factorialOverflowSub", locale, {
                  localeLabel: locale.toUpperCase(),
                })}
              </Text>
            </View>
          </TouchableOpacity>

          {/* 3. SHARED */}
          <Text style={styles.sectionHeader}>
            {t("app.sectionShared", locale)}
          </Text>
          <View style={styles.row}>
            <TouchableOpacity
              style={[
                styles.btn,
                styles.btnAmber,
                activeCmd === "get_user" && styles.active,
              ]}
              onPress={() =>
                runBridge<User>(
                  "get_user",
                  "shared",
                  t("app.getUserLogTitle", locale),
                  "user",
                  { type: "get_user", id: "1" },
                )
              }
              activeOpacity={0.7}
            >
              <Text style={styles.btnIcon}>🔍</Text>
              <Text style={[styles.btnTitle, styles.textAmber]}>
                {t("app.getUser", locale)}
              </Text>
            </TouchableOpacity>

            <TouchableOpacity
              style={[
                styles.btn,
                styles.btnEmerald,
                activeCmd === "save_user" && styles.active,
              ]}
              onPress={() => {
                const id = String(Date.now()).slice(-4);
                runBridge<User>(
                  "save_user",
                  "shared",
                  t("app.saveUserLogTitle", locale, { id }),
                  "user",
                  {
                    type: "save_user",
                    id,
                    name: "Carlos Dev",
                    role: "Lead Engineer",
                  },
                );
              }}
              activeOpacity={0.7}
            >
              <Text style={styles.btnIcon}>💾</Text>
              <Text style={[styles.btnTitle, styles.textEmerald]}>
                {t("app.saveUser", locale)}
              </Text>
            </TouchableOpacity>
          </View>

          {/* Loading Indicator */}
          {loading && (
            <View style={styles.loadingBanner}>
              <ActivityIndicator size="small" color="#4f46e5" />
              <Text style={styles.loadingText}>{t("app.loading", locale)}</Text>
            </View>
          )}

          <ExecutionConsole logs={logs} onClear={clearLogs} locale={locale} />
        </ScrollView>
      </SafeAreaView>
    </SafeAreaProvider>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: "#f8fafc",
  },
  scrollContent: {
    paddingHorizontal: 20,
    paddingTop: Platform.OS === "android" ? 40 : 20,
    paddingBottom: 40,
  },
  header: {
    alignItems: "center",
    marginBottom: 20,
  },
  badge: {
    flexDirection: "row",
    alignItems: "center",
    backgroundColor: "#ecfdf5",
    borderColor: "#a7f3d0",
    borderWidth: 1,
    paddingHorizontal: 10,
    paddingVertical: 3,
    borderRadius: 16,
    marginBottom: 8,
    gap: 6,
  },
  badgeDot: {
    width: 6,
    height: 6,
    borderRadius: 3,
    backgroundColor: "#059669",
  },
  badgeText: {
    color: "#047857",
    fontSize: 10,
    fontWeight: "800",
    letterSpacing: 1,
  },
  title: {
    fontSize: 24,
    fontWeight: "900",
    color: "#0f172a",
    letterSpacing: -0.5,
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 12,
    color: "#64748b",
  },
  sectionHeader: {
    color: "#475569",
    fontSize: 11,
    fontWeight: "800",
    letterSpacing: 1,
    textTransform: "uppercase",
    marginBottom: 8,
    marginTop: 12,
  },
  row: {
    flexDirection: "row",
    gap: 10,
    marginBottom: 4,
  },
  btn: {
    flex: 1,
    flexDirection: "row",
    alignItems: "center",
    padding: 12,
    borderRadius: 12,
    borderWidth: 1,
    gap: 8,
  },
  btnFull: {
    flexDirection: "row",
    alignItems: "center",
    padding: 12,
    borderRadius: 12,
    borderWidth: 1,
    gap: 10,
    marginBottom: 4,
  },
  btnIndigo: { backgroundColor: "#eef2ff", borderColor: "#c7d2fe" },
  btnSky: { backgroundColor: "#f0f9ff", borderColor: "#bae6fd" },
  btnTeal: { backgroundColor: "#f0fdf4", borderColor: "#bbf7d0" },
  btnAmber: { backgroundColor: "#fffbeb", borderColor: "#fde68a" },
  btnEmerald: { backgroundColor: "#ecfdf5", borderColor: "#a7f3d0" },
  btnRose: { backgroundColor: "#fff1f2", borderColor: "#fecdd3" },
  textIndigo: { color: "#3730a3" },
  textSky: { color: "#075985" },
  textTeal: { color: "#065f46" },
  textAmber: { color: "#92400e" },
  textEmerald: { color: "#065f46" },
  textRose: { color: "#9f1239" },
  active: { opacity: 0.5 },
  btnIcon: { fontSize: 16 },
  btnTextCol: { flex: 1 },
  btnTitle: { fontSize: 13, fontWeight: "700" },
  btnSub: { fontSize: 11, color: "#64748b", marginTop: 1 },
  loadingBanner: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "center",
    gap: 8,
    backgroundColor: "#eef2ff",
    borderColor: "#c7d2fe",
    borderWidth: 1,
    paddingVertical: 8,
    borderRadius: 10,
    marginTop: 12,
  },
  loadingText: { color: "#3730a3", fontSize: 12, fontWeight: "600" },
});
