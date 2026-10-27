<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from "vue";
import ConfirmDialog from "./ConfirmDialog.vue";
import {
  chooseSqlDump,
  chooseSqlDumpDestination,
  chooseSqliteFile,
  chooseSqliteCopyDestination,
  exportSqlDump,
  importSqlDump,
  importSqliteFile,
  normalizeBackendError,
} from "../services/database";
import type { DatabaseSchema } from "../types/database";

const props = defineProps<{ connectionId: number }>();
const emit = defineEmits<{ "busy-change": [busy: boolean]; "schema-imported": [schema: DatabaseSchema] }>();

const busy = ref(false);
const confirmSqlImport = ref(false);
const sqlSource = ref<string | null>(null);
const error = ref("");
const status = ref("");
let generation = 0;
let active = true;
watch(
  () => props.connectionId,
  () => {
    generation += 1;
    sqlSource.value = null;
    confirmSqlImport.value = false;
    error.value = "";
    status.value = "";
    if (busy.value) {
      busy.value = false;
      emit("busy-change", false);
    }
  },
);

onBeforeUnmount(() => { active = false; generation += 1; emit("busy-change", false); });

function current(token: number, connectionId: number): boolean {
  return active && token === generation && connectionId === props.connectionId;
}

function operationError(cause: unknown): string {
  const backendError = normalizeBackendError(cause);
  return backendError.code === "destination_exists"
    ? "A file already exists at that destination. Choose a new path; existing files are never overwritten."
    : backendError.message;
}

async function requestSqlImport(): Promise<void> {
  if (busy.value) return;
  const token = ++generation;
  const connectionId = props.connectionId;
  error.value = "";
  status.value = "";
  let sourcePath: string | null;
  try {
    sourcePath = await chooseSqlDump();
  } catch (cause) {
    if (current(token, connectionId)) {
      status.value = "";
      error.value = `Could not choose SQL dump: ${normalizeBackendError(cause).message}`;
    }
    return;
  }
  if (current(token, connectionId) && sourcePath) {
    sqlSource.value = sourcePath;
    confirmSqlImport.value = true;
  }
}

async function importSql(): Promise<void> {
  const sourcePath = sqlSource.value;
  confirmSqlImport.value = false;
  if (!sourcePath || busy.value) return;
  const token = ++generation; const connectionId = props.connectionId;
  busy.value = true; emit("busy-change", true); error.value = ""; status.value = "";
  try {
    const response = await importSqlDump({ connectionId, sourcePath });
    if (!current(token, connectionId)) return;
    emit("schema-imported", response.schema);
    status.value = "SQL dump imported atomically.";
  } catch (cause) { if (current(token, connectionId)) { status.value = ""; error.value = normalizeBackendError(cause).message; } }
  finally { if (current(token, connectionId)) { busy.value = false; emit("busy-change", false); } }
}

async function createSqliteCopy(): Promise<void> {
  if (busy.value) return;
  const token = ++generation;
  const connectionId = props.connectionId;
  error.value = "";
  status.value = "";
  let sourcePath: string | null; let destinationPath: string | null;
  try { sourcePath = await chooseSqliteFile(); } catch (cause) { if (current(token, connectionId)) { status.value = ""; error.value = `Could not choose SQLite source: ${operationError(cause)}`; } return; }
  if (!sourcePath) return;
  if (!current(token, connectionId)) return;
  try { destinationPath = await chooseSqliteCopyDestination(); } catch (cause) { if (current(token, connectionId)) { status.value = ""; error.value = `Could not choose copy destination: ${operationError(cause)}`; } return; }
  if (!destinationPath) return;
  if (!current(token, connectionId)) return;
  busy.value = true; emit("busy-change", true);
  try { const response = await importSqliteFile({ sourcePath, destinationPath }); if (current(token, connectionId)) status.value = `Created SQLite copy: ${response.path}`; }
  catch (cause) { if (current(token, connectionId)) { status.value = ""; error.value = operationError(cause); } }
  finally { if (current(token, connectionId)) { busy.value = false; emit("busy-change", false); } }
}

async function exportDump(): Promise<void> {
  if (busy.value) return;
  const token = ++generation;
  const connectionId = props.connectionId;
  error.value = "";
  status.value = "";
  let destinationPath: string | null;
  try { destinationPath = await chooseSqlDumpDestination(); } catch (cause) { if (current(token, connectionId)) { status.value = ""; error.value = `Could not choose SQL dump destination: ${normalizeBackendError(cause).message}`; } return; }
  if (!destinationPath) return;
  if (!current(token, connectionId)) return;
  busy.value = true; emit("busy-change", true);
  try { const response = await exportSqlDump({ connectionId, destinationPath }); if (current(token, connectionId)) status.value = `Exported ${response.rowCount} rows to ${response.path}`; }
  catch (cause) { if (current(token, connectionId)) { status.value = ""; error.value = operationError(cause); } }
  finally { if (current(token, connectionId)) { busy.value = false; emit("busy-change", false); } }
}
</script>

<template>
  <section class="mt-8 border-t border-white/8 pt-5" aria-labelledby="database-tools-heading">
    <p class="px-2 text-[10px] font-semibold uppercase tracking-widest text-slate-600">Database tools</p>
    <h2 id="database-tools-heading" class="mb-2 mt-1 px-2 text-xs font-semibold text-slate-300">Import &amp; export</h2>
    <div class="grid gap-1">
      <button type="button" class="rounded-lg px-2 py-2 text-left text-xs text-slate-400 transition hover:bg-white/5 hover:text-slate-100 disabled:hover:bg-transparent" :disabled="busy" @click="requestSqlImport">Import SQL dump</button>
      <button type="button" class="rounded-lg px-2 py-2 text-left text-xs text-slate-400 transition hover:bg-white/5 hover:text-slate-100 disabled:hover:bg-transparent" :disabled="busy" @click="createSqliteCopy">Create SQLite copy</button>
      <button type="button" class="rounded-lg px-2 py-2 text-left text-xs text-slate-400 transition hover:bg-white/5 hover:text-slate-100 disabled:hover:bg-transparent" :disabled="busy" @click="exportDump">Export SQL dump</button>
    </div>
    <p class="mt-2 px-2 text-[10px] leading-4 text-slate-600">Copies are validated and never overwrite existing files.</p>
    <p v-if="error" class="mt-2 rounded-lg border border-rose-400/20 bg-rose-400/5 px-2 py-2 text-[11px] text-rose-300" role="alert">{{ error }}</p><p v-if="status" class="mt-2 px-2 text-[11px] text-emerald-300" role="status">{{ status }}</p>
    <ConfirmDialog :open="confirmSqlImport" title="Import SQL dump?" message="This can alter schema and data. The import is atomic: it either completes or rolls back." confirm-label="Import SQL dump" @cancel="confirmSqlImport = false" @confirm="importSql" />
  </section>
</template>

<style scoped>
.database-tools { display: grid; gap: 6px; margin-top: 22px; }
.database-tools h2 { margin: 0 0 3px; font-size: 15px; }
.database-tools button { padding: 6px; border: 1px solid var(--line); border-radius: 4px; background: transparent; color: inherit; font-size: 11px; text-align: left; }
.tool-hint { margin: 4px 0; color: var(--muted); font-size: 10px; line-height: 1.4; }
</style>
