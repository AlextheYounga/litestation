<script setup lang="ts">
import { Inbox } from "@lucide/vue";
import type { SqlValue } from "../types/database";
import { displayValue } from "../utils/sqliteValue";
defineProps<{ columns: string[]; rows: SqlValue[][]; emptyLabel?: string }>();
</script>
<template>
  <div
    class="grid-scroll max-h-[360px] overflow-auto rounded-xl border border-white/10 bg-surface"
    tabindex="0"
    aria-label="Query results"
  >
    <table v-if="columns.length" class="w-max min-w-full text-left">
      <thead>
        <tr>
          <th class="sticky top-0 z-10 w-10 border-b border-r border-white/8 bg-raised px-2 py-2 text-right text-2xs font-semibold text-slate-600" scope="col" aria-label="Row number">#</th>
          <th v-for="column in columns" :key="column" class="sticky top-0 z-10 min-w-28 max-w-72 border-b border-r border-white/8 bg-raised px-2.5 py-2 text-2xs font-semibold uppercase tracking-wider text-slate-500" scope="col">
            {{ column }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, index) in rows" :key="index" class="group odd:bg-white/[.015] hover:bg-white/[.04]">
          <td class="h-9 border-b border-r border-white/6 px-2 py-2 text-right align-middle text-2xs tabular-nums text-slate-600">
            {{ index + 1 }}
          </td>
          <td
            v-for="(value, cell) in row"
            :key="cell"
            class="h-9 max-w-72 truncate border-b border-r border-white/6 px-2.5 py-2 align-middle text-xs text-slate-300"
            :class="value.kind === 'null' ? 'italic text-accent/70' : ''"
            :title="displayValue(value)"
          >
            {{ displayValue(value) }}
          </td>
        </tr>
      </tbody>
    </table>
    <div v-else class="grid place-items-center px-8 py-10 text-center">
      <div>
        <span class="grid size-9 place-items-center justify-self-center rounded-lg border border-white/8 bg-white/[.03] text-slate-500" aria-hidden="true"><Inbox :size="15" /></span>
        <p class="mt-2 text-xs text-slate-500">{{ emptyLabel ?? "No result rows." }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
