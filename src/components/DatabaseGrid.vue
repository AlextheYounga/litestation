<script setup lang="ts">
import { computed } from "vue";
import { AgGridVue } from "ag-grid-vue3";
import {
  ClientSideRowModelModule,
  ModuleRegistry,
  RowSelectionModule,
  TooltipModule,
  themeQuartz,
  type AutoSizeStrategy,
  type ColDef,
  type GetRowIdParams,
  type GridSizeChangedEvent,
  type RowDoubleClickedEvent,
  type RowSelectionOptions,
  type SelectionChangedEvent,
} from "ag-grid-community";
import type { BrowseRow, SqlValue } from "../types/database";
import { displayValue } from "../utils/sqliteValue";

ModuleRegistry.registerModules([
  ClientSideRowModelModule,
  RowSelectionModule,
  TooltipModule,
]);

interface GridRow {
  key: string;
  source: BrowseRow;
}

const props = defineProps<{
  columns: string[];
  rows: BrowseRow[];
  offset: number;
  loading?: boolean;
}>();

const emit = defineEmits<{
  "selection-change": [rows: BrowseRow[]];
  "row-double-click": [row: BrowseRow];
}>();

const gridTheme = themeQuartz.withParams({
  accentColor: "#cfed66",
  backgroundColor: "#0d1016",
  borderColor: "#252a33",
  browserColorScheme: "dark",
  cellFontFamily: "InterVariable, Inter, ui-sans-serif, system-ui, sans-serif",
  cellFontSize: 12,
  foregroundColor: "#cbd5e1",
  headerBackgroundColor: "#151a23",
  headerFontFamily: "InterVariable, Inter, ui-sans-serif, system-ui, sans-serif",
  headerFontSize: 10,
  headerTextColor: "#64748b",
  oddRowBackgroundColor: "#0f1218",
  rowHoverColor: "rgba(255, 255, 255, 0.04)",
  rowHeight: 36,
  selectedRowBackgroundColor: "rgba(207, 237, 102, 0.10)",
  wrapperBorderRadius: 8,
});

const defaultColumnDefinition: ColDef<GridRow, SqlValue> = {
  editable: false,
  filter: false,
  minWidth: 120,
  resizable: true,
  sortable: false,
  suppressHeaderMenuButton: true,
};

// Fill the available grid width whenever columns change (new table, reload).
const autoSizeStrategy: AutoSizeStrategy = {
  type: "fitGridWidth",
  defaultMinWidth: 120,
};

const columnDefinitions = computed<ColDef<GridRow, SqlValue>[]>(() =>
  props.columns.map((name, index) => ({
    colId: `column-${index}`,
    headerName: name,
    valueGetter: ({ data }) => data?.source.values[index],
    valueFormatter: ({ value }) => (value ? displayValue(value) : ""),
    tooltipValueGetter: ({ value }) => (value ? displayValue(value) : ""),
  })),
);

const rowData = computed<GridRow[]>(() =>
  props.rows.map((source, index) => ({
    key: `${props.offset + index}:${JSON.stringify(source.identity)}`,
    source,
  })),
);

const rowSelection: RowSelectionOptions<GridRow> = {
  mode: "multiRow",
  checkboxes: true,
  headerCheckbox: true,
  enableClickSelection: true,
  selectAll: "currentPage",
};

function getRowId({ data }: GetRowIdParams<GridRow>): string {
  return data.key;
}

function selectionChanged(event: SelectionChangedEvent<GridRow>): void {
  emit(
    "selection-change",
    (event.selectedNodes ?? [])
      .map((node) => node.data?.source)
      .filter((row): row is BrowseRow => Boolean(row)),
  );
}

function rowDoubleClicked(event: RowDoubleClickedEvent<GridRow>): void {
  if (event.data) emit("row-double-click", event.data.source);
}

function gridSizeChanged(event: GridSizeChangedEvent<GridRow>): void {
  // Keep columns filling the window as it is resized.
  event.api.sizeColumnsToFit();
}
</script>

<template>
  <div class="min-h-0 flex-1 overflow-hidden rounded-lg">
    <AgGridVue
      class="size-full"
      :theme="gridTheme"
      :column-defs="columnDefinitions"
      :default-col-def="defaultColumnDefinition"
      :row-data="rowData"
      :row-selection="rowSelection"
      :selection-column-def="{ width: 42, minWidth: 42, maxWidth: 42, resizable: false }"
      :get-row-id="getRowId"
      :loading="loading"
      :animate-rows="false"
      :suppress-cell-focus="false"
      :auto-size-strategy="autoSizeStrategy"
      @selection-changed="selectionChanged"
      @row-double-clicked="rowDoubleClicked"
      @grid-size-changed="gridSizeChanged"
    />
  </div>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
