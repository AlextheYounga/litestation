import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  BackendError,
  ConnectionMode,
  DatabaseConnection,
} from "../types/database";

const SQLITE_FILTER = [
  { name: "SQLite database", extensions: ["sqlite", "sqlite3", "db"] },
];

export async function chooseSqliteFile(): Promise<string | null> {
  const selection = await open({
    directory: false,
    multiple: false,
    filters: SQLITE_FILTER,
    title: "Choose a SQLite database",
  });

  return typeof selection === "string" ? selection : null;
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
  return typeof candidate.code === "string" && typeof candidate.message === "string";
}
