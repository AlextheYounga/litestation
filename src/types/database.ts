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
  schema: DatabaseSchema;
}

export interface BackendError {
  code: string;
  message: string;
}

export type SqlValue =
  | { kind: "null" }
  | { kind: "integer"; value: string }
  | { kind: "real"; value: number }
  | { kind: "text"; value: string }
  | { kind: "blob"; value: string };

export type RowIdentity =
  | { kind: "primaryKey"; values: SqlValue[] }
  | { kind: "rowid"; value: string };
export interface TableColumn {
  name: string;
  declaredType: string | null;
  notNull: boolean;
  defaultSql: string | null;
  primaryKeyPosition: number;
}
export interface TableMetadata {
  name: string;
  columns: TableColumn[];
  primaryKey: string[];
  identityKind: string;
  editable: boolean;
  nonEditableReason: string | null;
}
export interface BrowseRow {
  values: SqlValue[];
  identity: RowIdentity | null;
}
export interface BrowseTableResponse {
  metadata: TableMetadata;
  columns: string[];
  rows: BrowseRow[];
  offset: number;
  limit: number;
  hasMore: boolean;
}
export interface ExecuteSqlResponse {
  columns: string[];
  rows: SqlValue[][];
  affectedRows: number | null;
  truncated: boolean;
  durationMs: number;
  schemaChanged: boolean;
}
export interface ColumnValue {
  column: string;
  value: SqlValue;
}
export interface MutationResult {
  affectedRows: number;
  identity: RowIdentity | null;
}

export type ExportFormat = "json" | "csv";

export interface ImportSqlDumpResponse {
  schema: DatabaseSchema;
}

export interface ImportSqliteFileResponse {
  path: string;
  displayName: string;
}

export interface ExportRowsResponse {
  path: string;
  rowCount: number;
}

export interface ExportSqlDumpResponse {
  path: string;
  rowCount: number;
}
