<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import {
  Bookmark,
  BookmarkX,
  Database,
  RefreshCw,
  SquareTerminal,
  Unplug,
  X,
} from "@lucide/vue";
import SchemaBrowser from "./components/SchemaBrowser.vue";
import DatabaseTools from "./components/DatabaseTools.vue";
import DatabaseTabs from "./components/DatabaseTabs.vue";
import SqlWorkspace from "./components/SqlWorkspace.vue";
import TableWorkspace from "./components/TableWorkspace.vue";
import WelcomeBackdrop from "./components/WelcomeBackdrop.vue";
import WelcomeScreen from "./components/WelcomeScreen.vue";
import {
  chooseSqliteFile,
  closeDatabase,
  normalizeBackendError,
  openLocalDatabase,
  refreshSchema,
} from "./services/database";
import type { ConnectionCatalogEntry } from "./types/connectionCatalog";
import { ConnectionCatalog, browserCatalogStorage } from "./services/connectionCatalog";
import { openPath as openDatabasePath } from "./services/openPath";
import { applyImportedSchema, selectedTableExists } from "./utils/schema";
import { useDatabaseSessions } from "./composables/useDatabaseSessions";
import { onDatabasePathsAvailable, takePendingDatabasePaths } from "./services/launch";

const selectedPath = ref(""),
 opening = ref(false),
 pickerBusy = ref(false),
 closingConnectionIds = ref(new Set<number>()),
 openErrorMessage = ref(""),
 persistenceWarning = ref(false),
 catalog = new ConnectionCatalog(browserCatalogStorage()),
 catalogEntries = ref<ConnectionCatalogEntry[]>([]),
 databaseSessions = useDatabaseSessions(),
 { sessions, activeConnectionId, activeSession } = databaseSessions;
let openGeneration = 0,
 stopLaunchPathListener: (() => void) | null = null,
 launchPathWork = Promise.resolve(),
 openPathWork = Promise.resolve();

const connection = computed(() => activeSession.value?.connection ?? null),
 hasConnection = computed(() => sessions.value.length > 0),
 selectedTable = computed({
  get: () => activeSession.value?.selectedTable ?? null,
  set: (table: string | null) => {
    if (activeSession.value) {activeSession.value.selectedTable = table;}
  },
}),
 sqlSource = computed({
  get: () => activeSession.value?.sqlSource ?? "",
  set: (sql: string) => {
    if (activeSession.value) {activeSession.value.sqlSource = sql;}
  },
}),
 savedConnections = computed(() =>
  catalogEntries.value.filter((entry) => entry.saved),
),
 recentConnections = computed(() =>
  catalogEntries.value.filter((entry) => !entry.saved),
),
 activeCatalogEntry = computed(() => {
  const current = connection.value;
  if (!current) {return null;}
  return catalogEntries.value.find((entry) => entry.path === current.path) ?? null;
}),
 controlsDisabled = computed(
  () =>
    opening.value ||
    pickerBusy.value ||
    (connection.value !== null && closingConnectionIds.value.has(connection.value.id)) ||
    activeSession.value?.workspaceBusy ||
    activeSession.value?.toolsBusy ||
    activeSession.value?.schemaRefreshing ||
    false,
),
 busyConnectionIds = computed(() => new Set(
  sessions.value
    .filter((session) => session.workspaceBusy || session.toolsBusy || session.schemaRefreshing)
    .map((session) => session.connection.id),
));

applyCatalogState(catalog.load());

onMounted(async () => {
  try {
    stopLaunchPathListener = await onDatabasePathsAvailable(schedulePendingDatabasePaths);
    schedulePendingDatabasePaths();
  } catch (error) {
    openErrorMessage.value = normalizeBackendError(error).message;
  }
});

onBeforeUnmount(() => stopLaunchPathListener?.());

function schedulePendingDatabasePaths(): void {
  launchPathWork = launchPathWork.then(openPendingDatabasePaths).catch((error: unknown) => {
    openErrorMessage.value = normalizeBackendError(error).message;
  });
}

async function openPendingDatabasePaths(): Promise<void> {
  const paths = await takePendingDatabasePaths();
  await Promise.all(paths.map((path) => openPath(path)));
}

