<script setup lang="ts">
import { Database, Plus, X } from "@lucide/vue";
import type { DatabaseSession } from "../composables/useDatabaseSessions";

defineProps<{
  sessions: DatabaseSession[];
  activeConnectionId: number | null;
  opening: boolean;
  closingConnectionIds: Set<number>;
  busyConnectionIds: Set<number>;
}>();

defineEmits<{
  activate: [connectionId: number];
  close: [connectionId: number];
  open: [];
}>();
</script>

<template>
  <nav class="flex h-10 shrink-0 items-end gap-0.5 overflow-x-auto border-b border-white/8 bg-[#0b0e13] px-2" aria-label="Open databases">
    <div
      v-for="session in sessions"
      :key="session.connection.id"
      class="flex h-9 max-w-56 shrink-0 items-center rounded-t-lg border border-b-0 text-xs transition"
      :class="session.connection.id === activeConnectionId
        ? 'border-white/10 bg-surface text-slate-100'
        : 'border-transparent text-slate-500 hover:bg-white/[.035] hover:text-slate-300'"
    >
      <button
        type="button"
        class="flex min-w-0 flex-1 items-center gap-2 py-2 pl-3 text-left"
        :aria-current="session.connection.id === activeConnectionId ? 'page' : undefined"
        :title="session.connection.path"
        @click="$emit('activate', session.connection.id)"
      >
        <Database :size="12" class="shrink-0" aria-hidden="true" />
        <span class="truncate">{{ session.connection.displayName }}</span>
      </button>
      <button
        type="button"
        class="mr-1 grid size-5 shrink-0 place-items-center rounded text-slate-600 hover:bg-white/10 hover:text-slate-300"
        :class="{ 'animate-pulse': closingConnectionIds.has(session.connection.id) }"
        :disabled="closingConnectionIds.has(session.connection.id) || busyConnectionIds.has(session.connection.id)"
        :aria-label="`Close ${session.connection.displayName}`"
        :title="busyConnectionIds.has(session.connection.id) ? 'Wait for the current database operation to finish' : `Close ${session.connection.displayName}`"
        @click="$emit('close', session.connection.id)"
      ><X :size="11" aria-hidden="true" /></button>
    </div>
    <button
      type="button"
      class="mb-1 grid size-7 shrink-0 place-items-center rounded-md text-slate-500 transition hover:bg-white/5 hover:text-accent disabled:opacity-40"
      :disabled="opening"
      :aria-label="opening ? 'Opening database…' : 'Open another database'"
      :title="opening ? 'Opening database…' : 'Open another database'"
      @click="$emit('open')"
    ><Plus :size="14" aria-hidden="true" /></button>
  </nav>
</template>
