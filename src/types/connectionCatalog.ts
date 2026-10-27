import type { ConnectionMode } from "./database";

/** Non-secret metadata retained locally for reopening a database. */
export interface ConnectionCatalogEntry {
  id: string;
  path: string;
  displayName: string;
  mode: ConnectionMode;
  saved: boolean;
  lastOpenedAt: string;
}

export interface ConnectionCatalogState {
  entries: ConnectionCatalogEntry[];
  persistenceWarning: boolean;
}
