import {
  Platform,
  StyleSheet,
  Text,
  TouchableOpacity,
  View,
} from "react-native";
import type { LogEntry } from "../hooks/useRustBridge";

interface ExecutionConsoleProps {
  logs: LogEntry[];
  onClear: () => void;
}

export function ExecutionConsole({ logs, onClear }: ExecutionConsoleProps) {
  return (
    <View style={styles.consoleBox}>
      <View style={styles.consoleHeader}>
        <Text style={styles.consoleTitle}>EXECUTION LOGS</Text>
        {logs.length > 0 && (
          <TouchableOpacity onPress={onClear}>
            <Text style={styles.clearText}>Clear</Text>
          </TouchableOpacity>
        )}
      </View>

      <View style={styles.consoleBody}>
        {logs.length === 0 ? (
          <Text style={styles.emptyText}>
            Tap any button above to call Rust FFI logic.
          </Text>
        ) : (
          logs.map((log) => (
            <View key={log.id} style={styles.logCard}>
              <View style={styles.logMeta}>
                <Text
                  style={[
                    styles.tag,
                    log.type === "system" && styles.tagSystem,
                    log.type === "native" && styles.tagNative,
                    log.type === "shared" && styles.tagShared,
                    log.type === "error" && styles.tagError,
                  ]}
                >
                  {log.type.toUpperCase()}
                </Text>
                <Text style={styles.time}>{log.timestamp}</Text>
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
