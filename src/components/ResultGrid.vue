<script setup lang="ts">
import type { SqlValue } from "../types/database";
import { displayValue } from "../utils/sqliteValue";
defineProps<{ columns: string[]; rows: SqlValue[][]; emptyLabel?: string }>();
</script>
<template>
  <div
    class="grid-scroll max-h-[360px] overflow-auto rounded-xl border border-white/10 bg-[#0d1016]"
    tabindex="0"
    aria-label="Query results"
  >
    <table v-if="columns.length" class="w-max min-w-full text-left">
      <thead>
        <tr>
          <th v-for="column in columns" :key="column" class="sticky top-0 z-10 min-w-28 max-w-72 border-b border-r border-white/8 bg-[#151a23] px-2.5 py-2 text-[10px] font-semibold uppercase tracking-wider text-slate-500" scope="col">
            {{ column }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, index) in rows" :key="index" class="group hover:bg-white/[.025]">
          <td
            v-for="(value, cell) in row"
            :key="cell"
            class="h-9 max-w-72 truncate border-b border-r border-white/6 px-2.5 py-2 align-middle text-[11px] text-slate-300"
            :class="value.kind === 'null' ? 'italic text-[#cfed66]/70' : ''"
            :title="displayValue(value)"
          >
            {{ displayValue(value) }}
          </td>
        </tr>
      </tbody>
    </table>
    <p v-else class="m-0 p-8 text-center text-xs text-slate-500">{{ emptyLabel ?? "No result rows." }}</p>
  </div>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
