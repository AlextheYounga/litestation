<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
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
  <form class="row-editor" @submit.prevent="submit">
    <header>
      <h3>{{ title }}</h3>
      <button type="button" class="plain" @click="emit('cancel')">Close</button>
    </header>
    <p v-if="insert" class="hint">
      Unchecked columns are omitted so database defaults can apply.
    </p>
    <label v-for="column in columns" :key="column.name" class="field">
      <span>
        <input
          v-if="insert"
          v-model="fields[column.name].include"
          type="checkbox"
        />
        {{ column.name }} <small>{{ column.declaredType ?? "untyped" }}</small>
      </span>
      <div v-if="fields[column.name].include" class="field-controls">
        <select
          v-model="fields[column.name].kind"
          :disabled="primaryKey.includes(column.name) && !insert"
        >
          <option value="null">NULL</option>
          <option value="integer">integer</option>
          <option value="real">real</option>
          <option value="text">text</option>
          <option value="blob">blob (base64)</option>
        </select>
        <input
          v-model="fields[column.name].raw"
          :disabled="
            fields[column.name].kind === 'null' ||
            (primaryKey.includes(column.name) && !insert)
          "
          :aria-label="`${column.name} value`"
        />
      </div>
    </label>
    <p v-if="error" class="error-message" role="alert">{{ error }}</p>
    <button class="primary-button" :disabled="busy" type="submit">
      {{ insert ? "Insert row" : "Save changes" }}
    </button>
  </form>
</template>

<style scoped>
.row-editor {
  display: grid;
  gap: 9px;
  padding: 12px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--surface);
}
header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
h3 {
  margin: 0;
  font-size: 14px;
}
.field {
  display: grid;
  gap: 4px;
  font-size: 12px;
}
.field small,
.hint {
  color: var(--muted);
  font-size: 10px;
}
.field-controls {
  display: grid;
  grid-template-columns: 110px 1fr;
  gap: 6px;
}
.field input,
.field select {
  min-width: 0;
  padding: 6px;
  border: 1px solid var(--line);
  border-radius: 4px;
  background: var(--surface-strong);
  color: inherit;
}
.plain {
  border: 0;
  background: none;
  color: var(--muted);
}
</style>
