<script setup lang="ts">
import { computed, ref } from "vue";
import {
  Bookmark,
  BookmarkX,
  Clock,
  Database,
  FolderOpen,
  SquareTerminal,
  Star,
  Unplug,
  X,
} from "@lucide/vue";
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

async function openEnteredPath(): Promise<void> {
  const path = selectedPath.value.trim();
  if (!path) {
    errorMessage.value = "Enter the path to an existing SQLite database.";
    return;
  }

  selectedPath.value = path;
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
  <main class="flex h-screen min-h-0 flex-col overflow-hidden bg-[#090b0f] text-slate-200">
    <header class="flex h-14 shrink-0 items-center justify-between border-b border-white/8 bg-[#0d1016] px-5 text-xs">
      <div class="flex items-center gap-2.5 font-semibold tracking-tight text-slate-100">
        <span class="grid size-6 place-items-center rounded-md bg-[#cfed66] text-[11px] font-black text-[#182010] shadow-lg shadow-[#cfed66]/15" aria-hidden="true">L</span>
        <span>LiteStation</span>
      </div>
      <div v-if="connection" class="hidden items-center gap-2 text-slate-400 sm:flex">
        <span class="grid size-5 place-items-center rounded bg-[#cfed66]/10 text-[#cfed66]" aria-hidden="true"><Database :size="11" /></span>
        <span>{{ connection.displayName }}</span>
        <span class="max-w-80 truncate text-slate-600">{{ connection.path }}</span>
      </div>
      <span class="hidden rounded-full border border-white/10 bg-white/[.03] px-2.5 py-1 text-[11px] text-slate-500 sm:block">Local · SQLite</span>
    </header>
    <section v-if="!hasConnection" class="relative mx-auto grid min-h-0 max-w-5xl flex-1 items-center gap-12 overflow-auto px-6 py-16 lg:grid-cols-[1fr_400px]">
      <div class="pointer-events-none absolute left-0 top-1/3 hidden h-96 w-96 -translate-x-24 rounded-full bg-[#cfed66]/6 blur-3xl lg:block" aria-hidden="true"></div>
      <div class="relative max-w-xl">
        <span class="mb-5 inline-flex items-center gap-2 rounded-full border border-[#cfed66]/20 bg-[#cfed66]/8 px-3 py-1.5 text-[11px] font-medium text-[#cfed66]"><span class="size-1.5 rounded-full bg-[#cfed66]"></span>Local-first SQLite</span>
        <h1 class="text-4xl font-semibold tracking-[-0.04em] text-white sm:text-5xl">A focused workspace<br /><span class="text-[#cfed66]">for your databases.</span></h1>
        <p class="mt-5 max-w-md text-sm leading-7 text-slate-400">Browse schemas, edit data, run queries, and export results without your database ever leaving this device.</p>
      </div>
      <form class="rounded-2xl border border-white/10 bg-[#11151d] p-6 shadow-2xl shadow-black/20" @submit.prevent="openEnteredPath">
        <div class="mb-5 grid size-11 place-items-center rounded-xl border border-[#cfed66]/15 bg-[#cfed66]/8 text-[#cfed66]" aria-hidden="true"><Database :size="20" /></div>
        <div>
          <h2 class="text-base font-semibold text-white">Open a SQLite database</h2>
          <p class="mt-1 text-xs leading-5 text-slate-500">Enter the path to, or choose, an existing <code>.db</code>, <code>.sqlite</code>, or <code>.sqlite3</code> file.</p>
        </div>
        <label class="mt-6 block text-[11px] font-medium text-slate-400" for="database-path">Database path</label>
        <div class="relative mt-2">
          <span class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-600" aria-hidden="true"><FolderOpen :size="13" /></span>
          <input
            id="database-path"
            v-model="selectedPath"
            class="mt-0 block w-full rounded-lg border border-white/10 bg-black/15 py-2.5 pl-9 pr-3 text-xs text-slate-200 transition placeholder:text-slate-600 focus:border-[#cfed66]/40 focus:outline-none focus:ring-2 focus:ring-[#cfed66]/15"
            type="text"
            placeholder="/path/to/database.sqlite"
            :disabled="opening || pickerBusy"
            @input="errorMessage = ''"
          />
        </div>
        <div class="mt-3 grid grid-cols-2 gap-2">
          <button class="rounded-lg bg-[#cfed66] px-4 py-2.5 text-xs font-semibold text-[#182010] shadow-lg shadow-[#cfed66]/10 transition hover:bg-[#d9f57c] disabled:hover:bg-[#cfed66]" :disabled="opening || pickerBusy">
            {{ opening ? "Opening…" : "Open database" }}
          </button>
          <button type="button" class="flex items-center justify-center gap-1.5 rounded-lg border border-white/10 bg-white/5 px-4 py-2.5 text-xs font-medium text-slate-300 transition hover:bg-white/10 hover:text-white" :disabled="opening || pickerBusy" @click="openDatabase">
            <FolderOpen :size="13" /> {{ pickerBusy ? "Choosing…" : "Choose file" }}
          </button>
        </div>
        <p v-if="errorMessage" class="mt-3 rounded-lg border border-rose-400/20 bg-rose-400/5 px-3 py-2 text-xs text-rose-300" role="alert">
          {{ errorMessage }}
        </p>
        <p v-if="persistenceWarning" class="mt-3 rounded-lg border border-amber-400/20 bg-amber-400/5 px-3 py-2 text-xs text-amber-300" role="status">
          Connection metadata could not be saved. Database work still works; check local storage permissions.
        </p>
      </form>
        <section
        v-if="savedConnections.length || recentConnections.length"
        class="col-span-full grid max-h-64 gap-5 overflow-auto rounded-xl border border-white/8 bg-[#0f131a] p-5 lg:col-start-2"
        aria-label="Saved and recent databases"
      >
          <div v-if="savedConnections.length" class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-widest text-slate-500"><Star :size="11" class="text-[#cfed66]/70" /> Saved</h2>
          <ul class="catalog-list">
            <li v-for="entry in savedConnections" :key="entry.id">
              <span class="ml-2 grid size-4 shrink-0 place-items-center text-slate-600" aria-hidden="true"><Database :size="11" /></span>
              <button type="button" class="grid min-w-0 flex-1 rounded-lg px-3 py-2 text-left transition hover:bg-white/5" :disabled="controlsDisabled" @click="openPath(entry.path)">
                <strong class="truncate text-xs font-medium text-slate-200">{{ entry.displayName }}</strong><span class="truncate text-[11px] text-slate-500">{{ entry.path }}</span>
              </button>
              <button type="button" class="px-2 text-[10px] text-slate-600 hover:text-rose-300" :disabled="controlsDisabled" :aria-label="`Forget saved connection ${entry.displayName}; database files are not deleted`" @click="forgetConnection(entry)">Forget</button>
            </li>
          </ul>
        </div>
          <div v-if="recentConnections.length" class="space-y-2">
            <h2 class="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-widest text-slate-500"><Clock :size="11" /> Recent</h2>
          <ul class="catalog-list">
            <li v-for="entry in recentConnections" :key="entry.id">
              <span class="ml-2 grid size-4 shrink-0 place-items-center text-slate-600" aria-hidden="true"><Database :size="11" /></span>
              <button type="button" class="grid min-w-0 flex-1 rounded-lg px-3 py-2 text-left transition hover:bg-white/5" :disabled="controlsDisabled" @click="openPath(entry.path)">
                <strong class="truncate text-xs font-medium text-slate-200">{{ entry.displayName }}</strong><span class="truncate text-[11px] text-slate-500">{{ entry.path }} · {{ formatOpenedAt(entry.lastOpenedAt) }}</span>
              </button>
              <button type="button" class="px-2 text-[10px] text-slate-600 hover:text-rose-300" :disabled="controlsDisabled" :aria-label="`Forget recent connection ${entry.displayName}; database files are not deleted`" @click="forgetConnection(entry)">Forget</button>
            </li>
          </ul>
        </div>
        <p class="text-[10px] text-slate-600">Forget removes LiteStation metadata only. It never deletes database files.</p>
      </section>
    </section>
    <section v-else class="grid min-h-0 flex-1 grid-cols-[240px_minmax(0,1fr)] overflow-hidden lg:grid-cols-[260px_minmax(0,1fr)]">
      <aside class="flex min-h-0 flex-col overflow-hidden border-r border-white/8 bg-[#0d1016] p-3">
        <nav class="shrink-0" aria-label="Workspace views">
          <p class="mb-2 px-2 text-[10px] font-semibold uppercase tracking-widest text-slate-600">Workspace</p>
          <button
            type="button"
            class="flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-left text-xs transition hover:bg-white/5 disabled:hover:bg-transparent"
            :class="{ active: selectedTable === null }"
            :aria-current="selectedTable === null ? 'page' : undefined"
            :disabled="controlsDisabled"
            @click="selectedTable = null"
          >
            <span class="text-[#cfed66]" aria-hidden="true"><SquareTerminal :size="14" /></span>
            <span :class="selectedTable === null ? 'text-slate-100' : 'text-slate-400'">SQL editor</span>
          </button>
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
        <p v-if="persistenceWarning" class="mb-3 rounded-md border border-amber-400/20 bg-amber-400/5 px-3 py-2 text-xs text-amber-300" role="status">
          Connection metadata could not be saved. Database work still works; check local storage permissions.
        </p>
        <p v-if="errorMessage" class="mb-3 rounded-md border border-rose-400/20 bg-rose-400/5 px-3 py-2 text-xs text-rose-300" role="alert">
          {{ errorMessage }}
        </p>
        <TableWorkspace
          v-if="connection && selectedTable"
          :key="`${connection.id}:${selectedTable}`"
          :connection-id="connection.id"
          :table-name="selectedTable"
          :indexes="connection.schema.indexes"
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
    <footer v-if="connection" class="flex h-10 shrink-0 items-center gap-3 border-t border-white/8 bg-[#0d1016] px-3">
      <div class="flex min-w-0 items-center gap-2 text-[11px]">
        <span class="size-1.5 shrink-0 rounded-full bg-emerald-400 shadow-[0_0_8px_#34d399]" aria-hidden="true"></span>
        <strong class="shrink-0 font-medium text-slate-300">{{ connection.displayName }}</strong>
        <span class="hidden max-w-80 truncate text-slate-600 md:block" :title="connection.path">{{ connection.path }}</span>
      </div>
      <div class="ml-auto flex items-center gap-1">
        <DatabaseTools
          :connection-id="connection.id"
          @busy-change="toolsBusy = $event"
          @schema-imported="applySqlImport"
        />
        <span class="mx-1 h-4 w-px bg-white/10" aria-hidden="true"></span>
        <button
          v-if="activeCatalogEntry?.saved"
          type="button"
          class="grid size-7 place-items-center rounded text-slate-500 hover:bg-white/5 hover:text-white"
          :disabled="controlsDisabled"
          aria-label="Remove from saved connections"
          title="Remove from saved connections"
          @click="unsaveConnection(activeCatalogEntry)"
        ><BookmarkX :size="14" /></button>
        <button
          v-else
          type="button"
          class="grid size-7 place-items-center rounded text-slate-500 hover:bg-white/5 hover:text-white"
          :disabled="controlsDisabled"
          aria-label="Save connection"
          title="Save connection"
          @click="saveActiveConnection"
        ><Bookmark :size="14" /></button>
        <button
          v-if="activeCatalogEntry"
          type="button"
          class="grid size-7 place-items-center rounded text-slate-600 hover:bg-white/5 hover:text-rose-300"
          :disabled="controlsDisabled"
          aria-label="Forget connection metadata"
          title="Forget connection metadata"
          @click="forgetConnection(activeCatalogEntry)"
        ><X :size="14" /></button>
        <button
          type="button"
          class="grid size-7 place-items-center rounded text-slate-500 hover:bg-rose-400/10 hover:text-rose-300"
          :disabled="controlsDisabled"
          aria-label="Disconnect"
          title="Disconnect"
          @click="closeConnection"
        ><Unplug :size="14" /></button>
      </div>
    </footer>
  </main>
</template>