async function openDatabase(): Promise<void> {
  if (opening.value || pickerBusy.value) {return;}
  const pickerGeneration = ++openGeneration;
  pickerBusy.value = true;
  let path: string | null;
  try {
    path = await chooseSqliteFile();
  } catch (error) {
    if (pickerGeneration === openGeneration)
      {openErrorMessage.value = normalizeBackendError(error).message;}
    pickerBusy.value = false;
    return;
  }
  if (pickerGeneration !== openGeneration) {
    pickerBusy.value = false;
    return;
  }
  // Release the picker guard before handing the selected path to openPath.
  pickerBusy.value = false;
  if (!path) {return;}
  await openPath(path);
}

async function openEnteredPath(): Promise<void> {
  const path = selectedPath.value.trim();
  if (!path) {
    openErrorMessage.value = "Enter the path to an existing SQLite database.";
    return;
  }

  selectedPath.value = path;
  await openPath(path);
}

function openPath(path: string): Promise<void> {
  const request = openPathWork.then(() => openPathNow(path));
  openPathWork = request.catch(() => undefined);
  return request;
}

async function openPathNow(path: string): Promise<void> {
  await waitForFilePicker();
  const existing = sessions.value.find((session) => session.connection.path === path);
  if (existing) {
    databaseSessions.activate(existing.connection.id);
    return;
  }
  const generation = ++openGeneration;
  selectedPath.value = path;
  opening.value = true;
  openErrorMessage.value = "";
  try {
    const opened = await openDatabasePath(path, openLocalDatabase, catalog);
    applyCatalogState(opened.catalogState);
    if (generation !== openGeneration) {
      await closeDatabase(opened.connection.id);
      return;
    }
    const result = databaseSessions.add(opened.connection);
    if (!result.added) {await closeDatabase(opened.connection.id);}
  } catch (error) {
    if (generation === openGeneration)
      {openErrorMessage.value = normalizeBackendError(error).message;}
  } finally {
    if (generation === openGeneration) {opening.value = false;}
  }
}

function waitForFilePicker(): Promise<void> {
  if (!pickerBusy.value) {return Promise.resolve();}

  return new Promise((resolve) => {
    const stop = watch(pickerBusy, (busy) => {
      if (busy) {return;}
      stop();
      resolve();
    });
  });
}

function saveActiveConnection(): void {
  if (!connection.value || controlsDisabled.value) {return;}
  applyCatalogState(catalog.save(connection.value));
}

function unsaveConnection(entry: ConnectionCatalogEntry): void {
  if (controlsDisabled.value) {return;}
  applyCatalogState(catalog.unsave(entry));
}

function forgetConnection(entry: ConnectionCatalogEntry): void {
  if (controlsDisabled.value) {return;}
  applyCatalogState(catalog.forget(entry));
}

function applyCatalogState(state: import("./types/connectionCatalog").ConnectionCatalogState): void {
  catalogEntries.value = state.entries;
  persistenceWarning.value = state.persistenceWarning;
}

function setWorkspaceBusy(connectionId: number, busy: boolean): void {
  const session = sessions.value.find((candidate) => candidate.connection.id === connectionId);
  if (session) {session.workspaceBusy = busy;}
}

function setToolsBusy(connectionId: number, busy: boolean): void {
  const session = sessions.value.find((candidate) => candidate.connection.id === connectionId);
  if (session) {session.toolsBusy = busy;}
}

async function closeConnection(connectionId = activeConnectionId.value): Promise<void> {
  if (connectionId === null || closingConnectionIds.value.has(connectionId)) {return;}
  const session = sessions.value.find((candidate) => candidate.connection.id === connectionId);
  if (!session || session.workspaceBusy || session.toolsBusy || session.schemaRefreshing) {return;}
  closingConnectionIds.value = new Set(closingConnectionIds.value).add(connectionId);
  session.errorMessage = "";
  try {
    await closeDatabase(connectionId);
    databaseSessions.remove(connectionId);
    if (!sessions.value.length) {selectedPath.value = "";}
  } catch (error) {
    session.errorMessage = normalizeBackendError(error).message;
  } finally {
    const next = new Set(closingConnectionIds.value);
    next.delete(connectionId);
    closingConnectionIds.value = next;
  }
}

