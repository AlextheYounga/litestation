import { describe, expect, it } from "vitest";
import type { DatabaseConnection } from "../types/database";
import { CONNECTION_CATALOG_KEY, ConnectionCatalog, type CatalogStorage } from "./connectionCatalog";
import { openPath } from "./openPath";

class MemoryStorage implements CatalogStorage {
  values = new Map<string, string>();
  getItem(key: string): string | null { return this.values.get(key) ?? null; }
  setItem(key: string, value: string): void { this.values.set(key, value); }
}

function connection(path: string, mode: "readOnly" | "readWrite" = "readOnly"): DatabaseConnection {
  const segments = path.split(/[\\/]/);
  return { id: 1, path, displayName: segments[segments.length - 1] ?? path, mode, schema: { tables: [], views: [], indexes: [], triggers: [] } };
}

describe("ConnectionCatalog", () => {
  it("recovers from corrupt and unknown-version data", () => {
    const storage = new MemoryStorage();
    storage.setItem(CONNECTION_CATALOG_KEY, "not json");
    expect(new ConnectionCatalog(storage).load().entries).toEqual([]);
    storage.setItem(CONNECTION_CATALOG_KEY, JSON.stringify({ version: 99, entries: [] }));
    expect(new ConnectionCatalog(storage).load().entries).toEqual([]);
  });

  it("rejects parseable timestamps that are not canonical ISO values", () => {
    const storage = new MemoryStorage();
    storage.setItem(CONNECTION_CATALOG_KEY, JSON.stringify({
      version: 1,
      entries: [{
        id: "stale",
        path: "/stale.db",
        displayName: "stale.db",
        mode: "readOnly",
        saved: false,
        lastOpenedAt: "2025-01-01",
      }],
    }));

    expect(new ConnectionCatalog(storage).load().entries).toEqual([]);
  });

  it("orders saved and recent entries by timestamp", () => {
    const storage = new MemoryStorage();
    storage.setItem(CONNECTION_CATALOG_KEY, JSON.stringify({
      version: 1,
      entries: [
        { id: "recent-old", path: "/recent-old.db", displayName: "recent-old.db", mode: "readOnly", saved: false, lastOpenedAt: "2025-01-01T00:00:00.000Z" },
        { id: "saved-new", path: "/saved-new.db", displayName: "saved-new.db", mode: "readOnly", saved: true, lastOpenedAt: "2025-01-03T00:00:00.000Z" },
        { id: "saved-old", path: "/saved-old.db", displayName: "saved-old.db", mode: "readOnly", saved: true, lastOpenedAt: "2025-01-02T00:00:00.000Z" },
        { id: "recent-new", path: "/recent-new.db", displayName: "recent-new.db", mode: "readOnly", saved: false, lastOpenedAt: "2025-01-04T00:00:00.000Z" },
      ],
    }));

    expect(new ConnectionCatalog(storage).load().entries.map((entry) => entry.path)).toEqual([
      "/saved-new.db", "/saved-old.db", "/recent-new.db", "/recent-old.db",
    ]);
  });

  it("deduplicates and reorders an entry after a successful open", () => {
    let tick = 0;
    const catalog = new ConnectionCatalog(new MemoryStorage(), () => new Date(`2025-01-01T00:00:0${++tick}.000Z`));
    catalog.recordOpened(connection("/one.db"));
    catalog.recordOpened(connection("/two.db"));
    const state = catalog.recordOpened(connection("/one.db"));
    expect(state.entries.map((entry) => entry.path)).toEqual(["/one.db", "/two.db"]);
  });

  it("caps recent entries without evicting saved connections", () => {
    let tick = 0;
    const catalog = new ConnectionCatalog(new MemoryStorage(), () => new Date(1_700_000_000_000 + ++tick));
    catalog.recordOpened(connection("/saved.db"));
    catalog.save(connection("/saved.db"));
    for (let index = 0; index < 21; index += 1) catalog.recordOpened(connection(`/recent-${index}.db`));
    const state = catalog.load();
    expect(state.entries.filter((entry) => entry.saved).map((entry) => entry.path)).toEqual(["/saved.db"]);
    expect(state.entries.filter((entry) => !entry.saved)).toHaveLength(20);
    expect(state.entries.some((entry) => entry.path === "/recent-0.db")).toBe(false);
  });

  it("saves, unsaves, and forgets metadata without affecting other entries", () => {
    const catalog = new ConnectionCatalog(new MemoryStorage());
    catalog.recordOpened(connection("/keep.db"));
    let state = catalog.save(connection("/keep.db"));
    expect(state.entries[0].saved).toBe(true);
    state = catalog.unsave(state.entries[0]);
    expect(state.entries[0].saved).toBe(false);
    state = catalog.forget(state.entries[0]);
    expect(state.entries).toEqual([]);
  });

  it("keeps the same path opened in distinct modes", () => {
    const catalog = new ConnectionCatalog(new MemoryStorage());
    catalog.recordOpened(connection("C:\\data\\demo.db", "readOnly"));
    const state = catalog.recordOpened(connection("C:\\data\\demo.db", "readWrite"));
    expect(state.entries).toHaveLength(2);
  });

  it("keeps database work usable when storage writes fail", () => {
    const storage: CatalogStorage = { getItem: () => null, setItem: () => { throw new Error("blocked"); } };
    const state = new ConnectionCatalog(storage).recordOpened(connection("/demo.db"));
    expect(state.entries).toHaveLength(1);
    expect(state.persistenceWarning).toBe(true);
  });
});

describe("openPath", () => {
  it("records only successful opens and uses the stored entry mode", async () => {
    const catalog = new ConnectionCatalog(new MemoryStorage());
    const openDatabase = async (path: string, mode: "readOnly" | "readWrite") => connection(path, mode);
    await openPath("/demo.db", "readWrite", openDatabase, catalog);
    expect(catalog.load().entries[0].mode).toBe("readWrite");

    await expect(openPath("/missing.db", "readOnly", async () => { throw new Error("missing"); }, catalog)).rejects.toThrow("missing");
    expect(catalog.load().entries.map((entry) => entry.path)).toEqual(["/demo.db"]);
  });
});
