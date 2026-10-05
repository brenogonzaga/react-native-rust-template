import {
  Platform,
  StyleSheet,
  Text,
  TouchableOpacity,
  View,
} from "react-native";
import type { Call } from "../hooks/useCallLog";
import { t, type Locale } from "../i18n";
import { describeError } from "../utils/describeError";

export interface LogEntry {
  id: number;
  time: string;
  kind: Call["kind"] | "error";
  title: string;
  payload: string;
}

export function toLogEntry(call: Call, locale: Locale): LogEntry {
  const { result } = call;
  const title = t(call.title, locale, call.params);
  return {
    id: call.id,
    time: new Date(call.startedAt).toLocaleTimeString(),
    kind: result?.ok === false ? "error" : call.kind,
    title: call.ms === undefined ? title : `${title} (${call.ms}ms)`,
    payload: !result
      ? "…"
      : result.ok
        ? typeof result.data === "object"
          ? JSON.stringify(result.data, null, 2)
          : String(result.data)
        : describeError(result.error, locale),
  };
}

interface ExecutionConsoleProps {
  calls: Call[];
  onClear: () => void;
  locale: Locale;
}

export function ExecutionConsole({
  calls,
  onClear,
  locale,
}: ExecutionConsoleProps) {
  return (
    <View style={styles.consoleBox}>
      <View style={styles.consoleHeader}>
        <Text style={styles.consoleTitle}>{t("app.logsTitle", locale)}</Text>
        {calls.length > 0 && (
          <TouchableOpacity onPress={onClear}>
            <Text style={styles.clearText}>{t("app.clear", locale)}</Text>
          </TouchableOpacity>
        )}
      </View>

      <View style={styles.consoleBody}>
        {calls.length === 0 ? (
          <Text style={styles.emptyText}>{t("app.logsEmpty", locale)}</Text>
        ) : (
          calls
            .map((call) => toLogEntry(call, locale))
            .map((log) => (
              <View key={log.id} style={styles.logCard}>
                <View style={styles.logMeta}>
                  <Text
                    style={[
                      styles.tag,
                      log.kind === "system" && styles.tagSystem,
                      log.kind === "native" && styles.tagNative,
                      log.kind === "shared" && styles.tagShared,
                      log.kind === "error" && styles.tagError,
                    ]}
                  >
                    {log.kind.toUpperCase()}
                  </Text>
                  <Text style={styles.time}>{log.time}</Text>
                </View>
                <Text style={styles.logTitle}>{log.title}</Text>
                <Text style={styles.logPayload}>{log.payload}</Text>
              </View>
            ))
        )}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  consoleBox: {
    backgroundColor: "#0f172a",
    borderColor: "#1e293b",
    borderWidth: 1,
    borderRadius: 14,
    marginTop: 16,
    overflow: "hidden",
  },
  consoleHeader: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    backgroundColor: "#1e293b",
    paddingHorizontal: 12,
    paddingVertical: 8,
  },
  consoleTitle: {
    color: "#cbd5e1",
    fontSize: 10,
    fontWeight: "800",
    letterSpacing: 1,
  },
  clearText: { color: "#38bdf8", fontSize: 11, fontWeight: "600" },
  consoleBody: { padding: 12, minHeight: 100 },
  emptyText: {
    color: "#64748b",
    fontStyle: "italic",
    fontSize: 12,
    textAlign: "center",
    marginTop: 20,
  },
  logCard: {
    backgroundColor: "#1e293b",
    borderColor: "#334155",
    borderWidth: 1,
    borderRadius: 8,
    padding: 8,
    marginBottom: 6,
  },
  logMeta: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    marginBottom: 2,
  },
  tag: {
    fontSize: 8,
    fontWeight: "800",
    paddingHorizontal: 5,
    paddingVertical: 1,
    borderRadius: 3,
    overflow: "hidden",
  },
  tagSystem: { backgroundColor: "#312e81", color: "#a5b4fc" },
  tagNative: { backgroundColor: "#134e4a", color: "#2dd4bf" },
  tagShared: { backgroundColor: "#78350f", color: "#fcd34d" },
  tagError: { backgroundColor: "#7f1d1d", color: "#fca5a5" },
  time: {
    color: "#94a3b8",
    fontSize: 9,
    fontFamily: Platform.OS === "ios" ? "Menlo" : "monospace",
  },
  logTitle: {
    color: "#f8fafc",
    fontSize: 11,
    fontWeight: "700",
    marginBottom: 2,
  },
  logPayload: {
    color: "#38bdf8",
    fontFamily: Platform.OS === "ios" ? "Menlo" : "monospace",
    fontSize: 10,
    lineHeight: 14,
  },
});
