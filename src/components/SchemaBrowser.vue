<script setup lang="ts">
import { computed } from "vue";
import { Table2, Eye, Zap } from "@lucide/vue";
import type { DatabaseSchema, SchemaItem } from "../types/database";

const props = defineProps<{
  schema: DatabaseSchema;
  selectedTable?: string | null;
  disabled?: boolean;
}>();
const emit = defineEmits<{ selectTable: [name: string] }>();

const groups: Array<{
  key: keyof DatabaseSchema;
  label: string;
  icon: typeof Table2;
}> = [
  { key: "tables", label: "Tables", icon: Table2 },
  { key: "views", label: "Views", icon: Eye },
  { key: "triggers", label: "Triggers", icon: Zap },
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
        <p class="eyebrow">Object browser</p>
        <h2 id="schema-heading" class="mt-1 text-xs font-semibold text-slate-300">Schema</h2>
      </div>
      <span class="chip rounded-full px-2 py-0.5">{{ objectTotal }}</span>
    </div>

    <div class="schema-groups">
      <section
        v-for="group in groups"
        :key="group.key"
        class="mt-5"
        :aria-labelledby="`${group.key}-heading`"
      >
        <h3 :id="`${group.key}-heading`" class="eyebrow mb-1.5 flex items-center gap-1.5 px-2">
          <component :is="group.icon" :size="11" class="shrink-0 text-slate-500" />
          <span class="min-w-0 flex-1">{{ group.label }}</span>
          <span v-if="schema[group.key].length" class="shrink-0 normal-case tracking-normal text-slate-600">{{ schema[group.key].length }}</span>
        </h3>
        <ul v-if="schema[group.key].length" class="space-y-0.5">
          <li
            v-for="item in schema[group.key]"
            :key="`${group.key}-${item.name}`"
          >
            <div class="group relative">
              <span
                v-if="group.key === 'tables' && selectedTable === item.name"
                class="absolute inset-y-1 left-0 w-0.5 rounded-full bg-accent"
                aria-hidden="true"
              ></span>
              <button
                v-if="group.key === 'tables'"
                type="button"
                :disabled="disabled"
                class="w-full truncate rounded-md px-3 py-1.5 text-left text-xs transition hover:bg-white/5 hover:text-slate-200 disabled:hover:bg-transparent"
                :class="selectedTable === item.name ? 'bg-accent/10 text-accent' : 'text-slate-400'"
                :title="item.name"
                @click="emit('selectTable', item.name)"
              >
                {{ itemLabel(item) }}
              </button>
              <span
                v-else
                class="block truncate rounded-md px-3 py-1.5 text-xs text-slate-500"
                :title="item.sql ?? item.name"
              >{{ itemLabel(item) }}</span>
            </div>
          </li>
        </ul>
        <p v-else class="px-2 text-xs text-slate-700">
          No {{ group.label.toLowerCase() }} found
        </p>
      </section>
    </div>
  </section>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>