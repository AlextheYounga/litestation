import { describe, expect, it } from "vitest";
import { applyImportedSchema, selectedTableExists } from "../../src/utils/schema";
import type { DatabaseConnection, DatabaseSchema } from "../../src/types/database";

const schema: DatabaseSchema = { tables: [{ name: "accounts", tableName: "accounts", sql: null }], views: [], indexes: [], triggers: [] };
const connection: DatabaseConnection = { id: 1, path: "/tmp/demo.db", displayName: "demo", schema: { tables: [], views: [], indexes: [], triggers: [] } };

describe("imported schema application", () => {
  it("replaces the active schema and clears selections for removed tables", () => {
    expect(applyImportedSchema(connection, schema).schema).toBe(schema);
    expect(selectedTableExists(schema, "removed_table")).toBe(false);
    expect(selectedTableExists(schema, "accounts")).toBe(true);
  });
});
