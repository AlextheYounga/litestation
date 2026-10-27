import type { DatabaseConnection } from "../types/database";
import type {
  ConnectionCatalogEntry,
  ConnectionCatalogState,
} from "../types/connectionCatalog";

export const CONNECTION_CATALOG_KEY = "litestation.connections.v1";
const CATALOG_VERSION = 1;
const RECENT_LIMIT = 20;

export interface CatalogStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export class ConnectionCatalog {
  private entries: ConnectionCatalogEntry[] = [];

  constructor(
    private readonly storage: CatalogStorage | null,
    private readonly now: () => Date = () => new Date(),
  ) {}

  load(): ConnectionCatalogState {
    if (!this.storage) return this.state(true);

    try {
      const serialized = this.storage.getItem(CONNECTION_CATALOG_KEY);
      if (!serialized) return this.state(false);
      this.entries = readCatalog(serialized);
      return this.state(false);
    } catch {
      this.entries = [];
      return this.state(true);
    }
  }

  recordOpened(connection: DatabaseConnection): ConnectionCatalogState {
    const entry = this.find(connection.path);
    const next: ConnectionCatalogEntry = {
      id: connectionId(connection.path),
      path: connection.path,
      displayName: connection.displayName,
      saved: entry?.saved ?? false,
      lastOpenedAt: this.now().toISOString(),
    };
    this.entries = limitRecents([next, ...this.entries.filter((item) => item.id !== next.id)]);
    return this.persist();
  }

  save(connection: DatabaseConnection): ConnectionCatalogState {
    const entry = this.find(connection.path);
    const next: ConnectionCatalogEntry = {
      id: connectionId(connection.path),
      path: connection.path,
      displayName: connection.displayName,
      saved: true,
      lastOpenedAt: entry?.lastOpenedAt ?? this.now().toISOString(),
    };
    this.entries = limitRecents([next, ...this.entries.filter((item) => item.id !== next.id)]);
    return this.persist();
  }

  unsave(entry: ConnectionCatalogEntry): ConnectionCatalogState {
    this.entries = this.entries.map((item) =>
      item.id === entry.id ? { ...item, saved: false } : item,
    );
    this.entries = limitRecents(this.entries);
    return this.persist();
  }

  forget(entry: ConnectionCatalogEntry): ConnectionCatalogState {
    this.entries = this.entries.filter((item) => item.id !== entry.id);
    return this.persist();
  }

  private find(path: string): ConnectionCatalogEntry | undefined {
    return this.entries.find((entry) => entry.id === connectionId(path));
  }

  private persist(): ConnectionCatalogState {
    if (!this.storage) return this.state(true);
    try {
      this.storage.setItem(
        CONNECTION_CATALOG_KEY,
        JSON.stringify({ version: CATALOG_VERSION, entries: this.entries }),
      );
      return this.state(false);
    } catch {
      return this.state(true);
    }
  }

  private state(persistenceWarning: boolean): ConnectionCatalogState {
    return { entries: [...this.entries], persistenceWarning };
  }
}

export function connectionId(path: string): string {
  return path;
}

export function browserCatalogStorage(): CatalogStorage | null {
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function readCatalog(serialized: string): ConnectionCatalogEntry[] {
  try {
    const parsed: unknown = JSON.parse(serialized);
    if (!isCatalog(parsed)) return [];
    const entries = parsed.entries.filter(isEntry).map((entry) => ({
      id: connectionId(entry.path),
      path: entry.path,
      displayName: entry.displayName,
      saved: entry.saved,
      lastOpenedAt: entry.lastOpenedAt,
    }));
    return limitRecents(mergeEntries(entries));
  } catch {
    return [];
  }
}

function isCatalog(value: unknown): value is { version: number; entries: unknown[] } {
  return !!value
    && typeof value === "object"
    && (value as { version?: unknown }).version === CATALOG_VERSION
    && Array.isArray((value as { entries?: unknown }).entries);
}

function isEntry(value: unknown): value is ConnectionCatalogEntry {
  if (!value || typeof value !== "object") return false;
  const entry = value as Partial<ConnectionCatalogEntry>;
  return typeof entry.path === "string"
    && typeof entry.displayName === "string"
    && typeof entry.saved === "boolean"
    && typeof entry.lastOpenedAt === "string"
    && isCanonicalTimestamp(entry.lastOpenedAt);
}

function isCanonicalTimestamp(value: string): boolean {
  const timestamp = new Date(value);
  return !Number.isNaN(timestamp.getTime()) && timestamp.toISOString() === value;
}

function mergeEntries(entries: ConnectionCatalogEntry[]): ConnectionCatalogEntry[] {
  const merged = new Map<string, ConnectionCatalogEntry>();
  for (const entry of entries) {
    const existing = merged.get(entry.id);
    if (!existing) {
      merged.set(entry.id, entry);
      continue;
    }
    const newest = Date.parse(entry.lastOpenedAt) > Date.parse(existing.lastOpenedAt) ? entry : existing;
    merged.set(entry.id, { ...newest, saved: existing.saved || entry.saved });
  }
  return [...merged.values()];
}

function limitRecents(entries: ConnectionCatalogEntry[]): ConnectionCatalogEntry[] {
  const byNewest = (left: ConnectionCatalogEntry, right: ConnectionCatalogEntry) => {
    const timestampDifference = Date.parse(right.lastOpenedAt) - Date.parse(left.lastOpenedAt);
    return timestampDifference || left.id.localeCompare(right.id);
  };
  const saved = entries.filter((entry) => entry.saved).sort(byNewest);
  const recent = entries.filter((entry) => !entry.saved).sort(byNewest).slice(0, RECENT_LIMIT);
  return [...saved, ...recent];
}
