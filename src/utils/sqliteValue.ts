import type { SqlValue } from "../types/database";
export type EditableKind = SqlValue["kind"];
export function displayValue(value: SqlValue): string {
  if (value.kind === "null") return "NULL";
  if (value.kind === "blob") return `BLOB · ${base64Bytes(value.value)} bytes`;
  return String(value.value);
}
export function base64Bytes(value: string): number {
  const clean = value.replace(/=+$/, "");
  return Math.floor((clean.length * 3) / 4);
}
export function defaultKind(declaredType: string | null): EditableKind {
  const type = (declaredType ?? "").toUpperCase();
  return /INT/.test(type)
    ? "integer"
    : /REAL|FLOA|DOUB/.test(type)
      ? "real"
      : /BLOB/.test(type)
        ? "blob"
        : "text";
}
export function parseValue(kind: EditableKind, raw: string): SqlValue | string {
  if (kind === "null") return { kind };
  if (kind === "integer")
    return /^[-+]?\d+$/.test(raw) ? { kind, value: raw } : "Enter an integer.";
  if (kind === "real") {
    const value = Number(raw);
    return raw.trim() !== "" && Number.isFinite(value)
      ? { kind, value }
      : "Enter a finite real number.";
  }
  if (kind === "blob")
    return validBase64(raw) ? { kind, value: raw } : "Enter valid base64.";
  return { kind, value: raw };
}
export function validBase64(value: string): boolean {
  return (
    value === "" ||
    /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(
      value,
    )
  );
}
export function sameValue(a: SqlValue, b: SqlValue): boolean {
  return (
    a.kind === b.kind &&
    (a.kind === "null" ||
      a.value === (b as Exclude<SqlValue, { kind: "null" }>).value)
  );
}