function applySqlImport(connectionId: number, schema: import("./types/database").DatabaseSchema): void {
  const session = sessions.value.find((candidate) => candidate.connection.id === connectionId);
  if (!session) {return;}
  session.connection = applyImportedSchema(session.connection, schema);
  if (!selectedTableExists(schema, session.selectedTable)) {session.selectedTable = null;}
}

async function updateSchema(connectionId = activeConnectionId.value): Promise<void> {
  if (connectionId === null) {return;}
  const session = sessions.value.find((candidate) => candidate.connection.id === connectionId);
  if (!session) {return;}
  session.schemaRefreshing = true;
  try {
    const schema = await refreshSchema(connectionId);
    if (sessions.value.includes(session)) {session.connection = { ...session.connection, schema };}
  } catch (error) {
    if (sessions.value.includes(session)) {session.errorMessage = normalizeBackendError(error).message;}
  } finally {
    if (sessions.value.includes(session)) {session.schemaRefreshing = false;}
  }
}
</script>

<template>
  <main class="relative isolate flex h-screen min-h-0 flex-col overflow-hidden bg-[#090b0f] text-slate-200">
    <WelcomeBackdrop v-if="!hasConnection" />
    <header class="flex h-14 shrink-0 items-center justify-between border-b border-white/8 bg-surface px-5 text-xs">
      <div class="flex items-center gap-2.5 font-semibold tracking-tight text-slate-100">
        <span class="grid size-6 place-items-center rounded-md bg-accent text-2xs font-black text-accent-ink shadow-lg shadow-accent/15" aria-hidden="true">L</span>
        <span>LiteStation</span>
      </div>
      <span v-if="connection" class="chip rounded-full">
        <span class="size-1.5 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]" aria-hidden="true"></span>
        {{ sessions.length }} connected · Local SQLite
      </span>
      <span v-else class="chip hidden rounded-full sm:inline-flex">Local · SQLite</span>
    </header>
    <DatabaseTabs
      v-if="hasConnection"
      :sessions="sessions"
      :active-connection-id="activeConnectionId"
      :opening="opening || pickerBusy"
      :closing-connection-ids="closingConnectionIds"
      :busy-connection-ids="busyConnectionIds"
      @activate="databaseSessions.activate"
      @close="closeConnection"
      @open="openDatabase"
    />
    <WelcomeScreen
      v-if="!hasConnection"
      v-model:selected-path="selectedPath"
      :opening="opening"
      :picker-busy="pickerBusy"
      :error-message="openErrorMessage"
      :persistence-warning="persistenceWarning"
      :saved-connections="savedConnections"
      :recent-connections="recentConnections"
      @clear-error="openErrorMessage = ''"
      @choose="openDatabase"
      @submit="openEnteredPath"
      @open="openPath"
      @forget="forgetConnection"
    />
    <section v-else class="grid min-h-0 flex-1 grid-cols-[240px_minmax(0,1fr)] overflow-hidden lg:grid-cols-[260px_minmax(0,1fr)]">
      <aside class="flex min-h-0 flex-col overflow-hidden border-r border-white/8 bg-surface p-3">
        <div v-if="connection" class="mb-3 flex shrink-0 items-center gap-2.5 rounded-lg border border-white/8 bg-white/[.025] px-3 py-2.5">
          <span class="grid size-7 shrink-0 place-items-center rounded-md bg-accent/10 text-accent" aria-hidden="true"><Database :size="14" /></span>
          <div class="min-w-0 flex-1">
            <p class="truncate text-xs font-medium text-slate-200">{{ connection.displayName }}</p>
            <p class="text-2xs text-slate-600">{{ connection.schema.tables.length }} tables · {{ connection.schema.views.length }} views</p>
          </div>
        </div>
        <nav class="shrink-0" aria-label="Workspace views">
          <p class="eyebrow mb-2 px-2">Workspace</p>
          <div class="relative">
            <span v-if="selectedTable === null" class="absolute inset-y-1 left-0 w-0.5 rounded-full bg-accent" aria-hidden="true"></span>
            <button
              type="button"
              class="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-xs transition hover:bg-white/5 disabled:hover:bg-transparent"
              :class="{ active: selectedTable === null }"
              :aria-current="selectedTable === null ? 'page' : undefined"
              :disabled="controlsDisabled"
              @click="selectedTable = null"
            >
              <span :class="selectedTable === null ? 'text-accent' : 'text-slate-500'" aria-hidden="true"><SquareTerminal :size="14" /></span>
              <span :class="selectedTable === null ? 'text-slate-100' : 'text-slate-400'">SQL editor</span>
            </button>
          </div>
        </nav>
        <div class="min-h-0 flex-1 overflow-y-auto">
          <SchemaBrowser
            v-if="connection"
            :schema="connection.schema"
            :selected-table="selectedTable"
            :disabled="controlsDisabled"
            @select-table="selectedTable = $event"
          />
        </div>
      </aside>
      <section class="min-h-0 min-w-0 bg-[#0a0d12] p-4 sm:p-5" :class="selectedTable ? 'overflow-hidden' : 'overflow-auto'">
        <p v-if="persistenceWarning" class="alert alert-warn mb-3" role="status">
          Connection metadata could not be saved. Database work still works; check local storage permissions.
        </p>
        <p v-if="openErrorMessage || activeSession?.errorMessage" class="alert alert-error mb-3" role="alert">
          {{ openErrorMessage || activeSession?.errorMessage }}
        </p>
        <TableWorkspace
          v-if="connection && selectedTable"
          :key="`${connection.id}:${selectedTable}`"
          :connection-id="connection.id"
          :table-name="selectedTable"
          :indexes="connection.schema.indexes"
          @busy-change="setWorkspaceBusy(connection.id, $event)"
        />
        <SqlWorkspace
          v-else-if="connection"
          :key="connection.id"
          :connection-id="connection.id"
          v-model="sqlSource"
          @schema-changed="updateSchema(connection.id)"
          @busy-change="setWorkspaceBusy(connection.id, $event)"
        />
      </section>
    </section>
    <footer v-if="connection" class="flex h-10 shrink-0 items-center gap-3 border-t border-white/8 bg-surface px-3">
      <div class="flex min-w-0 items-center gap-2 text-xs">
        <span class="size-1.5 shrink-0 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]" aria-hidden="true"></span>
        <strong class="shrink-0 font-medium text-slate-300">{{ connection.displayName }}</strong>
        <span class="hidden max-w-80 truncate text-slate-600 md:block" :title="connection.path">{{ connection.path }}</span>
      </div>
      <div class="ml-auto flex items-center gap-1">
        <DatabaseTools
          :connection-id="connection.id"
          @busy-change="setToolsBusy(connection.id, $event)"
          @schema-imported="applySqlImport(connection.id, $event)"
        />
        <span class="mx-1 h-4 w-px bg-white/10" aria-hidden="true"></span>
        <button
          type="button"
          class="icon-btn"
          :disabled="controlsDisabled"
          :aria-label="activeSession?.schemaRefreshing ? 'Refreshing schema…' : 'Refresh schema'"
          :title="activeSession?.schemaRefreshing ? 'Refreshing schema…' : 'Refresh schema'"
          @click="updateSchema(connection.id)"
        ><RefreshCw :size="14" :class="{ 'animate-spin': activeSession?.schemaRefreshing }" /></button>
        <button
          v-if="activeCatalogEntry?.saved"
          type="button"
          class="icon-btn"
          :disabled="controlsDisabled"
          aria-label="Remove from saved connections"
          title="Remove from saved connections"
          @click="unsaveConnection(activeCatalogEntry)"
        ><BookmarkX :size="14" /></button>
        <button
          v-else
          type="button"
          class="icon-btn"
          :disabled="controlsDisabled"
          aria-label="Save connection"
          title="Save connection"
          @click="saveActiveConnection"
        ><Bookmark :size="14" /></button>
        <button
          v-if="activeCatalogEntry"
          type="button"
          class="icon-btn text-slate-600 hover:text-rose-300"
          :disabled="controlsDisabled"
          aria-label="Forget connection metadata"
          title="Forget connection metadata"
          @click="forgetConnection(activeCatalogEntry)"
        ><X :size="14" /></button>
        <button
          type="button"
          class="icon-btn hover:bg-rose-400/10 hover:text-rose-300"
          :disabled="controlsDisabled"
          aria-label="Disconnect"
          title="Disconnect"
          @click="closeConnection()"
        ><Unplug :size="14" /></button>
      </div>
    </footer>
  </main>
</template>
