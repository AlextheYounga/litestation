import { computed, ref } from "vue";
import type { DatabaseConnection } from "../types/database";

const DEFAULT_SQL = "SELECT name FROM sqlite_master ORDER BY name;";

export interface DatabaseSession {
  connection: DatabaseConnection;
  selectedTable: string | null;
  sqlSource: string;
  workspaceBusy: boolean;
  toolsBusy: boolean;
  schemaRefreshing: boolean;
  errorMessage: string;
}

export function useDatabaseSessions() {
  const sessions = ref<DatabaseSession[]>([]);
  const activeConnectionId = ref<number | null>(null);
  const activeSession = computed(
    () => sessions.value.find((session) => session.connection.id === activeConnectionId.value) ?? null,
  );

  function activate(connectionId: number): void {
    if (sessions.value.some((session) => session.connection.id === connectionId)) {
      activeConnectionId.value = connectionId;
    }
  }

  function add(connection: DatabaseConnection): { session: DatabaseSession; added: boolean } {
    const existing = sessions.value.find((session) => session.connection.path === connection.path);
    if (existing) {
      activeConnectionId.value = existing.connection.id;
      return { session: existing, added: false };
    }

    const session: DatabaseSession = {
      connection,
      selectedTable: null,
      sqlSource: DEFAULT_SQL,
      workspaceBusy: false,
      toolsBusy: false,
      schemaRefreshing: false,
      errorMessage: "",
    };
    sessions.value.push(session);
    activeConnectionId.value = connection.id;
    return { session, added: true };
  }

  function remove(connectionId: number): void {
    const index = sessions.value.findIndex((session) => session.connection.id === connectionId);
    if (index < 0) return;

    const wasActive = activeConnectionId.value === connectionId;
    sessions.value.splice(index, 1);
    if (wasActive) {
      activeConnectionId.value = sessions.value[index]?.connection.id
        ?? sessions.value[index - 1]?.connection.id
        ?? null;
    }
  }

  return { sessions, activeConnectionId, activeSession, activate, add, remove };
}
