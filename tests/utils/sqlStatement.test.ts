import { describe, expect, it } from "vitest";
import {
  firstMeaningfulKeyword,
  isPotentiallyDestructive,
  runAllStatement,
  statementAtCursor,
  statements,
} from "../../src/utils/sqlStatement";
describe("SQL statement scanner", () => {
  it("finds the following statement at a shared boundary and the last at document end", () => {
    const sql = "select 1;select 2";
    expect(statementAtCursor(sql, 8)).toBe("select 1;");
    expect(statementAtCursor(sql, 9)).toBe("select 2");
    expect(statementAtCursor(sql, sql.length)).toBe("select 2");
  });

  it("ignores semicolons in SQL literals, identifiers, and comments", () => {
    const sql = "select ';', \"x;y\", `z;q`, [a;b] /* ; */ -- ;\n; select 2";
    expect(statements(sql)).toHaveLength(2);
  });

  it("handles comments and quote escapes at statement edges", () => {
    expect(
      statements("-- leading ;\nselect 'it''s; ok'; /* trailing ; */"),
    ).toHaveLength(1);
    expect(statements("/* only ; */ -- still only ;")).toEqual([]);
  });

  it("identifies a single statement for run all", () => {
    expect(runAllStatement("select 1;")).toBe("select 1;");
    expect(runAllStatement("select 1; select 2")).toBeNull();
  });

  it("extracts keywords after leading comments and classifies mutations", () => {
    expect(firstMeaningfulKeyword(" \n/* ignored */ -- also ignored\n update users set name = 'x'"))
      .toBe("UPDATE");
    expect(isPotentiallyDestructive("-- comment\nDrOp table users")).toBe(true);
    expect(isPotentiallyDestructive("pragma journal_mode = WAL")).toBe(true);
  });

  it("does not classify ordinary reads as destructive", () => {
    expect(isPotentiallyDestructive("SELECT * FROM users")).toBe(false);
    expect(isPotentiallyDestructive("/* note */ EXPLAIN SELECT 1")).toBe(false);
    expect(isPotentiallyDestructive("WITH rows AS (SELECT 1) SELECT * FROM rows")).toBe(false);
  });
});
