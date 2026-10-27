<script setup lang="ts">
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { sql } from "@codemirror/lang-sql";
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
const props = defineProps<{ modelValue: string }>();
const emit = defineEmits<{
  "update:modelValue": [value: string];
  run: [selection: string, cursor: number];
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
  );
}
defineExpose({ run });
</script>
<template>
  <div class="relative overflow-hidden rounded-xl border border-white/10 bg-[#0b0e13] shadow-inner shadow-black/20">
    <div ref="host" class="min-h-[260px] text-left" aria-label="SQL editor"></div>
    <button type="button" class="absolute bottom-3 right-3 rounded-lg bg-[#cfed66] px-3 py-2 text-xs font-semibold text-[#182010] shadow-lg shadow-[#cfed66]/10 transition hover:bg-[#d9f57c]" @click="run">
      Run <kbd class="ml-1 text-[#52612a]">⌘↵</kbd>
    </button>
  </div>
</template>
