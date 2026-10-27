import { describe, expect, it } from "vitest";
import { displayValue, parseValue, validBase64 } from "./sqliteValue";
describe("SQLite values", () => {
  it("displays null and preserves large integers", () => {
    expect(displayValue({ kind: "null" })).toBe("NULL");
    expect(parseValue("integer", "9223372036854775807")).toEqual({
      kind: "integer",
      value: "9223372036854775807",
    });
  });
  it("keeps empty text and validates real values", () => {
    expect(parseValue("text", "")).toEqual({ kind: "text", value: "" });
    expect(parseValue("real", "1.25")).toEqual({ kind: "real", value: 1.25 });
    expect(parseValue("real", "NaN")).toBe("Enter a finite real number.");
  });
  it("validates base64", () => {
    expect(validBase64("aGVsbG8=")).toBe(true);
    expect(validBase64("not base64!")).toBe(false);
  });
});
