import { describe, expect, it } from "vitest";
import { useDatabaseSessions } from "../../src/composables/useDatabaseSessions";
import type { DatabaseConnection } from "../../src/types/database";

function connection(id: number, path: string): DatabaseConnection {
  const segments = path.split("/");
  return {
    id,
    path,
    displayName: segments[segments.length - 1] || path,
    schema: { tables: [], views: [], indexes: [], triggers: [] },
  };
}

describe("database sessions", () => {
  it("keeps independent workspace state while switching databases", () => {
    const databaseSessions = useDatabaseSessions();
    const first = databaseSessions.add(connection(1, "/data/one.db")).session;
    first.selectedTable = "users";
    first.sqlSource = "SELECT * FROM users";

    databaseSessions.add(connection(2, "/data/two.db"));
    databaseSessions.activate(1);

    expect(databaseSessions.activeSession.value?.selectedTable).toBe("users");
    expect(databaseSessions.activeSession.value?.sqlSource).toBe("SELECT * FROM users");
  });

  it("activates an existing path instead of adding a duplicate session", () => {
    const databaseSessions = useDatabaseSessions();
    databaseSessions.add(connection(1, "/data/one.db"));
    databaseSessions.add(connection(2, "/data/two.db"));

    const result = databaseSessions.add(connection(3, "/data/one.db"));

    expect(result.added).toBe(false);
    expect(databaseSessions.sessions.value).toHaveLength(2);
    expect(databaseSessions.activeConnectionId.value).toBe(1);
  });

  it("activates a neighboring database when the active one closes", () => {
    const databaseSessions = useDatabaseSessions();
    databaseSessions.add(connection(1, "/data/one.db"));
    databaseSessions.add(connection(2, "/data/two.db"));
    databaseSessions.add(connection(3, "/data/three.db"));
    databaseSessions.activate(2);

    databaseSessions.remove(2);

    expect(databaseSessions.activeConnectionId.value).toBe(3);
    expect(databaseSessions.sessions.value.map((session) => session.connection.id)).toEqual([1, 3]);
  });
});
