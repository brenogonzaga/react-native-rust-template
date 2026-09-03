import { useState } from "react";
import { callRust } from "rust-bridge";
import { describeError } from "../utils/describeError";
import type { Locale } from "../i18n";

export interface LogEntry {
  id: string;
  timestamp: string;
  type: "system" | "native" | "shared" | "error";
  title: string;
  payload: string;
}

export function useRustBridge(locale: Locale) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [activeCmd, setActiveCmd] = useState<string | null>(null);

  const addLog = (type: LogEntry["type"], title: string, payload: string) => {
    setLogs((prev) => [
      {
        id: Math.random().toString(36).substring(2, 9),
        timestamp: new Date().toLocaleTimeString(),
        type,
        title,
        payload,
      },
      ...prev,
    ]);
  };

  const clearLogs = () => setLogs([]);

  const runBridge = async <T>(
    cmdKey: string,
    type: LogEntry["type"],
    title: string,
    command: string,
    args?: unknown,
  ) => {
    setLoading(true);
    setActiveCmd(cmdKey);
    const start = Date.now();
    try {
      const res = await callRust<T>(command, args);
      const latency = Date.now() - start;
      const formatted =
        typeof res === "object" ? JSON.stringify(res, null, 2) : String(res);
      addLog(type, `${title} (${latency}ms)`, formatted);
    } catch (err) {
      addLog("error", `Error: ${title}`, describeError(err, locale));
    } finally {
      setLoading(false);
      setActiveCmd(null);
    }
  };

  return { logs, loading, activeCmd, runBridge, clearLogs };
}
