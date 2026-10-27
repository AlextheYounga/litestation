import type { ConnectionMode, DatabaseConnection } from "../types/database";
import type { ConnectionCatalogState } from "../types/connectionCatalog";
import { ConnectionCatalog } from "./connectionCatalog";

/** Opens a database and only records it after the backend has succeeded. */
export async function openPath(
  path: string,
  mode: ConnectionMode,
  openDatabase: (path: string, mode: ConnectionMode) => Promise<DatabaseConnection>,
  catalog: ConnectionCatalog,
): Promise<{ connection: DatabaseConnection; catalogState: ConnectionCatalogState }> {
  const connection = await openDatabase(path, mode);
  const state = catalog.recordOpened(connection);
  return { connection, catalogState: state };
}
