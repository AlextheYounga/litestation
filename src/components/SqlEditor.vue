<script setup lang="ts">
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { sql } from "@codemirror/lang-sql";
import { Play, SquareTerminal } from "@lucide/vue";
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
const props = defineProps<{ modelValue: string }>();
const emit = defineEmits<{
  "update:modelValue": [value: string];
  run: [selection: string, cursor: number, source: string];
}>();
const host = ref<HTMLElement>();
let view: EditorView | undefined;
onMounted(() => {
  view = new EditorView({
    parent: host.value,
    state: EditorState.create({
      doc: props.modelValue,
      extensions: [
        sql(),
        keymap.of([
          {
            key: "Mod-Enter",
            run: (target) => {
              emit(
                "run",
                target.state.sliceDoc(
                  target.state.selection.main.from,
                  target.state.selection.main.to,
                ),
                target.state.selection.main.head,
                target.state.doc.toString(),
              );
              return true;
            },
          },
        ]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged)
            emit("update:modelValue", update.state.doc.toString());
        }),
      ],
    }),
  });
});
watch(
  () => props.modelValue,
  (value) => {
    if (view && value !== view.state.doc.toString())
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: value },
      });
  },
);
onBeforeUnmount(() => view?.destroy());
function run(): void {
  if (!view) return;
  const selection = view.state.selection.main;
  emit(
    "run",
    view.state.sliceDoc(selection.from, selection.to),
    selection.head,
    view.state.doc.toString(),
  );
}
defineExpose({ run });
</script>
<template>
  <div class="relative overflow-hidden rounded-xl border border-white/10 bg-[#0b0e13] shadow-inner shadow-black/20">
    <div class="flex h-10 items-center justify-between gap-2 border-b border-white/8 bg-[#0d1016] pl-3 pr-2">
      <div class="flex min-w-0 items-center gap-2">
        <span class="grid size-5 shrink-0 place-items-center rounded bg-[#cfed66]/10 text-[#cfed66]" aria-hidden="true"><SquareTerminal :size="11" /></span>
        <span class="truncate font-mono text-[11px] text-slate-400">query.sql</span>
      </div>
      <button type="button" class="flex h-7 shrink-0 items-center gap-1.5 rounded-lg bg-[#cfed66] px-3 text-xs font-semibold text-[#182010] shadow-lg shadow-[#cfed66]/10 transition hover:bg-[#d9f57c]" @click="run">
        <Play :size="12" /> Run <kbd class="text-[9px] font-semibold text-[#52612a]">⌘↵</kbd>
      </button>
    </div>
    <div ref="host" class="text-left" aria-label="SQL editor"></div>
  </div>
</template>

<style scoped>
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
