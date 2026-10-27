import type { DatabaseConnection } from "../types/database";
import type { ConnectionCatalogState } from "../types/connectionCatalog";
import { ConnectionCatalog } from "./connectionCatalog";

/** Opens a database and only records it after the backend has succeeded. */
export async function openPath(
  path: string,
  openDatabase: (path: string) => Promise<DatabaseConnection>,
  catalog: ConnectionCatalog,
): Promise<{ connection: DatabaseConnection; catalogState: ConnectionCatalogState }> {
  const connection = await openDatabase(path);
  const state = catalog.recordOpened(connection);
  return { connection, catalogState: state };
}
