import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  BackendError,
  ConnectionMode,
  DatabaseConnection,
  BrowseTableResponse,
  ColumnValue,
  ExecuteSqlResponse,
  MutationResult,
  RowIdentity,
  TableMetadata,
  ExportFormat,
  ExportRowsResponse,
  ExportSqlDumpResponse,
  ImportSqlDumpResponse,
  ImportSqliteFileResponse,
  SqlValue,
} from "../types/database";

const SQLITE_FILTER = [
  { name: "SQLite database", extensions: ["sqlite", "sqlite3", "db"] },
];
const SQL_FILTER = [{ name: "SQL dump", extensions: ["sql"] }];
const EXPORT_FILTERS: Record<ExportFormat, { name: string; extensions: string[] }[]> = {
  json: [{ name: "JSON file", extensions: ["json"] }],
  csv: [{ name: "CSV file", extensions: ["csv"] }],
};

export async function chooseSqliteFile(): Promise<string | null> {
  const selection = await open({
    directory: false,
    multiple: false,
    filters: SQLITE_FILTER,
    title: "Choose a SQLite database",
  });

  return typeof selection === "string" ? selection : null;
}

export async function chooseSqlDump(): Promise<string | null> {
  const selection = await open({ directory: false, multiple: false, filters: SQL_FILTER, title: "Choose a SQL dump" });
  return typeof selection === "string" ? selection : null;
}

export async function chooseSqliteCopyDestination(): Promise<string | null> {
  return save({ filters: SQLITE_FILTER, title: "Create SQLite copy" });
}

export async function chooseExportDestination(format: ExportFormat): Promise<string | null> {
  return save({ filters: EXPORT_FILTERS[format], title: `Export ${format.toUpperCase()}` });
}

export async function chooseSqlDumpDestination(): Promise<string | null> {
  return save({ filters: SQL_FILTER, title: "Export SQL dump" });
}

export function executeSql(request: {
  connectionId: number;
  sql: string;
  maxRows?: number;
}): Promise<ExecuteSqlResponse> {
  return invoke("execute_sql", { request });
}
export function getTableMetadata(request: {
  connectionId: number;
  tableName: string;
}): Promise<TableMetadata> {
  return invoke("get_table_metadata", { request });
}
export function browseTable(request: {
  connectionId: number;
  tableName: string;
  offset: number;
  limit: number;
}): Promise<BrowseTableResponse> {
  return invoke("browse_table", { request });
}
export function refreshSchema(
  connectionId: number,
): Promise<import("../types/database").DatabaseSchema> {
  return invoke("refresh_schema", { connectionId });
}
export function insertRow(request: {
  connectionId: number;
  tableName: string;
  values: ColumnValue[];
}): Promise<MutationResult> {
  return invoke("insert_row", { request });
}
export function updateRow(request: {
  connectionId: number;
  tableName: string;
  identity: RowIdentity;
  expected: ColumnValue[];
  changes: ColumnValue[];
}): Promise<MutationResult> {
  return invoke("update_row", { request });
}
export function deleteRow(request: {
  connectionId: number;
  tableName: string;
  identity: RowIdentity;
  expected: ColumnValue[];
}): Promise<MutationResult> {
  return invoke("delete_row", { request });
}

export function openLocalDatabase(
  path: string,
  mode: ConnectionMode,
): Promise<DatabaseConnection> {
  return invoke<DatabaseConnection>("open_local_database", {
    request: { path, mode },
  });
}

export function closeDatabase(connectionId: number): Promise<void> {
  return invoke("close_database", { connectionId });
}

export function importSqlDump(request: { connectionId: number; sourcePath: string }): Promise<ImportSqlDumpResponse> {
  return invoke("import_sql_dump", { request });
}

export function importSqliteFile(request: { sourcePath: string; destinationPath: string }): Promise<ImportSqliteFileResponse> {
  return invoke("import_sqlite_file", { request });
}

export function exportRows(request: { columns: string[]; rows: SqlValue[][]; format: ExportFormat; destinationPath: string }): Promise<ExportRowsResponse> {
  return invoke("export_rows", { request });
}

export function exportSqlDump(request: { connectionId: number; destinationPath: string }): Promise<ExportSqlDumpResponse> {
  return invoke("export_sql_dump", { request });
}

export function normalizeBackendError(error: unknown): BackendError {
  if (isBackendError(error)) {
    return error;
  }

  if (error instanceof Error && error.message) {
    return { code: "unexpected_error", message: error.message };
  }

  return {
    code: "unexpected_error",
    message: "The database operation could not be completed.",
  };
}

function isBackendError(error: unknown): error is BackendError {
  if (!error || typeof error !== "object") {
    return false;
  }

  const candidate = error as Partial<BackendError>;
  return (
    typeof candidate.code === "string" && typeof candidate.message === "string"
  );
}
