<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { Play } from "@lucide/vue";
import ConfirmDialog from "./ConfirmDialog.vue";
import ResultGrid from "./ResultGrid.vue";
import SqlEditor from "./SqlEditor.vue";
import {
  chooseExportDestination,
  executeSql,
  exportRows,
  normalizeBackendError,
} from "../services/database";
import type { ExportFormat } from "../types/database";
import {
  isPotentiallyDestructive,
  statementAtCursor,
  statements,
} from "../utils/sqlStatement";
import type { ExecuteSqlResponse } from "../types/database";

const props = defineProps<{
  connectionId: number;
  modelValue: string;
}>();
const emit = defineEmits<{
  "schema-changed": [];
  "busy-change": [busy: boolean];
  "update:modelValue": [value: string];
}>();

const result = ref<ExecuteSqlResponse | null>(null);
const error = ref("");
const status = ref("");
const running = ref(false);
const exporting = ref(false);
const confirmRunAll = ref(false);
const pendingSql = ref<string | null>(null);
let generation = 0;
let active = true;

const runAllStatements = computed(() => statements(props.modelValue));
const busy = computed(() => running.value || exporting.value);

watch(busy, (value) => emit("busy-change", value), { immediate: true });
watch(
  () => props.connectionId,
  () => {
    generation += 1;
    result.value = null;
    error.value = "";
    status.value = "";
    confirmRunAll.value = false;
    pendingSql.value = null;
    running.value = false;
    exporting.value = false;
  },
);
onBeforeUnmount(() => {
  active = false;
  generation += 1;
  pendingSql.value = null;
  emit("busy-change", false);
});

async function run(selection: string, cursor: number, source = props.modelValue): Promise<void> {
  const text = selection || statementAtCursor(source, cursor);
  if (!text?.trim()) {
    status.value = "";
    error.value = "Place the cursor in a SQL statement or select SQL to run.";
    return;
  }
  await requestExecution(text);
}

async function requestRunAll(): Promise<void> {
  if (!runAllStatements.value.length) {
    status.value = "";
    error.value = "There are no SQL statements to run.";
  } else if (runAllStatements.value.length === 1) {
    await requestExecution(runAllStatements.value[0].text);
  } else {
    confirmRunAll.value = true;
  }
}

async function requestExecution(sql: string): Promise<void> {
  if (busy.value) return;
  error.value = "";
  status.value = "";
  if (isPotentiallyDestructive(sql)) {
    pendingSql.value = sql;
    return;
  }
  await executeOne(sql);
}

async function confirmPendingSql(): Promise<void> {
  const sql = pendingSql.value;
  pendingSql.value = null;
  if (sql !== null) await executeOne(sql);
}

function cancelPendingSql(): void {
  pendingSql.value = null;
}

async function executeAll(): Promise<void> {
  confirmRunAll.value = false;
  if (busy.value) return;

  const token = ++generation;
  const connectionId = props.connectionId;
  const commands = statements(props.modelValue);
  let completed = 0;
  let changedSchema = false;
  running.value = true;
  result.value = null;
  error.value = "";

  try {
    for (const [index, command] of commands.entries()) {
      status.value = `Running statement ${index + 1} of ${commands.length}…`;
      try {
        const response = await executeSql({
          connectionId,
          sql: command.text,
          maxRows: 100,
        });
        if (!isCurrent(token, connectionId)) return;
        result.value = response;
        changedSchema ||= response.schemaChanged;
        completed += 1;
      } catch (cause) {
        if (!isCurrent(token, connectionId)) return;
        error.value = `Statement ${index + 1} failed after ${completed} completed. Earlier successful statements remain applied because Run all is non-atomic. ${normalizeBackendError(cause).message}`;
        status.value = "";
        return;
      }
    }
    status.value = `Completed ${completed} statements sequentially (non-atomic).`;
  } finally {
    if (isCurrent(token, connectionId)) {
      if (changedSchema) emit("schema-changed");
      running.value = false;
    }
  }
}

async function executeOne(sql: string): Promise<void> {
  if (busy.value) return;
  const token = ++generation;
  const connectionId = props.connectionId;
  running.value = true;
  result.value = null;
  error.value = "";
  status.value = "Running query…";

  try {
    const response = await executeSql({ connectionId, sql, maxRows: 100 });
    if (!isCurrent(token, connectionId)) return;
    result.value = response;
    status.value = "Statement completed.";
    if (response.schemaChanged) emit("schema-changed");
  } catch (cause) {
    if (isCurrent(token, connectionId)) {
      error.value = normalizeBackendError(cause).message;
      status.value = "";
    }
  } finally {
    if (isCurrent(token, connectionId)) running.value = false;
  }
}

