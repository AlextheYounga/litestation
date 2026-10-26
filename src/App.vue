<script setup lang="ts">
import { computed, ref } from "vue";
import SchemaBrowser from "./components/SchemaBrowser.vue";
import {
  chooseSqliteFile,
  closeDatabase,
  normalizeBackendError,
  openLocalDatabase,
} from "./services/database";
import type { ConnectionMode, DatabaseConnection } from "./types/database";

type ViewState = "disconnected" | "opening" | "error" | "connected";

const mode = ref<ConnectionMode>("readOnly");
const selectedPath = ref("");
const connection = ref<DatabaseConnection | null>(null);
const state = ref<ViewState>("disconnected");
const errorMessage = ref("");

const isOpening = computed(() => state.value === "opening");
const hasConnection = computed(() => connection.value !== null);
const actionLabel = computed(() => (isOpening.value ? "Opening…" : "Choose database"));

async function openDatabase(): Promise<void> {
  if (isOpening.value || hasConnection.value) {
    return;
  }

  let path: string | null;
  try {
    path = await chooseSqliteFile();
  } catch (error) {
    state.value = "error";
    errorMessage.value = normalizeBackendError(error).message;
    return;
  }

  if (!path) {
    return;
  }

  selectedPath.value = path;
  errorMessage.value = "";
  state.value = "opening";

  try {
    connection.value = await openLocalDatabase(path, mode.value);
    state.value = "connected";
  } catch (error) {
    connection.value = null;
    state.value = "error";
    errorMessage.value = normalizeBackendError(error).message;
  }
}

async function closeConnection(): Promise<void> {
  if (!connection.value || isOpening.value) {
    return;
  }

  try {
    await closeDatabase(connection.value.id);
    connection.value = null;
    selectedPath.value = "";
    errorMessage.value = "";
    state.value = "disconnected";
  } catch (error) {
    errorMessage.value = normalizeBackendError(error).message;
    state.value = "error";
  }
}

function displayPath(path: string): string {
  return path || "No database selected";
}
</script>

<template>
  <main class="app-shell">
    <header class="topbar">
      <span class="brand" aria-label="LiteStation">
        <span class="brand-mark" aria-hidden="true">L</span>
        <span>LiteStation</span>
      </span>
      <span class="product-label">SQLite workspace</span>
    </header>

    <section v-if="!hasConnection" class="welcome-panel" aria-labelledby="welcome-heading">
      <div class="intro">
        <p class="eyebrow">Local database</p>
        <h1 id="welcome-heading">Open a database<br /><em>with confidence.</em></h1>
        <p class="intro-copy">A focused place to inspect your SQLite schema. Your database stays on this device.</p>
      </div>

      <form class="open-card" @submit.prevent="openDatabase">
        <fieldset :disabled="isOpening">
          <legend>Connection mode</legend>
          <div class="mode-options">
            <label class="mode-option" :class="{ selected: mode === 'readOnly' }">
              <input v-model="mode" type="radio" value="readOnly" name="mode" />
              <span class="radio-dot" aria-hidden="true"></span>
              <span><strong>Read-only</strong><small>Safest for exploring</small></span>
            </label>
            <label class="mode-option" :class="{ selected: mode === 'readWrite' }">
              <input v-model="mode" type="radio" value="readWrite" name="mode" />
              <span class="radio-dot" aria-hidden="true"></span>
              <span><strong>Read &amp; write</strong><small>Changes can be made</small></span>
            </label>
          </div>
        </fieldset>

        <p v-if="mode === 'readWrite'" class="write-warning" role="note">
          <span aria-hidden="true">!</span> Writable mode can change or remove data in this file.
        </p>

        <div class="selected-file" :class="{ populated: selectedPath }">
          <span class="file-icon" aria-hidden="true">▧</span>
          <span class="file-details"><strong>{{ selectedPath ? "Selected file" : "Choose a SQLite file" }}</strong><small>{{ displayPath(selectedPath) }}</small></span>
        </div>

        <button class="primary-button" type="submit" :disabled="isOpening">
          <span>{{ actionLabel }}</span><span aria-hidden="true">→</span>
        </button>

        <p v-if="isOpening" class="status-message" role="status" aria-live="polite">Opening database and reading schema…</p>
        <p v-if="errorMessage" class="error-message" role="alert"><strong>Couldn’t open database.</strong> {{ errorMessage }}</p>
      </form>
    </section>

    <section v-else class="workspace" aria-labelledby="connection-heading">
      <aside class="connection-sidebar">
        <p class="eyebrow">Active connection</p>
        <div class="connection-card">
          <span class="connection-icon" aria-hidden="true">⌁</span>
          <div class="connection-name"><h1 id="connection-heading">{{ connection?.displayName }}</h1><p>{{ connection?.mode === 'readOnly' ? 'Read-only' : 'Read & write' }}</p></div>
          <span class="live-dot" title="Connected"></span>
        </div>
        <div class="path-block"><span>Location</span><strong :title="connection?.path">{{ connection?.path }}</strong></div>
        <button class="close-button" type="button" @click="closeConnection"><span aria-hidden="true">×</span> Close connection</button>
        <p v-if="errorMessage" class="error-message" role="alert">{{ errorMessage }}</p>
      </aside>
      <div class="schema-area"><SchemaBrowser v-if="connection" :schema="connection.schema" /></div>
    </section>
  </main>
