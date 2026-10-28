<script setup lang="ts">
import { Clock, Database, Download, FolderOpen, Pencil, SquareTerminal, Star, Table2 } from "@lucide/vue";
import type { ConnectionCatalogEntry } from "../types/connectionCatalog";

defineProps<{
  selectedPath: string;
  opening: boolean;
  pickerBusy: boolean;
  errorMessage: string;
  persistenceWarning: boolean;
  savedConnections: ConnectionCatalogEntry[];
  recentConnections: ConnectionCatalogEntry[];
}>();

defineEmits<{
  "update:selectedPath": [path: string];
  "clear-error": [];
  choose: [];
  submit: [];
  open: [path: string];
  forget: [entry: ConnectionCatalogEntry];
}>();

function formatOpenedAt(timestamp: string): string {
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(timestamp));
}
</script>

<template>
  <section class="relative mx-auto grid min-h-0 max-w-5xl flex-1 items-center gap-12 overflow-auto px-6 py-16 lg:grid-cols-[1fr_400px]">
    <div class="relative max-w-xl">
      <span class="mb-5 inline-flex items-center gap-2 rounded-full border border-accent/20 bg-accent/8 px-3 py-1.5 text-2xs font-medium text-accent"><span class="size-1.5 rounded-full bg-accent"></span>Local-first SQLite</span>
      <h1 class="text-4xl font-semibold tracking-[-0.04em] text-white sm:text-5xl">A focused workspace<br /><span class="text-accent">for your databases.</span></h1>
      <p class="mt-5 max-w-md text-sm leading-7 text-slate-400">Browse schemas, edit data, run queries, and export results without your database ever leaving this device.</p>
      <ul class="mt-7 flex flex-wrap gap-2">
        <li class="chip"><Table2 :size="11" /> Browse schemas</li>
        <li class="chip"><Pencil :size="11" /> Edit rows</li>
        <li class="chip"><SquareTerminal :size="11" /> Run SQL</li>
        <li class="chip"><Download :size="11" /> Export data</li>
      </ul>
    </div>
    <form class="rounded-2xl border border-white/10 bg-panel p-6 shadow-2xl shadow-black/20" @submit.prevent="$emit('submit')">
      <div class="mb-5 grid size-11 place-items-center rounded-xl border border-accent/15 bg-accent/8 text-accent" aria-hidden="true"><Database :size="20" /></div>
      <div>
        <h2 class="text-base font-semibold text-white">Open a SQLite database</h2>
        <p class="mt-1 text-xs leading-5 text-slate-500">Enter the path to, or choose, an existing <code>.db</code>, <code>.sqlite</code>, or <code>.sqlite3</code> file.</p>
      </div>
      <label class="field-label mt-6" for="database-path">Database path</label>
      <div class="relative mt-2">
        <span class="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-slate-600" aria-hidden="true"><FolderOpen :size="13" /></span>
        <input
          id="database-path"
          :value="selectedPath"
          class="field pl-9"
          type="text"
          placeholder="/path/to/database.sqlite"
          :disabled="opening || pickerBusy"
          @input="$emit('update:selectedPath', ($event.target as HTMLInputElement).value); $emit('clear-error')"
        />
      </div>
      <div class="mt-3 grid grid-cols-2 gap-2">
        <button class="btn btn-primary" :disabled="opening || pickerBusy">{{ opening ? "Opening…" : "Open database" }}</button>
        <button type="button" class="btn" :disabled="opening || pickerBusy" @click="$emit('choose')">
          <FolderOpen :size="13" /> {{ pickerBusy ? "Choosing…" : "Choose file" }}
        </button>
      </div>
      <p v-if="errorMessage" class="alert alert-error mt-3" role="alert">{{ errorMessage }}</p>
      <p v-if="persistenceWarning" class="alert alert-warn mt-3" role="status">
        Connection metadata could not be saved. Database work still works; check local storage permissions.
      </p>
    </form>
    <section
      v-if="savedConnections.length || recentConnections.length"
      class="col-span-full grid max-h-64 gap-5 overflow-auto rounded-xl border border-white/8 bg-[#0f131a] p-5 lg:col-start-2"
      aria-label="Saved and recent databases"
    >
      <div v-if="savedConnections.length" class="space-y-2">
        <h2 class="eyebrow flex items-center gap-1.5"><Star :size="11" class="text-accent/70" /> Saved</h2>
        <ul class="catalog-list">
          <li v-for="entry in savedConnections" :key="entry.id">
            <span class="ml-2 grid size-4 shrink-0 place-items-center text-slate-600" aria-hidden="true"><Database :size="11" /></span>
            <button type="button" class="grid min-w-0 flex-1 rounded-lg px-3 py-2 text-left transition hover:bg-white/5" :disabled="opening || pickerBusy" @click="$emit('open', entry.path)">
              <strong class="truncate text-xs font-medium text-slate-200">{{ entry.displayName }}</strong><span class="truncate text-2xs text-slate-500">{{ entry.path }}</span>
            </button>
            <button type="button" class="px-2 text-2xs text-slate-600 hover:text-rose-300" :disabled="opening || pickerBusy" :aria-label="`Forget saved connection ${entry.displayName}; database files are not deleted`" @click="$emit('forget', entry)">Forget</button>
          </li>
        </ul>
      </div>
      <div v-if="recentConnections.length" class="space-y-2">
        <h2 class="eyebrow flex items-center gap-1.5"><Clock :size="11" /> Recent</h2>
        <ul class="catalog-list">
          <li v-for="entry in recentConnections" :key="entry.id">
            <span class="ml-2 grid size-4 shrink-0 place-items-center text-slate-600" aria-hidden="true"><Database :size="11" /></span>
            <button type="button" class="grid min-w-0 flex-1 rounded-lg px-3 py-2 text-left transition hover:bg-white/5" :disabled="opening || pickerBusy" @click="$emit('open', entry.path)">
              <strong class="truncate text-xs font-medium text-slate-200">{{ entry.displayName }}</strong><span class="truncate text-2xs text-slate-500">{{ entry.path }} · {{ formatOpenedAt(entry.lastOpenedAt) }}</span>
            </button>
            <button type="button" class="px-2 text-2xs text-slate-600 hover:text-rose-300" :disabled="opening || pickerBusy" :aria-label="`Forget recent connection ${entry.displayName}; database files are not deleted`" @click="$emit('forget', entry)">Forget</button>
          </li>
        </ul>
      </div>
      <p class="text-2xs text-slate-600">Forget removes LiteStation metadata only. It never deletes database files.</p>
    </section>
  </section>
</template>
