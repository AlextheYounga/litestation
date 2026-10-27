import type { DatabaseConnection, DatabaseSchema } from "../types/database";

export function applyImportedSchema(
  connection: DatabaseConnection,
  schema: DatabaseSchema,
): DatabaseConnection {
  return { ...connection, schema };
}

export function selectedTableExists(schema: DatabaseSchema, selectedTable: string | null): boolean {
  return selectedTable === null || schema.tables.some((table) => table.name === selectedTable);
}
