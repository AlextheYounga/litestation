<script setup lang="ts">
import type { SqlValue } from "../types/database";
import { displayValue } from "../utils/sqliteValue";
defineProps<{ columns: string[]; rows: SqlValue[][]; emptyLabel?: string }>();
</script>
<template>
  <div class="grid-scroll max-h-[360px] overflow-auto rounded-xl border border-white/10 bg-[#0d1016]" tabindex="0" aria-label="Query results">
    <table v-if="columns.length">
      <thead>
        <tr>
          <th v-for="column in columns" :key="column" class="sticky top-0 border-b border-white/10 bg-[#151a23] px-3 py-2.5 text-left text-[10px] font-semibold uppercase tracking-wider text-slate-500" scope="col">
            {{ column }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, index) in rows" :key="index">
          <td
            v-for="(value, cell) in row"
            :key="cell"
            class="max-w-[320px] border-b border-white/6 px-3 py-2.5 text-left align-top text-xs text-slate-300"
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
