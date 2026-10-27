<script setup lang="ts">
import type { SqlValue } from "../types/database";
import { displayValue } from "../utils/sqliteValue";
defineProps<{ columns: string[]; rows: SqlValue[][]; emptyLabel?: string }>();
</script>
<template>
  <div class="grid-scroll" tabindex="0" aria-label="Query results">
    <table v-if="columns.length">
      <thead>
        <tr>
          <th v-for="column in columns" :key="column" scope="col">
            {{ column }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, index) in rows" :key="index">
          <td
            v-for="(value, cell) in row"
            :key="cell"
            :class="{ null: value.kind === 'null' }"
            :title="displayValue(value)"
          >
            {{ displayValue(value) }}
          </td>
        </tr>
      </tbody>
    </table>
    <p v-else class="empty-grid">{{ emptyLabel ?? "No result rows." }}</p>
  </div>
</template>
<style scoped>
.grid-scroll {
  max-height: 280px;
  overflow: auto;
  border: 1px solid var(--line);
  border-radius: 7px;
  background: var(--surface);
}
table {
  width: max-content;
  min-width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}
th,
td {
  max-width: 320px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--line);
  text-align: left;
  vertical-align: top;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
}
th {
  position: sticky;
  top: 0;
  background: var(--surface-strong);
  color: var(--muted);
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.08em;
}
td.null {
  color: #c496d7;
  font-style: italic;
}
.empty-grid {
  margin: 0;
  padding: 24px;
  color: var(--muted);
  font-size: 12px;
}
</style>