</template>

<style>
:root { font-family: Inter, ui-sans-serif, system-ui, sans-serif; color: #e8e8e3; background: #101111; font-synthesis: none; text-rendering: optimizeLegibility; -webkit-font-smoothing: antialiased; }
* { box-sizing: border-box; }
body { margin: 0; min-width: 320px; min-height: 100vh; }
button, input { font: inherit; }
button { cursor: pointer; }
button:focus-visible, input:focus-visible, a:focus-visible { outline: 2px solid #cfed66; outline-offset: 3px; }
.app-shell { min-height: 100vh; background: radial-gradient(circle at 76% 10%, #28302b 0, transparent 36%), #101111; }
.topbar { display: flex; align-items: center; justify-content: space-between; max-width: 1240px; margin: auto; padding: 28px 42px; color: #aeb3aa; font-size: 12px; letter-spacing: .04em; }
.brand { display: flex; align-items: center; gap: 10px; color: #f0f1eb; text-decoration: none; font-weight: 700; letter-spacing: -.02em; font-size: 15px; }
.brand-mark { display: grid; place-items: center; width: 25px; height: 25px; color: #171914; background: #cfed66; border-radius: 7px; font-weight: 900; }
.product-label { color: #70766d; }
.welcome-panel { display: grid; grid-template-columns: 1fr minmax(320px, 450px); gap: clamp(50px, 10vw, 150px); align-items: center; max-width: 1040px; margin: 0 auto; padding: clamp(70px, 13vh, 150px) 42px 100px; }
.eyebrow { margin: 0 0 14px; color: #9daa77; font-size: 11px; font-weight: 700; letter-spacing: .14em; text-transform: uppercase; }
h1, h2, h3, p { margin-top: 0; }
.intro h1 { margin-bottom: 25px; color: #f2f3ed; font-size: clamp(39px, 5vw, 66px); line-height: .98; letter-spacing: -.065em; }
.intro h1 em { color: #cfed66; font-style: normal; }
.intro-copy { max-width: 350px; color: #8e948b; line-height: 1.7; font-size: 14px; }
.open-card { padding: 27px; border: 1px solid #333832; border-radius: 16px; background: #1a1d1b; box-shadow: 0 24px 70px #0005; }
fieldset { border: 0; margin: 0; padding: 0; }
legend { margin-bottom: 15px; color: #c5c8bd; font-size: 12px; font-weight: 600; }
.mode-options { display: grid; gap: 8px; }
.mode-option { display: flex; align-items: center; gap: 12px; padding: 13px; border: 1px solid #353a35; border-radius: 9px; color: #b2b7ad; cursor: pointer; }
.mode-option.selected { border-color: #879b43; background: #252d20; color: #edf0e6; }
.mode-option input { position: absolute; opacity: 0; }
.radio-dot { width: 15px; height: 15px; border: 1px solid #6c7368; border-radius: 50%; }
.selected .radio-dot { border: 4px solid #cfed66; }
.mode-option strong, .mode-option small { display: block; }
.mode-option strong { font-size: 13px; }
.mode-option small { margin-top: 2px; color: #7e857b; font-size: 11px; }
.write-warning { display: flex; gap: 9px; margin: 16px 0 0; color: #e8bc70; font-size: 11px; line-height: 1.5; }
.write-warning span { display: inline-grid; flex: 0 0 16px; place-items: center; height: 16px; border: 1px solid #9d7134; border-radius: 50%; font-weight: 700; }
.selected-file { display: flex; align-items: center; gap: 11px; margin: 21px 0 13px; padding: 11px 2px; color: #767d73; }
.selected-file.populated { color: #c9cec3; }
.file-icon, .connection-icon { color: #cfed66; font-size: 21px; }
.file-details { min-width: 0; }
.file-details strong, .file-details small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.file-details strong { color: #cfd4c9; font-size: 12px; }
.file-details small { max-width: 355px; margin-top: 3px; color: #747b71; font-size: 11px; }
.primary-button { display: flex; align-items: center; justify-content: space-between; width: 100%; padding: 13px 15px; border: 0; border-radius: 8px; color: #171914; background: #cfed66; font-size: 13px; font-weight: 800; }
.primary-button:hover:not(:disabled) { background: #e1fa83; }
button:disabled { cursor: wait; opacity: .6; }
.status-message, .error-message { margin: 13px 0 0; font-size: 11px; line-height: 1.5; }
.status-message { color: #aebd7d; }
.error-message { color: #ee9b87; }
.workspace { display: grid; grid-template-columns: 285px 1fr; max-width: 1240px; min-height: calc(100vh - 80px); margin: 20px auto 0; border: 1px solid #30352f; border-bottom: 0; border-radius: 13px 13px 0 0; overflow: hidden; background: #161817; }
.connection-sidebar { padding: 34px 26px; border-right: 1px solid #30352f; background: #1b1e1c; }
.connection-card { display: flex; align-items: center; gap: 10px; padding: 13px 0 20px; border-bottom: 1px solid #30352f; }
.connection-name { min-width: 0; flex: 1; }
.connection-name h1 { overflow: hidden; margin: 0 0 4px; color: #edf0e7; font-size: 14px; text-overflow: ellipsis; white-space: nowrap; }
.connection-name p { margin: 0; color: #aebb72; font-size: 11px; }
.live-dot { width: 7px; height: 7px; border-radius: 50%; background: #cfed66; box-shadow: 0 0 0 4px #cfed661f; }
.path-block { padding: 22px 0 28px; }
.path-block span { display: block; margin-bottom: 7px; color: #70776d; font-size: 10px; text-transform: uppercase; letter-spacing: .1em; }
.path-block strong { display: block; overflow-wrap: anywhere; color: #aeb4aa; font-size: 11px; font-weight: 400; line-height: 1.6; }
.close-button { padding: 0; border: 0; color: #9c7771; background: transparent; font-size: 11px; }
.close-button:hover { color: #e7a095; }
.close-button span { margin-right: 5px; font-size: 18px; vertical-align: -2px; }
.schema-area { padding: 34px clamp(26px, 5vw, 70px); }
.section-heading { display: flex; align-items: end; justify-content: space-between; padding-bottom: 20px; border-bottom: 1px solid #30352f; }
.section-heading h2 { margin: 0; color: #edf0e7; font-size: 25px; letter-spacing: -.04em; }
.schema-summary { color: #70786d; font-size: 11px; }
.schema-groups { display: grid; grid-template-columns: repeat(2, minmax(180px, 1fr)); gap: 35px 60px; padding-top: 25px; }
.schema-group h3 { display: flex; justify-content: space-between; margin-bottom: 12px; color: #aeb5a8; font-size: 11px; letter-spacing: .08em; text-transform: uppercase; }
.count { color: #71796e; font-weight: 400; }
.object-list { display: grid; gap: 7px; margin: 0; padding: 0; list-style: none; }
.object-list li { display: flex; align-items: center; gap: 9px; padding: 8px 0; color: #d1d5cc; font-size: 13px; }
.object-mark { width: 5px; height: 5px; border: 1px solid #cfed66; border-radius: 1px; }
.empty-group { margin: 0; color: #626a61; font-size: 12px; }
@media (max-width: 700px) { .topbar { padding: 22px 22px; } .welcome-panel { display: block; padding: 65px 22px; } .intro { margin-bottom: 45px; } .workspace { display: block; margin: 10px 12px 0; } .connection-sidebar { border-right: 0; border-bottom: 1px solid #30352f; } .schema-groups { grid-template-columns: 1fr; gap: 28px; } .schema-area { padding: 28px 22px; } }
@media (prefers-color-scheme: light) { :root { color: #20231f; background: #f2f3ed; } .app-shell { background: radial-gradient(circle at 75% 0, #e2ebc4 0, transparent 35%), #f2f3ed; } .topbar { color: #6d7468; } .brand { color: #20231f; } .product-label { color: #92998c; } .intro h1 { color: #20231f; } .intro-copy { color: #667064; } .open-card, .workspace, .connection-sidebar { background: #fff; border-color: #d9ded2; } .mode-option { border-color: #d9ded2; color: #657064; } .mode-option.selected { background: #f0f6df; border-color: #a8bd63; color: #273021; } .selected-file { color: #778073; } .file-details strong { color: #3d463a; } .file-details small, .schema-summary, .path-block span { color: #858e80; } .workspace, .connection-sidebar, .section-heading { border-color: #d9ded2; } .connection-name h1, .section-heading h2 { color: #283027; } .connection-name p { color: #71883a; } .path-block strong, .object-list li { color: #4d594b; } .schema-group h3 { color: #677263; } .empty-group { color: #98a092; } }
</style>
