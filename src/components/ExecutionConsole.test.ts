import { RustBridgeError } from "rust-bridge";
import type { Call } from "../hooks/useCallLog";
import { toLogEntry } from "./ExecutionConsole";

const base: Call = {
  id: 1,
  kind: "shared",
  title: "app.saveUserLogTitle",
  startedAt: 0,
};

describe("toLogEntry", () => {
  it("titles a finished call with its params and latency, pretty-printing data", () => {
    const user = { id: "42", name: "Ana", role: "QA" };
    const entry = toLogEntry(
      { ...base, params: user, ms: 3, result: { ok: true, data: user } },
      "en",
    );
    expect(entry).toMatchObject({
      kind: "shared",
      title: "Save User #42 (3ms)",
      payload: JSON.stringify(user, null, 2),
    });
  });

  it("shows a failed call as a localized error", () => {
    const entry = toLogEntry(
      {
        ...base,
        title: "app.getUserLogTitle",
        params: { id: "42" },
        ms: 1,
        result: {
          ok: false,
          error: new RustBridgeError({ kind: "not_found", id: "42" }),
        },
      },
      "pt",
    );
    expect(entry).toMatchObject({
      kind: "error",
      title: "Buscar Usuário #42 (1ms)",
      payload: "Usuário 42 não encontrado",
    });
  });

  it("shows a placeholder while the call is in flight", () => {
    const entry = toLogEntry(
      {
        ...base,
        kind: "native",
        title: "app.factorialLogTitle",
        params: { n: 5 },
      },
      "en",
    );
    expect(entry).toMatchObject({ title: "Factorial (5!)", payload: "…" });
  });
});
