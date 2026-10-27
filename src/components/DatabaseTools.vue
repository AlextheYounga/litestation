<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
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
import type { ConnectionMode, DatabaseSchema } from "../types/database";

const props = defineProps<{ connectionId: number; mode: ConnectionMode }>();
const emit = defineEmits<{ "busy-change": [busy: boolean]; "schema-imported": [schema: DatabaseSchema] }>();

const busy = ref(false);
const confirmSqlImport = ref(false);
const sqlSource = ref<string | null>(null);
const error = ref("");
const status = ref("");
let generation = 0;
let active = true;
const canImportSql = computed(() => props.mode === "readWrite");

watch(
  () => [props.connectionId, props.mode],
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
  if (!canImportSql.value || busy.value) return;
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
  <section class="database-tools" aria-labelledby="database-tools-heading">
    <p class="eyebrow">Database tools</p><h2 id="database-tools-heading">Import &amp; export</h2>
    <button type="button" :disabled="busy || !canImportSql" title="SQL import requires a read & write connection." @click="requestSqlImport">Import SQL dump</button>
    <button type="button" :disabled="busy" @click="createSqliteCopy">Create SQLite copy</button>
    <button type="button" :disabled="busy" @click="exportDump">Export SQL dump</button>
    <p class="tool-hint">A SQLite copy is validated and consistent; it does not merge into this database or alter its source. Existing destinations are rejected.</p>
    <p v-if="error" class="error-message" role="alert">{{ error }}</p><p v-if="status" class="status-message" role="status">{{ status }}</p>
    <ConfirmDialog :open="confirmSqlImport" title="Import SQL dump?" message="This can alter schema and data. The import is atomic: it either completes or rolls back." confirm-label="Import SQL dump" @cancel="confirmSqlImport = false" @confirm="importSql" />
  </section>
</template>

<style scoped>
.database-tools { display: grid; gap: 6px; margin-top: 22px; }
.database-tools h2 { margin: 0 0 3px; font-size: 15px; }
.database-tools button { padding: 6px; border: 1px solid var(--line); border-radius: 4px; background: transparent; color: inherit; font-size: 11px; text-align: left; }
.tool-hint { margin: 4px 0; color: var(--muted); font-size: 10px; line-height: 1.4; }
</style>
