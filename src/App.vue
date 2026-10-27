<script setup lang="ts">
import { computed, ref } from "vue";
import SchemaBrowser from "./components/SchemaBrowser.vue";
import DatabaseTools from "./components/DatabaseTools.vue";
import SqlWorkspace from "./components/SqlWorkspace.vue";
import TableWorkspace from "./components/TableWorkspace.vue";
import {
  chooseSqliteFile,
  closeDatabase,
  normalizeBackendError,
  openLocalDatabase,
  refreshSchema,
} from "./services/database";
import type { DatabaseConnection } from "./types/database";
import type { ConnectionCatalogEntry } from "./types/connectionCatalog";
import { browserCatalogStorage, ConnectionCatalog } from "./services/connectionCatalog";
import { openPath as openDatabasePath } from "./services/openPath";
import { applyImportedSchema, selectedTableExists } from "./utils/schema";

const selectedPath = ref("");
const connection = ref<DatabaseConnection | null>(null);
const opening = ref(false);
const pickerBusy = ref(false);
const closing = ref(false);
const schemaRefreshing = ref(false);
const workspaceBusy = ref(false);
const toolsBusy = ref(false);
const errorMessage = ref("");
const persistenceWarning = ref(false);
const selectedTable = ref<string | null>(null);
const sqlSource = ref("SELECT name FROM sqlite_master ORDER BY name;");
const catalog = new ConnectionCatalog(browserCatalogStorage());
const catalogEntries = ref<ConnectionCatalogEntry[]>([]);
let connectionGeneration = 0;

const hasConnection = computed(() => connection.value !== null);
const savedConnections = computed(() =>
  catalogEntries.value.filter((entry) => entry.saved),
);
const recentConnections = computed(() =>
  catalogEntries.value.filter((entry) => !entry.saved),
);
const activeCatalogEntry = computed(() => {
  const current = connection.value;
  if (!current) return null;
  return catalogEntries.value.find((entry) => entry.path === current.path) ?? null;
});
const controlsDisabled = computed(
  () =>
    opening.value ||
    pickerBusy.value ||
    closing.value ||
    workspaceBusy.value ||
    toolsBusy.value ||
    schemaRefreshing.value,
);

applyCatalogState(catalog.load());

async function openDatabase(): Promise<void> {
  if (opening.value || pickerBusy.value || connection.value) return;
  const pickerGeneration = ++connectionGeneration;
  pickerBusy.value = true;
  let path: string | null;
  try {
    path = await chooseSqliteFile();
  } catch (error) {
    if (pickerGeneration === connectionGeneration)
      errorMessage.value = normalizeBackendError(error).message;
    pickerBusy.value = false;
    return;
  }
  if (pickerGeneration !== connectionGeneration) {
    pickerBusy.value = false;
    return;
  }
  // Release the picker guard before handing the selected path to openPath.
  pickerBusy.value = false;
  if (!path) return;
  await openPath(path);
}

async function openPath(path: string): Promise<void> {
  if (opening.value || pickerBusy.value || connection.value) return;
  const generation = ++connectionGeneration;
  selectedPath.value = path;
  opening.value = true;
  errorMessage.value = "";
  try {
    const opened = await openDatabasePath(path, openLocalDatabase, catalog);
    applyCatalogState(opened.catalogState);
    if (generation === connectionGeneration) connection.value = opened.connection;
  } catch (error) {
    if (generation === connectionGeneration)
      errorMessage.value = normalizeBackendError(error).message;
  } finally {
    if (generation === connectionGeneration) opening.value = false;
  }
}

function saveActiveConnection(): void {
  if (!connection.value || controlsDisabled.value) return;
  applyCatalogState(catalog.save(connection.value));
}

function unsaveConnection(entry: ConnectionCatalogEntry): void {
  if (controlsDisabled.value) return;
  applyCatalogState(catalog.unsave(entry));
}

function forgetConnection(entry: ConnectionCatalogEntry): void {
  if (controlsDisabled.value) return;
  applyCatalogState(catalog.forget(entry));
}

function applyCatalogState(state: import("./types/connectionCatalog").ConnectionCatalogState): void {
  catalogEntries.value = state.entries;
  persistenceWarning.value = state.persistenceWarning;
}

function formatOpenedAt(timestamp: string): string {
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(timestamp));
}

async function closeConnection(): Promise<void> {
  if (!connection.value || controlsDisabled.value) return;
  const current = connection.value;
  const generation = ++connectionGeneration;
  closing.value = true;
  errorMessage.value = "";
  try {
    await closeDatabase(current.id);
    if (generation !== connectionGeneration) return;
    connection.value = null;
    selectedTable.value = null;
    selectedPath.value = "";
    workspaceBusy.value = false;
    toolsBusy.value = false;
    errorMessage.value = "";
  } catch (error) {
    if (generation === connectionGeneration)
      errorMessage.value = normalizeBackendError(error).message;
  } finally {
    if (generation === connectionGeneration) closing.value = false;
  }
}

function applySqlImport(schema: import("./types/database").DatabaseSchema): void {
  const current = connection.value;
  if (!current) return;
  const imported = applyImportedSchema(current, schema);
  if (connection.value?.id !== current.id) return;
  connection.value = imported;
  if (!selectedTableExists(schema, selectedTable.value)) selectedTable.value = null;
}