async function exportResult(format: ExportFormat): Promise<void> {
  const displayed = result.value;
  if (!displayed?.columns.length || busy.value) return;
  error.value = "";
  let destinationPath: string | null;
  try {
    destinationPath = await chooseExportDestination(format);
  } catch (cause) {
    error.value = `Could not choose ${format.toUpperCase()} destination: ${normalizeBackendError(cause).message}`;
    return;
  }
  if (!destinationPath) return;

  const token = ++generation;
  const connectionId = props.connectionId;
  exporting.value = true;
  try {
    const response = await exportRows({
      columns: displayed.columns,
      rows: displayed.rows,
      format,
      destinationPath,
    });
    if (isCurrent(token, connectionId))
      status.value = `Exported ${response.rowCount} displayed rows to ${response.path}`;
  } catch (cause) {
    if (isCurrent(token, connectionId))
      error.value = exportError(cause);
  } finally {
    if (isCurrent(token, connectionId)) exporting.value = false;
  }
}

function isCurrent(token: number, connectionId: number): boolean {
  return active && token === generation && connectionId === props.connectionId;
}

function exportError(cause: unknown): string {
  const backendError = normalizeBackendError(cause);
  return backendError.code === "destination_exists"
    ? "A file already exists at that destination. Choose a new path; existing files are never overwritten."
    : backendError.message;
}
</script>

<template>
  <section class="grid gap-4">
    <header class="flex items-end justify-between gap-4">
      <div>
        <p class="text-[10px] font-semibold uppercase tracking-widest text-[#cfed66]">SQL query</p>
        <h2 class="mt-1 text-xl font-semibold tracking-tight text-white">Query workspace</h2>
      </div>
      <button
        type="button"
        class="flex items-center gap-1.5 rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-xs font-medium text-slate-300 transition hover:border-[#cfed66]/30 hover:bg-[#cfed66]/10 hover:text-[#cfed66]"
        :disabled="busy"
        @click="requestRunAll"
      >
        <Play :size="12" /> Run all
      </button>
    </header>
    <SqlEditor
      :model-value="modelValue"
      @update:model-value="emit('update:modelValue', $event)"
      @run="run"
    />
    <p v-if="error" class="rounded-lg border border-rose-400/20 bg-rose-400/5 px-3 py-2 text-xs text-rose-300" role="alert">{{ error }}</p>
    <p v-if="running || status" role="status" class="text-xs text-slate-500">
      {{ status }}
    </p>
    <div v-if="result" class="flex flex-wrap items-center gap-2 text-[11px] text-slate-500">
      <span class="rounded-md border border-white/8 bg-white/[.03] px-2 py-1">{{ result.durationMs }} ms</span>
      <span v-if="result.affectedRows !== null" class="rounded-md border border-white/8 bg-white/[.03] px-2 py-1"
        >{{ result.affectedRows }} affected</span
      >
      <span v-if="result.truncated" class="rounded-md border border-amber-400/20 bg-amber-400/10 px-2 py-1 text-amber-300"
        >Showing first 100 rows; result was truncated.</span
      >
    </div>
    <div v-if="result?.columns.length" class="flex flex-wrap items-center gap-2" aria-label="Export displayed query result">
      <span class="text-[11px] text-slate-600">Export displayed rows</span>
      <button type="button" class="rounded-md border border-white/10 px-2.5 py-1.5 text-[11px] text-slate-400 transition hover:bg-white/5 hover:text-white" :disabled="busy" @click="exportResult('json')">Export JSON</button>
      <button type="button" class="rounded-md border border-white/10 px-2.5 py-1.5 text-[11px] text-slate-400 transition hover:bg-white/5 hover:text-white" :disabled="busy" @click="exportResult('csv')">Export CSV</button>
    </div>
    <ResultGrid
      v-if="result"
      :columns="result.columns"
      :rows="result.rows"
      :empty-label="
        result.affectedRows !== null
          ? 'Statement completed without result rows.'
          : 'No rows returned.'
      "
    />
    <ConfirmDialog
      :open="confirmRunAll"
      title="Run all statements?"
      message="Statements run sequentially, are non-atomic, stop at the first error, and earlier successful writes remain applied."
      confirm-label="Run all statements"
      :destructive="false"
      @cancel="confirmRunAll = false"
      @confirm="executeAll"
    />
    <ConfirmDialog
      :open="pendingSql !== null"
      title="Run potentially destructive statement?"
      message="This statement may change schema or data."
      confirm-label="Run statement"
      :destructive="true"
      @cancel="cancelPendingSql"
      @confirm="confirmPendingSql"
    />
  </section>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
