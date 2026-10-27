<script setup lang="ts">
import { computed } from "vue";
import type { DatabaseSchema, SchemaItem } from "../types/database";

const props = defineProps<{
  schema: DatabaseSchema;
  selectedTable?: string | null;
  disabled?: boolean;
}>();
const emit = defineEmits<{ selectTable: [name: string] }>();

const groups: Array<{ key: keyof DatabaseSchema; label: string }> = [
  { key: "tables", label: "Tables" },
  { key: "views", label: "Views" },
  { key: "indexes", label: "Indexes" },
  { key: "triggers", label: "Triggers" },
];

const objectTotal = computed<number>(() =>
  groups.reduce((total, group) => total + props.schema[group.key].length, 0),
);

function itemLabel(item: SchemaItem): string {
  return item.name;
}
</script>

<template>
  <section class="mt-7" aria-labelledby="schema-heading">
    <div class="flex items-center justify-between px-2">
      <div>
        <p class="text-[10px] font-semibold uppercase tracking-widest text-slate-600">Object browser</p>
        <h2 id="schema-heading" class="mt-1 text-xs font-semibold text-slate-300">Schema</h2>
      </div>
      <span class="text-[10px] text-slate-600">{{ objectTotal }}</span>
    </div>

    <div class="schema-groups">
      <section
        v-for="group in groups"
        :key="group.key"
        class="mt-5"
        :aria-labelledby="`${group.key}-heading`"
      >
        <h3 :id="`${group.key}-heading`" class="mb-1 flex items-center justify-between px-2 text-[10px] font-medium uppercase tracking-widest text-slate-600">
          <span>{{ group.label }}</span>
          <span>{{ schema[group.key].length }}</span>
        </h3>
        <ul v-if="schema[group.key].length" class="space-y-0.5">
          <li
            v-for="item in schema[group.key]"
            :key="`${group.key}-${item.name}`"
          >
            <span class="ml-2 size-1 shrink-0 rounded-full bg-slate-600" aria-hidden="true"></span>
            <button
              v-if="group.key === 'tables'"
              type="button"
              :disabled="disabled"
              class="w-full truncate rounded-md px-2 py-1.5 text-left text-xs text-slate-400 transition hover:bg-white/5 hover:text-slate-200 disabled:hover:bg-transparent"
              :class="selectedTable === item.name ? 'bg-[#cfed66]/10 text-[#cfed66]' : ''"
              @click="emit('selectTable', item.name)"
            >
              {{ itemLabel(item) }}
            </button>
            <span v-else class="truncate px-2 py-1 text-xs text-slate-500" :title="item.sql ?? undefined">{{
              itemLabel(item)
            }}</span>
          </li>
        </ul>
        <p v-else class="px-2 text-[11px] text-slate-700">
          No {{ group.label.toLowerCase() }} found
        </p>
      </section>
    </div>
  </section>
</template>