async function updateSchema(): Promise<void> {
  const current = connection.value;
  if (!current) return;
  const generation = connectionGeneration;
  schemaRefreshing.value = true;
  try {
    const schema = await refreshSchema(current.id);
    if (
      generation === connectionGeneration &&
      connection.value?.id === current.id
    ) {
      connection.value = { ...current, schema };
    }
  } catch (error) {
    if (generation === connectionGeneration)
      errorMessage.value = normalizeBackendError(error).message;
  } finally {
    if (generation === connectionGeneration) schemaRefreshing.value = false;
  }
}
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <span class="brand"><b>L</b> LiteStation</span
      ><span>SQLite workspace</span>
    </header>
    <section v-if="!hasConnection" class="welcome">
      <div>
        <p class="eyebrow">Local database</p>
        <h1>Open a database<br /><em>with confidence.</em></h1>
        <p>Your database stays on this device.</p>
      </div>
      <form class="open-card" @submit.prevent="openDatabase">
        <p class="path">
          {{ selectedPath || "Choose an existing SQLite file" }}
        </p>
        <button class="primary-button" :disabled="opening || pickerBusy">
          {{ pickerBusy ? "Choosing…" : opening ? "Opening…" : "Choose database" }}
        </button>
        <p v-if="errorMessage" class="error-message" role="alert">
          {{ errorMessage }}
        </p>
        <p v-if="persistenceWarning" class="persistence-warning" role="status">
          Connection metadata could not be saved. Database work still works; check local storage permissions.
        </p>
      </form>
      <section
        v-if="savedConnections.length || recentConnections.length"
        class="connection-catalog"
        aria-label="Saved and recent databases"
      >
        <div v-if="savedConnections.length" class="catalog-section">
          <h2>Saved</h2>
          <ul class="catalog-list">
            <li v-for="entry in savedConnections" :key="entry.id">
              <button type="button" class="catalog-open" :disabled="controlsDisabled" @click="openPath(entry.path)">
                <strong>{{ entry.displayName }}</strong><span>{{ entry.path }}</span>
              </button>
              <button type="button" class="catalog-forget" :disabled="controlsDisabled" :aria-label="`Forget saved connection ${entry.displayName}; database files are not deleted`" @click="forgetConnection(entry)">Forget</button>
            </li>
          </ul>
        </div>
        <div v-if="recentConnections.length" class="catalog-section">
          <h2>Recent</h2>
          <ul class="catalog-list">
            <li v-for="entry in recentConnections" :key="entry.id">
              <button type="button" class="catalog-open" :disabled="controlsDisabled" @click="openPath(entry.path)">
                <strong>{{ entry.displayName }}</strong><span>{{ entry.path }}</span><small>{{ formatOpenedAt(entry.lastOpenedAt) }}</small>
              </button>
              <button type="button" class="catalog-forget" :disabled="controlsDisabled" :aria-label="`Forget recent connection ${entry.displayName}; database files are not deleted`" @click="forgetConnection(entry)">Forget</button>
            </li>
          </ul>
        </div>
        <p class="catalog-note">Forget removes LiteStation metadata only. It never deletes database files.</p>
      </section>
    </section>
    <section v-else class="workspace">
      <aside class="connection-sidebar">
        <p class="eyebrow">Active connection</p>
        <h1>{{ connection?.displayName }}</h1>
        <p class="path">{{ connection?.path }}</p>
        <button
          v-if="activeCatalogEntry?.saved"
          type="button"
          class="secondary"
          :disabled="controlsDisabled"
          @click="unsaveConnection(activeCatalogEntry)"
        >
          Remove from saved connections
        </button>
        <button
          v-else
          type="button"
          class="secondary"
          :disabled="controlsDisabled"
          @click="saveActiveConnection"
        >
          Save connection
        </button>
        <button
          v-if="activeCatalogEntry"
          type="button"
          class="secondary catalog-forget"
          :disabled="controlsDisabled"
          @click="forgetConnection(activeCatalogEntry)"
        >
          Forget metadata (keeps file)
        </button>
        <button
          class="secondary"
          :disabled="controlsDisabled"
          @click="closeConnection"
        >
          Close connection
        </button>
        <nav class="workspace-navigation" aria-label="Workspace views">
          <button
            type="button"
            class="workspace-navigation-button"
            :class="{ active: selectedTable === null }"
            :aria-current="selectedTable === null ? 'page' : undefined"
            :disabled="controlsDisabled"
            @click="selectedTable = null"
          >
            SQL Query
          </button>
        </nav>
        <SchemaBrowser
          v-if="connection"
          :schema="connection.schema"
          :selected-table="selectedTable"
          :disabled="controlsDisabled"
          @select-table="selectedTable = $event"
        />
        <DatabaseTools
          v-if="connection"
          :connection-id="connection.id"
          @busy-change="toolsBusy = $event"
          @schema-imported="applySqlImport"
        />
      </aside>
      <section class="content">
        <p v-if="persistenceWarning" class="persistence-warning" role="status">
          Connection metadata could not be saved. Database work still works; check local storage permissions.
        </p>
        <p v-if="errorMessage" class="error-message" role="alert">
          {{ errorMessage }}
        </p>
        <TableWorkspace
          v-if="connection && selectedTable"
          :key="`${connection.id}:${selectedTable}`"
          :connection-id="connection.id"
          :table-name="selectedTable"
          @busy-change="workspaceBusy = $event"
        />
        <SqlWorkspace
          v-else-if="connection"
          :key="connection.id"
          :connection-id="connection.id"
          v-model="sqlSource"
          @schema-changed="updateSchema"
          @busy-change="workspaceBusy = $event"
        />
      </section>
    </section>
  </main>
</template>
