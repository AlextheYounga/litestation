<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from "vue";
import { Copy, FileDown, FileUp } from "@lucide/vue";
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
  <section class="flex items-center gap-1" aria-label="Database import and export tools">
    <div class="flex items-center gap-1">
      <button type="button" class="flex h-7 items-center gap-1.5 rounded px-2 text-[10px] text-slate-500 transition hover:bg-white/5 hover:text-slate-100 disabled:hover:bg-transparent" :disabled="busy" title="Import SQL dump" @click="requestSqlImport"><FileDown :size="13" />Import SQL dump</button>
      <button type="button" class="flex h-7 items-center gap-1.5 rounded px-2 text-[10px] text-slate-500 transition hover:bg-white/5 hover:text-slate-100 disabled:hover:bg-transparent" :disabled="busy" title="Create SQLite copy" @click="createSqliteCopy"><Copy :size="13" />Create copy</button>
      <button type="button" class="flex h-7 items-center gap-1.5 rounded px-2 text-[10px] text-slate-500 transition hover:bg-white/5 hover:text-slate-100 disabled:hover:bg-transparent" :disabled="busy" title="Export SQL dump" @click="exportDump"><FileUp :size="13" />Export dump</button>
    </div>
    <p v-if="error" class="overlay-in fixed bottom-12 right-3 z-40 max-w-sm rounded-lg border border-rose-400/20 bg-[#1b1216] px-3 py-2 text-[11px] text-rose-300 shadow-xl" role="alert">{{ error }}</p>
    <p v-if="status" class="overlay-in fixed bottom-12 right-3 z-40 max-w-sm rounded-lg border border-emerald-400/20 bg-[#101a17] px-3 py-2 text-[11px] text-emerald-300 shadow-xl" role="status">{{ status }}</p>
    <ConfirmDialog :open="confirmSqlImport" title="Import SQL dump?" message="This can alter schema and data. The import is atomic: it either completes or rolls back." confirm-label="Import SQL dump" @cancel="confirmSqlImport = false" @confirm="importSql" />
  </section>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
