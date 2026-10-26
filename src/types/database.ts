export type ConnectionMode = "readOnly" | "readWrite";

export interface SchemaItem {
  name: string;
  tableName: string;
  sql: string | null;
}

export interface DatabaseSchema {
  tables: SchemaItem[];
  views: SchemaItem[];
  indexes: SchemaItem[];
  triggers: SchemaItem[];
}

export interface DatabaseConnection {
  id: number;
  path: string;
  displayName: string;
  mode: ConnectionMode;
  schema: DatabaseSchema;
}

export interface BackendError {
  code: string;
  message: string;
}
