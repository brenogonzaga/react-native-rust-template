import type { User } from "@bindings/User";
import type { BridgeCommand, BridgeResponses } from "./index";

const mockCallRust = jest.fn();
const mockRequireNativeModule = jest.fn();

// Only RustBridge is faked: jest-expo's own setup resolves other native
// modules (e.g. ExpoFetchModule) through this same function.
jest.mock("expo-modules-core", () => {
  const actual = jest.requireActual("expo-modules-core");
  return {
    ...actual,
    requireNativeModule: (name: string) =>
      name === "RustBridge"
        ? mockRequireNativeModule(name)
        : actual.requireNativeModule(name),
  };
});

// Fresh copy per test: index.ts caches the native module after the first call.
function load(): typeof import("./index") {
  let mod!: typeof import("./index");
  jest.isolateModules(() => {
    mod = jest.requireActual("./index");
  });
  return mod;
}

afterEach(() => jest.restoreAllMocks());

beforeEach(() => {
  for (const level of ["debug", "info", "warn"] as const) {
    jest.spyOn(console, level).mockImplementation(() => {});
  }
  mockCallRust.mockReset();
  mockRequireNativeModule
    .mockReset()
    .mockReturnValue({ callRust: mockCallRust });
});

describe("callRust", () => {
  it("sends the command as a single JSON envelope and returns data", async () => {
    mockCallRust.mockResolvedValue('{"status":"success","data":"pong"}');
    const { callRust } = load();

    await expect(
      callRust({ cmd: "system", args: { type: "ping" } }),
    ).resolves.toBe("pong");
    expect(mockCallRust).toHaveBeenCalledWith(
      '{"cmd":"system","args":{"type":"ping"}}',
    );
  });

  it("throws a RustBridgeError carrying the structured error", async () => {
    mockCallRust.mockResolvedValue(
      '{"status":"error","kind":"not_found","id":"42"}',
    );
    const { callRust, RustBridgeError } = load();

    const call = callRust({
      cmd: "user",
      args: { type: "get_user", id: "42" },
    });
    await expect(call).rejects.toBeInstanceOf(RustBridgeError);
    await expect(call).rejects.toMatchObject({
      bridgeError: { kind: "not_found", id: "42" },
    });
  });

  // Android rejects (rather than returning hand-built JSON) when the .so fails
  // to load; the message is passed through verbatim, quotes and all.
  it("reports a native rejection as a plain bridge failure", async () => {
    mockCallRust.mockRejectedValue(
      new Error(
        'native library unavailable: dlopen failed: library "librust_bridge.so" not found',
      ),
    );
    const { callRust, RustBridgeError } = load();

    const call = callRust({ cmd: "system", args: { type: "ping" } });
    await expect(call).rejects.not.toBeInstanceOf(RustBridgeError);
    await expect(call).rejects.toThrow(
      '[RustBridge] Bridge call failed: native library unavailable: dlopen failed: library "librust_bridge.so" not found',
    );
  });

  it("explains a missing native module (Expo Go / web)", async () => {
    mockRequireNativeModule.mockImplementation(() => {
      throw new Error("Cannot find native module 'RustBridge'");
    });
    const { callRust } = load();

    await expect(
      callRust({ cmd: "system", args: { type: "ping" } }),
    ).rejects.toThrow(/Expo Go and web don't include custom native code/);
  });

  it("does not resolve the native module just by being imported", () => {
    load();
    expect(mockRequireNativeModule).not.toHaveBeenCalled();
  });

  // The assertions here are the `@ts-expect-error`s, checked by
  // `npm run typecheck`: each fails the build if the line *does* compile.
  it("rejects malformed commands at compile time", () => {
    const { callRust } = load();
    const malformed = () => [
      // @ts-expect-error unknown command
      callRust({ cmd: "nope", args: {} }),
      // @ts-expect-error get_user requires an id
      callRust({ cmd: "user", args: { type: "get_user" } }),
      // @ts-expect-error factorial takes a number
      callRust({ cmd: "math", args: { type: "factorial", n: "5" } }),
    ];
    expect(typeof malformed).toBe("function");
  });

  it("infers the result type from the command", () => {
    const { callRust } = load();
    const typed = async () => {
      const user: User = await callRust({
        cmd: "user",
        args: { type: "get_user", id: "1" },
      });
      const pong: string = await callRust({
        cmd: "system",
        args: { type: "ping" },
      });
      // @ts-expect-error ping's data is a string, not a User
      const wrong: User = await callRust({
        cmd: "system",
        args: { type: "ping" },
      });
      return [user, pong, wrong];
    };
    expect(typeof typed).toBe("function");
  });

  // Rust doesn't tie a `*Responses` struct to its command enum, so this is
  // where a command without a declared response (or a misspelled one) fails.
  it("declares a response type for every command", () => {
    const missing: [MissingResponse] extends [never]
      ? "none"
      : MissingResponse = "none";
    expect(missing).toBe("none");
  });
});

describe("dev logging", () => {
  it("logs each call with its label and duration", async () => {
    mockCallRust.mockResolvedValue('{"status":"success","data":"pong"}');
    await load().callRust({ cmd: "system", args: { type: "ping" } });
    expect(console.debug).toHaveBeenCalledWith(
      expect.stringMatching(/^\[RustBridge\] system\.ping \(\d+ms\)$/),
    );
  });

  it("logs Rust errors with their params as info, not as warnings", async () => {
    mockCallRust.mockResolvedValue(
      '{"status":"error","kind":"not_found","id":"42"}',
    );
    await load()
      .callRust({ cmd: "user", args: { type: "get_user", id: "42" } })
      .catch(() => {});
    expect(console.info).toHaveBeenCalledWith(
      expect.stringMatching(/user\.get_user → not_found \{"id":"42"\}/),
    );
    expect(console.warn).not.toHaveBeenCalled();
  });

  it("tells you to rebuild when the native library doesn't know a command", async () => {
    mockCallRust.mockResolvedValue(
      '{"status":"error","kind":"invalid_command","reason":"unknown variant `brand_new`"}',
    );
    await load()
      .callRust({ cmd: "system", args: { type: "ping" } })
      .catch(() => {});
    expect(console.warn).toHaveBeenCalledWith(
      expect.stringMatching(/unknown variant `brand_new`[\s\S]*npm run ios/),
    );
  });

  it("warns about unexpected bridge failures", async () => {
    mockCallRust.mockRejectedValue(new Error("boom"));
    await load()
      .callRust({ cmd: "system", args: { type: "ping" } })
      .catch(() => {});
    expect(console.warn).toHaveBeenCalledWith(
      expect.stringMatching(/system\.ping failed .*boom/),
    );
  });
});

describe("dataDir", () => {
  it("returns the native module's data directory", () => {
    mockRequireNativeModule.mockReturnValue({
      callRust: mockCallRust,
      dataDir: "/data/user/0/app/files",
    });
    expect(load().dataDir()).toBe("/data/user/0/app/files");
  });
});

type CommandKey = BridgeCommand extends infer C
  ? C extends {
      cmd: infer K extends string;
      args: { type: infer V extends string };
    }
    ? `${K}.${V}`
    : never
  : never;
type ResponseKey = {
  [K in keyof BridgeResponses]: `${K}.${keyof BridgeResponses[K] & string}`;
}[keyof BridgeResponses];
type MissingResponse = Exclude<CommandKey, ResponseKey>;
