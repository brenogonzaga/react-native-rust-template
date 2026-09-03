import { RustBridgeError } from "rust-bridge";
import { describeError } from "./describeError";
import { SUPPORTED_LOCALES } from "../i18n";

describe("describeError", () => {
  it("translates a RustBridgeError via i18n", () => {
    const err = new RustBridgeError({ kind: "not_found", id: "42" });
    expect(describeError(err, "en")).toBe("User 42 was not found");
  });

  it("translates using the given locale, not any global i18n state", () => {
    const err = new RustBridgeError({ kind: "not_found", id: "42" });
    expect(describeError(err, "pt")).toBe("Usuário 42 não encontrado");
  });

  it("falls back to the plain message for a regular Error", () => {
    expect(describeError(new Error("boom"), "en")).toBe("boom");
  });

  it("stringifies anything else", () => {
    expect(describeError("weird", "en")).toBe("weird");
  });
});

describe("i18n setup", () => {
  it("ships en and pt as supported locales", () => {
    expect(SUPPORTED_LOCALES).toEqual(expect.arrayContaining(["en", "pt"]));
  });
});
