<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, ref, watch } from "vue";
import {
  ChevronLeft,
  ChevronRight,
  Download,
  Pencil,
  Plus,
  RefreshCw,
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
      <div>
        <p class="text-[10px] font-semibold uppercase tracking-widest text-[#cfed66]">Table browser</p>
        <div class="mt-1 flex items-baseline gap-2">
          <h2 class="text-lg font-semibold tracking-tight text-white">{{ tableName }}</h2>
          <span v-if="page" class="text-[11px] text-slate-600">{{ page.metadata.columns.length }} columns</span>
        </div>
      </div>
      <div v-if="activeTab === 'data'" class="flex items-center gap-1.5">
        <button
          type="button"
          class="grid size-8 place-items-center rounded-md border border-white/10 text-slate-400 transition hover:bg-white/5 hover:text-white"
          :disabled="busy"
          title="Refresh rows"
          aria-label="Refresh rows"
          @click="load(page?.offset ?? 0)"
        ><RefreshCw :size="14" :class="{ 'animate-spin': loading }" /></button>
        <button
          class="flex h-8 items-center gap-1.5 rounded-md bg-[#cfed66] px-3 text-xs font-semibold text-[#182010] transition hover:bg-[#d9f57c] disabled:hover:bg-[#cfed66]"
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
      <button type="button" class="border-b-2 px-1 pb-2 text-xs font-medium transition" :class="activeTab === 'data' ? 'border-[#cfed66] text-[#cfed66]' : 'border-transparent text-slate-500 hover:text-slate-300'" :aria-current="activeTab === 'data' ? 'page' : undefined" @click="activeTab = 'data'">Data</button>
      <button type="button" class="border-b-2 px-1 pb-2 text-xs font-medium transition" :class="activeTab === 'indexes' ? 'border-[#cfed66] text-[#cfed66]' : 'border-transparent text-slate-500 hover:text-slate-300'" :aria-current="activeTab === 'indexes' ? 'page' : undefined" @click="activeTab = 'indexes'">
        Indexes <span class="ml-1 text-[10px] text-slate-600">{{ tableIndexCount }}</span>
      </button>
    </nav>
    <template v-if="activeTab === 'data'">
    <p v-if="mutationReason" class="rounded-lg border border-amber-400/15 bg-amber-400/5 px-3 py-2 text-xs text-amber-300">
      Editing unavailable: {{ mutationReason }}
    </p>
    <p v-if="error" class="rounded-lg border border-rose-400/20 bg-rose-400/5 px-3 py-2 text-xs text-rose-300" role="alert">{{ error }}</p>
    <p v-if="status" class="text-xs text-emerald-300" role="status">{{ status }}</p>
    <p v-if="loading" class="text-xs text-slate-500" role="status">Loading rows…</p>
    <template v-if="page">
      <div class="flex shrink-0 flex-wrap items-center gap-1.5" aria-label="Table actions">
        <span class="mr-auto text-[11px] text-slate-600">{{ selectedRows.length ? `${selectedRows.length} selected` : `${page.rows.length} displayed rows` }} · {{ page.metadata.identityKind }} identity</span>
        <button
          type="button"
          class="grid size-7 place-items-center rounded-md border border-white/10 text-slate-500 hover:bg-white/5 hover:text-white"
          :disabled="!canMutate || !selectedEditableRow || busy"
          aria-label="Edit selected row"
          title="Edit selected row"
          @click="editSelectedRow"
        ><Pencil :size="13" /></button>
        <button
          type="button"
          class="grid size-7 place-items-center rounded-md border border-white/10 text-slate-600 hover:bg-rose-400/10 hover:text-rose-300"
          :disabled="!canMutate || !selectedEditableRow || busy"
          aria-label="Delete selected row"
          title="Delete selected row"
          @click="deleteSelectedRow"
        ><Trash2 :size="13" /></button>
        <button type="button" class="flex h-7 items-center gap-1.5 rounded-md border border-white/10 px-2.5 text-[11px] text-slate-400 hover:bg-white/5 hover:text-white" :disabled="busy" @click="exportPage('json')"><Download :size="12" /> JSON</button>
        <button type="button" class="flex h-7 items-center gap-1.5 rounded-md border border-white/10 px-2.5 text-[11px] text-slate-400 hover:bg-white/5 hover:text-white" :disabled="busy" @click="exportPage('csv')"><Download :size="12" /> CSV</button>
      </div>
      <DatabaseGrid
        :columns="page.columns"
        :rows="page.rows"
        :offset="page.offset"
        :loading="loading"
        @selection-change="selectedRows = $event"
        @row-double-click="editDoubleClickedRow"
      />
      <nav class="flex shrink-0 items-center justify-between text-xs text-slate-500" aria-label="Table pages">
        <button class="flex h-7 items-center gap-1 rounded-md border border-white/10 px-2 text-[11px] hover:bg-white/5 hover:text-white" :disabled="busy || page.offset === 0"
          @click="load(Math.max(0, page.offset - page.limit))"
        >
          <ChevronLeft :size="13" /> Previous
        </button>
        <span
          >Rows {{ page.offset + 1 }}–{{ page.offset + page.rows.length }}</span
        >
        <button class="flex h-7 items-center gap-1 rounded-md border border-white/10 px-2 text-[11px] hover:bg-white/5 hover:text-white" :disabled="busy || !page.hasMore"
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
