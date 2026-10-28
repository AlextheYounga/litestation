<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, ref, watch } from "vue";
import {
  ChevronLeft,
  ChevronRight,
  Download,
  LoaderCircle,
  Pencil,
  Plus,
  RefreshCw,
  Table2,
  Trash2,
} from "@lucide/vue";
import ConfirmDialog from "./ConfirmDialog.vue";
import RowEditor from "./RowEditor.vue";
import TableIndexes from "./TableIndexes.vue";
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
  SchemaItem,
} from "../types/database";
import { displayValue, sameValue } from "../utils/sqliteValue";
import { TableRequestGuard } from "../utils/tableRequest";

const DatabaseGrid = defineAsyncComponent(() => import("./DatabaseGrid.vue"));

const props = defineProps<{
  connectionId: number;
  tableName: string;
  indexes: SchemaItem[];
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
const selectedRows = ref<BrowseRow[]>([]);
const activeTab = ref<"data" | "indexes">("data");
const requests = new TableRequestGuard();
let active = true;

const busy = computed(() => loading.value || mutating.value || exporting.value);
const canMutate = computed(() => Boolean(page.value?.metadata.editable));
const mutationReason = computed(() => page.value?.metadata.nonEditableReason);
const tableKey = computed(() => `${props.connectionId}:${props.tableName}`);
const tableIndexCount = computed(() =>
  props.indexes.filter((index) => index.tableName === props.tableName).length,
);
const selectedEditableRow = computed(() => {
  if (selectedRows.value.length !== 1) return null;
  return selectedRows.value[0].identity ? selectedRows.value[0] : null;
});
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
    selectedRows.value = [];
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
    if (active && requests.isCurrent(generation)) {
      page.value = response;
      selectedRows.value = [];
    }
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

function editSelectedRow(): void {
  if (selectedEditableRow.value) editing.value = selectedEditableRow.value;
}

function deleteSelectedRow(): void {
  if (selectedEditableRow.value) deleting.value = selectedEditableRow.value;
}

function editDoubleClickedRow(row: BrowseRow): void {
  if (canMutate.value && row.identity && !busy.value) editing.value = row;
}

function exportError(cause: unknown): string {
  const backendError = normalizeBackendError(cause);
  return backendError.code === "destination_exists"
    ? "A file already exists at that destination. Choose a new path; existing files are never overwritten."
    : backendError.message;
}
</script>

<template>
  <section class="flex h-full min-h-0 flex-col gap-3">
    <header class="flex min-h-14 items-center justify-between gap-4 border-b border-white/8 pb-3">
      <div class="flex min-w-0 items-center gap-2.5">
        <span class="grid size-7 shrink-0 place-items-center rounded-md bg-accent/10 text-accent" aria-hidden="true"><Table2 :size="14" /></span>
        <div class="min-w-0">
          <p class="eyebrow-accent">Table browser</p>
          <div class="mt-0.5 flex items-baseline gap-2">
            <h2 class="truncate text-lg font-semibold tracking-tight text-white">{{ tableName }}</h2>
            <span v-if="page" class="shrink-0 text-2xs text-slate-600">{{ page.metadata.columns.length }} columns</span>
          </div>
        </div>
      </div>
      <div v-if="activeTab === 'data'" class="flex items-center gap-1.5">
        <button
          type="button"
          class="icon-btn"
          :disabled="busy"
          title="Refresh rows"
          aria-label="Refresh rows"
          @click="load(page?.offset ?? 0)"
        ><RefreshCw :size="14" :class="{ 'animate-spin': loading }" /></button>
        <button
          class="btn btn-primary"
          :disabled="!canMutate || busy"
          :title="mutationReason ?? 'Insert row'"
          type="button"
          @click="inserting = true"
        >
          <Plus :size="14" /> Insert row
        </button>
      </div>
    </header>
    <nav class="flex shrink-0 gap-5 border-b border-white/8" aria-label="Table views">
      <button type="button" class="border-b-2 px-1 pb-2 text-xs font-medium transition" :class="activeTab === 'data' ? 'border-accent text-accent' : 'border-transparent text-slate-500 hover:text-slate-300'" :aria-current="activeTab === 'data' ? 'page' : undefined" @click="activeTab = 'data'">Data</button>
      <button type="button" class="border-b-2 px-1 pb-2 text-xs font-medium transition" :class="activeTab === 'indexes' ? 'border-accent text-accent' : 'border-transparent text-slate-500 hover:text-slate-300'" :aria-current="activeTab === 'indexes' ? 'page' : undefined" @click="activeTab = 'indexes'">
        Indexes <span class="ml-1 text-2xs text-slate-600">{{ tableIndexCount }}</span>
      </button>
    </nav>
    <template v-if="activeTab === 'data'">
    <p v-if="mutationReason" class="alert alert-warn">
      Editing unavailable: {{ mutationReason }}
    </p>
    <p v-if="error" class="alert alert-error" role="alert">{{ error }}</p>
    <p v-if="status" class="text-xs text-emerald-300" role="status">{{ status }}</p>
    <template v-if="page">
      <div class="flex shrink-0 flex-wrap items-center gap-1.5" aria-label="Table actions">
        <span class="mr-auto text-2xs text-slate-600">{{ selectedRows.length ? `${selectedRows.length} selected` : `${page.rows.length} displayed rows` }} · {{ page.metadata.identityKind }} identity</span>
        <button
          type="button"
          class="icon-btn"
          :disabled="!canMutate || !selectedEditableRow || busy"
          aria-label="Edit selected row"
          title="Edit selected row"
          @click="editSelectedRow"
        ><Pencil :size="13" /></button>
        <button
          type="button"
          class="icon-btn text-slate-600 hover:bg-rose-400/10 hover:text-rose-300"
          :disabled="!canMutate || !selectedEditableRow || busy"
          aria-label="Delete selected row"
          title="Delete selected row"
          @click="deleteSelectedRow"
        ><Trash2 :size="13" /></button>
        <button type="button" class="btn btn-sm" :disabled="busy" @click="exportPage('json')"><Download :size="12" /> JSON</button>
        <button type="button" class="btn btn-sm" :disabled="busy" @click="exportPage('csv')"><Download :size="12" /> CSV</button>
      </div>
      <div class="relative flex min-h-0 flex-1 flex-col" aria-label="Table data">
        <DatabaseGrid
          :columns="page.columns"
          :rows="page.rows"
          :offset="page.offset"
          :loading="loading"
          @selection-change="selectedRows = $event"
          @row-double-click="editDoubleClickedRow"
        />
        <div
          v-if="loading"
          class="overlay-in absolute inset-0 z-10 grid place-items-center rounded-lg bg-surface/70 backdrop-blur-[1px]"
          role="status"
        >
          <span class="flex items-center gap-2 rounded-full border border-white/10 bg-surface px-4 py-2 text-xs text-slate-400 shadow-xl">
            <LoaderCircle :size="13" class="animate-spin text-accent" /> Loading rows…
          </span>
        </div>
      </div>
      <nav class="flex shrink-0 items-center justify-between gap-2" aria-label="Table pages">
        <button
          class="btn btn-sm hover:border-white/10 hover:bg-white/5 hover:text-white disabled:hover:bg-transparent disabled:hover:text-slate-500"
          :disabled="busy || page.offset === 0"
          :title="page.offset === 0 ? 'Already on the first page' : 'Previous 100 rows'"
          @click="load(Math.max(0, page.offset - page.limit))"
        >
          <ChevronLeft :size="13" /> Previous
        </button>
        <span class="text-2xs tabular-nums text-slate-500">Rows {{ page.offset + 1 }}–{{ page.offset + page.rows.length }}</span>
        <button
          class="btn btn-sm hover:border-white/10 hover:bg-white/5 hover:text-white disabled:hover:bg-transparent disabled:hover:text-slate-500"
          :disabled="busy || !page.hasMore"
          :title="page.hasMore ? 'Next 100 rows' : 'No more rows'"
          @click="load(page.offset + page.limit)"
        >
          Next <ChevronRight :size="13" />
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
    </template>
    <TableIndexes v-else :table-name="tableName" :indexes="indexes" />
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
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
