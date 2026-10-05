import { useCallback, useRef, useState } from "react";

export interface CallMeta {
  kind: "system" | "native" | "shared";
  title: string;
  params?: Record<string, unknown>;
}

export interface Call extends CallMeta {
  id: number;
  startedAt: number;
  ms?: number;
  result?: { ok: true; data: unknown } | { ok: false; error: unknown };
}

export function useCallLog() {
  const [calls, setCalls] = useState<Call[]>([]);
  const nextId = useRef(0);

  const track = useCallback(
    async (meta: CallMeta, run: () => Promise<unknown>) => {
      const call: Call = {
        ...meta,
        id: ++nextId.current,
        startedAt: Date.now(),
      };
      setCalls((prev) => [call, ...prev]);

      let result: Call["result"];
      try {
        result = { ok: true, data: await run() };
      } catch (error) {
        result = { ok: false, error };
      }
      const ms = Date.now() - call.startedAt;
      setCalls((prev) =>
        prev.map((c) => (c.id === call.id ? { ...c, result, ms } : c)),
      );
    },
    [],
  );

  const clear = useCallback(() => setCalls([]), []);

  return { calls, busy: calls.some((c) => !c.result), track, clear };
}
