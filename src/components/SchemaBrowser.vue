<script setup lang="ts">
import { computed } from "vue";
import type { DatabaseSchema, SchemaItem } from "../types/database";

const props = defineProps<{ schema: DatabaseSchema }>();

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
  <section class="schema-panel" aria-labelledby="schema-heading">
    <div class="section-heading">
      <div>
        <p class="eyebrow">Object browser</p>
        <h2 id="schema-heading">Schema</h2>
      </div>
      <span class="schema-summary">{{ objectTotal }} objects</span>
    </div>

    <div class="schema-groups">
      <section v-for="group in groups" :key="group.key" class="schema-group" :aria-labelledby="`${group.key}-heading`">
        <h3 :id="`${group.key}-heading`">
          <span>{{ group.label }}</span>
          <span class="count">{{ schema[group.key].length }}</span>
        </h3>
        <ul v-if="schema[group.key].length" class="object-list">
          <li v-for="item in schema[group.key]" :key="`${group.key}-${item.name}`">
            <span class="object-mark" aria-hidden="true"></span>
            <span>{{ itemLabel(item) }}</span>
          </li>
        </ul>
        <p v-else class="empty-group">No {{ group.label.toLowerCase() }} found</p>
      </section>
    </div>
  </section>
</template>
