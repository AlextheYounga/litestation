<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import ConfirmDialog from "./ConfirmDialog.vue";
import ResultGrid from "./ResultGrid.vue";
import RowEditor from "./RowEditor.vue";
import {
  browseTable,
  chooseExportDestination,
  deleteRow,
  exportRows,
  insertRow,
  normalizeBackendError,
  updateRow,
} from "../services/database";
import type {
  BrowseRow,
  BrowseTableResponse,
  ColumnValue,
  MutationResult,
  ExportFormat,
} from "../types/database";
import { displayValue, sameValue } from "../utils/sqliteValue";
import { TableRequestGuard } from "../utils/tableRequest";

const props = defineProps<{
  connectionId: number;
  tableName: string;
  readOnly: boolean;
}>();
const emit = defineEmits<{ "busy-change": [busy: boolean] }>();

const PAGE_SIZE = 100;
const page = ref<BrowseTableResponse | null>(null);
const loading = ref(false);
const mutating = ref(false);
const exporting = ref(false);
const error = ref("");
const status = ref("");
const editing = ref<BrowseRow | null>(null);
const inserting = ref(false);
const deleting = ref<BrowseRow | null>(null);
const requests = new TableRequestGuard();
let active = true;

const busy = computed(() => loading.value || mutating.value || exporting.value);
const canMutate = computed(
  () => !props.readOnly && Boolean(page.value?.metadata.editable),
);
const mutationReason = computed(() =>
  props.readOnly
    ? "This connection is read-only."
    : page.value?.metadata.nonEditableReason,
);
const tableKey = computed(() => `${props.connectionId}:${props.tableName}`);
const editorKey = computed(() =>
  editing.value
    ? JSON.stringify({
        identity: editing.value.identity,
        values: editing.value.values,
      })
    : "insert",
);

watch(busy, (value) => emit("busy-change", value), { immediate: true });
watch(
  () => [props.connectionId, props.tableName],
  () => {
    editing.value = null;
    inserting.value = false;
    deleting.value = null;
    void load(0);
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  active = false;
  requests.invalidate();
  emit("busy-change", false);
});

async function load(offset: number): Promise<void> {
  const requestKey = {
    connectionId: props.connectionId,
    tableName: props.tableName,
    offset,
  };
  const generation = requests.begin(requestKey);
  loading.value = true;
  error.value = "";

  try {
    const response = await browseTable({ ...requestKey, limit: PAGE_SIZE });
    if (active && requests.isCurrent(generation)) page.value = response;
  } catch (cause) {
    if (active && requests.isCurrent(generation))
      error.value = normalizeBackendError(cause).message;
  } finally {
    if (active && requests.isCurrent(generation)) loading.value = false;
  }
}

async function saveInsert(values: ColumnValue[]): Promise<void> {
  await mutate(
    () =>
      insertRow({
        connectionId: props.connectionId,
        tableName: props.tableName,
        values,
      }),
    () => {
      inserting.value = false;
    },
  );
}

async function saveEdit(values: ColumnValue[]): Promise<void> {
  const row = editing.value;
  if (!row?.identity || !page.value) return;
  const expected = expectedValues(row, page.value);
  const changes = values.filter(
    (entry) =>
      !sameValue(
        entry.value,
        expected.find((old) => old.column === entry.column)!.value,
      ),
  );
  if (!changes.length) {
    editing.value = null;
    return;
  }
  await mutate(
    () =>
      updateRow({
        connectionId: props.connectionId,
        tableName: props.tableName,
        identity: row.identity!,
        expected,
        changes,
      }),
    () => {
      editing.value = null;
    },
  );
}

async function confirmDelete(): Promise<void> {
  const row = deleting.value;
  if (!row?.identity || !page.value) return;
  const expected = expectedValues(row, page.value);
  await mutate(
    () =>
      deleteRow({
        connectionId: props.connectionId,
        tableName: props.tableName,
        identity: row.identity!,
        expected,
      }),
    () => {
      deleting.value = null;
    },
  );
}

function expectedValues(
  row: BrowseRow,
  currentPage: BrowseTableResponse,
): ColumnValue[] {
  return currentPage.columns.map((column, index) => ({
    column,
    value: row.values[index],
  }));
}

async function mutate(
  action: () => Promise<MutationResult>,
  done: () => void,
): Promise<void> {
  if (mutating.value) return;
  const context = tableKey.value;
  const offset = page.value?.offset ?? 0;
  mutating.value = true;
  error.value = "";

  try {
    await action();
    if (!active || context !== tableKey.value) return;
    done();
    await load(offset);
  } catch (cause) {
    if (active && context === tableKey.value)
      error.value = normalizeBackendError(cause).message;
  } finally {
    if (active) mutating.value = false;
  }
}

