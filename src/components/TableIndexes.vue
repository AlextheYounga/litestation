<script setup lang="ts">
import { computed } from "vue";
import { KeySquare } from "@lucide/vue";
import type { SchemaItem } from "../types/database";

const props = defineProps<{
  tableName: string;
  indexes: SchemaItem[];
}>();

const tableIndexes = computed(() =>
  props.indexes.filter((index) => index.tableName === props.tableName),
);
</script>

<template>
  <div class="min-h-0 flex-1 overflow-auto rounded-lg border border-white/10 bg-surface">
    <table v-if="tableIndexes.length" class="w-full min-w-[640px] table-fixed text-left">
      <thead>
        <tr>
          <th class="sticky top-0 w-56 border-b border-r border-white/8 bg-raised px-3 py-2 text-2xs font-semibold uppercase tracking-wider text-slate-500" scope="col">Name</th>
          <th class="sticky top-0 border-b border-white/8 bg-raised px-3 py-2 text-2xs font-semibold uppercase tracking-wider text-slate-500" scope="col">Definition</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="index in tableIndexes" :key="index.name" class="align-top odd:bg-white/[.015] hover:bg-white/[.04]">
          <td class="border-b border-r border-white/8 px-3 py-2.5 font-mono text-xs text-slate-300">{{ index.name }}</td>
          <td class="border-b border-white/8 px-3 py-2.5 font-mono text-xs leading-5 text-slate-400">{{ index.sql ?? "SQLite-managed index" }}</td>
        </tr>
      </tbody>
    </table>
    <div v-else class="grid h-full min-h-48 place-items-center px-6 text-center">
      <div>
        <span class="grid size-10 place-items-center justify-self-center rounded-xl border border-white/8 bg-white/[.03] text-slate-500" aria-hidden="true"><KeySquare :size="16" /></span>
        <p class="mt-2 text-sm font-medium text-slate-400">No indexes</p>
        <p class="mt-1 text-xs text-slate-600">This table has no user-defined indexes.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>