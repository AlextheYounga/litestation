<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { Save, X } from "@lucide/vue";
import type { ColumnValue, SqlValue, TableColumn } from "../types/database";
import { defaultKind, parseValue } from "../utils/sqliteValue";

const props = defineProps<{
  columns: TableColumn[];
  initial?: SqlValue[];
  primaryKey: string[];
  insert?: boolean;
  busy?: boolean;
}>();

const emit = defineEmits<{ save: [values: ColumnValue[]]; cancel: [] }>();

type Field = { include: boolean; kind: SqlValue["kind"]; raw: string };

const fields = reactive<Record<string, Field>>({});
const error = ref("");
const title = computed(() => (props.insert ? "Insert row" : "Edit row"));

function resetFields(): void {
  for (const name of Object.keys(fields)) delete fields[name];

  for (const [index, column] of props.columns.entries()) {
    const value = props.initial?.[index];
    fields[column.name] = {
      include: !props.insert,
      kind: value?.kind ?? defaultKind(column.declaredType),
      raw: value?.kind === "null" ? "" : value ? String(value.value) : "",
    };
  }
  error.value = "";
}

watch(() => [props.columns, props.initial, props.insert], resetFields, {
  deep: true,
  immediate: true,
});

function submit(): void {
  const values: ColumnValue[] = [];

  for (const column of props.columns) {
    const field = fields[column.name];
    if (!field.include) continue;

    const value = parseValue(field.kind, field.raw);
    if (typeof value === "string") {
      error.value = `${column.name}: ${value}`;
      return;
    }
    values.push({ column: column.name, value });
  }

  error.value = "";
  emit("save", values);
}
</script>

<template>
  <form class="fixed inset-y-14 right-0 z-30 flex w-full max-w-md flex-col overflow-hidden border-l border-white/10 bg-[#11151b] shadow-2xl shadow-black/50" @submit.prevent="submit">
    <header class="flex h-14 shrink-0 items-center justify-between border-b border-white/8 px-4">
      <div>
        <p class="text-[10px] font-semibold uppercase tracking-widest text-[#cfed66]">Row details</p>
        <h3 class="mt-0.5 text-sm font-semibold text-slate-100">{{ title }}</h3>
      </div>
      <button type="button" class="grid size-8 place-items-center rounded-md text-slate-500 hover:bg-white/5 hover:text-white" aria-label="Close row editor" title="Close" @click="emit('cancel')"><X :size="16" /></button>
    </header>
    <div class="flex-1 space-y-4 overflow-y-auto p-4">
      <p v-if="insert" class="rounded-md border border-white/8 bg-white/[.025] px-3 py-2 text-[11px] leading-5 text-slate-500">
        Unchecked columns are omitted so database defaults can apply.
      </p>
      <label v-for="column in columns" :key="column.name" class="block">
        <span class="mb-1.5 flex items-center gap-2 text-[11px] font-medium text-slate-300">
          <input
            v-if="insert"
            v-model="fields[column.name].include"
            class="accent-[#cfed66]"
            type="checkbox"
          />
          <span class="min-w-0 truncate">{{ column.name }}</span>
          <small class="ml-auto shrink-0 font-mono text-[9px] uppercase text-slate-600">{{ column.declaredType ?? "untyped" }}</small>
        </span>
        <div v-if="fields[column.name].include" class="grid grid-cols-[90px_minmax(0,1fr)] gap-1.5">
          <select
            v-model="fields[column.name].kind"
            class="min-w-0 rounded-md border border-white/10 bg-[#0b0e13] px-2 py-2 text-[11px] text-slate-400"
            :disabled="primaryKey.includes(column.name) && !insert"
          >
            <option value="null">NULL</option>
            <option value="integer">integer</option>
            <option value="real">real</option>
            <option value="text">text</option>
            <option value="blob">blob</option>
          </select>
          <input
            v-model="fields[column.name].raw"
            class="min-w-0 rounded-md border border-white/10 bg-[#0b0e13] px-2.5 py-2 text-xs text-slate-200 disabled:text-slate-600"
            :disabled="
              fields[column.name].kind === 'null' ||
              (primaryKey.includes(column.name) && !insert)
            "
            :aria-label="`${column.name} value`"
          />
        </div>
      </label>
      <p v-if="error" class="rounded-md border border-rose-400/20 bg-rose-400/5 px-3 py-2 text-xs text-rose-300" role="alert">{{ error }}</p>
    </div>
    <footer class="shrink-0 border-t border-white/8 p-3">
      <button class="flex h-9 w-full items-center justify-center gap-2 rounded-md bg-[#cfed66] text-xs font-semibold text-[#182010] hover:bg-[#d9f57c]" :disabled="busy" type="submit">
        <Save :size="14" /> {{ insert ? "Insert row" : "Save changes" }}
      </button>
    </footer>
  </form>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