async function exportPage(format: ExportFormat): Promise<void> {
  const displayed = page.value;
  if (!displayed || busy.value) return;
  error.value = "";
  let destinationPath: string | null;
  try {
    destinationPath = await chooseExportDestination(format);
  } catch (cause) {
    error.value = `Could not choose ${format.toUpperCase()} destination: ${normalizeBackendError(cause).message}`;
    return;
  }
  if (!destinationPath) return;

  const context = tableKey.value;
  exporting.value = true;
  try {
    const response = await exportRows({
      columns: displayed.columns,
      rows: displayed.rows.map((row) => row.values),
      format,
      destinationPath,
    });
    if (active && context === tableKey.value)
      status.value = `Exported ${response.rowCount} displayed rows to ${response.path}`;
  } catch (cause) {
    if (active && context === tableKey.value)
      error.value = exportError(cause);
  } finally {
    if (active && context === tableKey.value) exporting.value = false;
  }
}

function preview(row: BrowseRow): string {
  return row.values.slice(0, 2).map(displayValue).join(", ");
}

function exportError(cause: unknown): string {
  const backendError = normalizeBackendError(cause);
  return backendError.code === "destination_exists"
    ? "A file already exists at that destination. Choose a new path; existing files are never overwritten."
    : backendError.message;
}
</script>

<template>
  <section class="table-workspace">
    <header class="workspace-heading">
      <div>
        <p class="eyebrow">Table browser</p>
        <h2>{{ tableName }}</h2>
        <p v-if="page" class="metadata">
          {{ page.metadata.identityKind }} identity ·
          {{ page.metadata.columns.length }} columns
        </p>
      </div>
      <button
        class="primary-button compact"
        :disabled="!canMutate || busy"
        :title="mutationReason ?? undefined"
        type="button"
        @click="inserting = true"
      >
        Insert row
      </button>
    </header>
    <p v-if="mutationReason" class="hint">
      Editing unavailable: {{ mutationReason }}
    </p>
    <p v-if="error" class="error-message" role="alert">{{ error }}</p>
    <p v-if="status" class="status-message" role="status">{{ status }}</p>
    <p v-if="loading" class="status-message" role="status">Loading rows…</p>
    <template v-if="page">
      <div class="export-actions" aria-label="Export displayed table page">
        <span>Export page ({{ page.rows.length }} displayed rows)</span>
        <button type="button" :disabled="busy" @click="exportPage('json')">Export JSON</button>
        <button type="button" :disabled="busy" @click="exportPage('csv')">Export CSV</button>
      </div>
      <div class="table-grid">
        <ResultGrid
          :columns="page.columns"
          :rows="page.rows.map((row) => row.values)"
        />
        <aside v-if="canMutate" class="row-actions" aria-label="Row actions">
          <div v-for="(row, index) in page.rows" :key="index">
            <button :disabled="!row.identity || busy" @click="editing = row">
              Edit
            </button>
            <button :disabled="!row.identity || busy" @click="deleting = row">
              Delete
            </button>
          </div>
        </aside>
      </div>
      <nav class="pagination" aria-label="Table pages">
        <button
          :disabled="busy || page.offset === 0"
          @click="load(Math.max(0, page.offset - page.limit))"
        >
          Previous
        </button>
        <span
          >Rows {{ page.offset + 1 }}–{{ page.offset + page.rows.length }}</span
        >
        <button
          :disabled="busy || !page.hasMore"
          @click="load(page.offset + page.limit)"
        >
          Next
        </button>
      </nav>
      <RowEditor
        v-if="inserting"
        :key="`insert:${tableKey}`"
        insert
        :columns="page.metadata.columns"
        :primary-key="page.metadata.primaryKey"
        :busy="busy"
        @cancel="inserting = false"
        @save="saveInsert"
      />
      <RowEditor
        v-if="editing"
        :key="editorKey"
        :columns="page.metadata.columns"
        :initial="editing.values"
        :primary-key="page.metadata.primaryKey"
        :busy="busy"
        @cancel="editing = null"
        @save="saveEdit"
      />
    </template>
    <ConfirmDialog
      :open="Boolean(deleting)"
      title="Delete row?"
      :message="`Delete this row from ${tableName}: ${deleting ? preview(deleting) : ''}. This cannot be undone.`"
      @cancel="deleting = null"
      @confirm="confirmDelete"
    />
  </section>
</template>

<style scoped>
.table-workspace {
  display: grid;
  gap: 11px;
}
.workspace-heading {
  display: flex;
  justify-content: space-between;
  align-items: end;
}
.workspace-heading h2 {
  margin: 0;
  font-size: 19px;
}
.metadata,
.hint {
  margin: 4px 0 0;
  color: var(--muted);
  font-size: 11px;
}
.compact {
  width: auto;
  padding: 8px 10px;
}
.table-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 74px;
  gap: 6px;
}
.row-actions {
  display: grid;
  grid-template-columns: 1fr;
  align-content: start;
  gap: 5px;
  padding-top: 36px;
}
.row-actions div {
  display: flex;
  gap: 3px;
}
.row-actions button,
.pagination button {
  padding: 5px;
  border: 1px solid var(--line);
  border-radius: 4px;
  background: var(--surface);
  color: inherit;
  font-size: 10px;
}
.pagination {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--muted);
  font-size: 11px;
}
.export-actions { display: flex; align-items: center; flex-wrap: wrap; gap: 6px; color: var(--muted); font-size: 11px; }
.export-actions button { padding: 5px; border: 1px solid var(--line); border-radius: 4px; background: var(--surface); color: inherit; font-size: 10px; }
</style>
