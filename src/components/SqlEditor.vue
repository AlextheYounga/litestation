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
  <div class="editor-shell">
    <div ref="host" class="editor" aria-label="SQL editor"></div>
    <button type="button" class="run-button" @click="run">
      Run <kbd>Ctrl↵</kbd>
    </button>
  </div>
</template>
<style scoped>
.editor-shell {
  position: relative;
  border: 1px solid var(--line);
  border-radius: 8px;
  overflow: hidden;
  background: var(--surface);
}
.editor {
  min-height: 130px;
  text-align: left;
}
.editor :deep(.cm-editor) {
  height: 160px;
  font:
    13px/1.55 ui-monospace,
    SFMono-Regular,
    monospace;
}
.editor :deep(.cm-scroller) {
  overflow: auto;
}
.run-button {
  position: absolute;
  right: 8px;
  bottom: 8px;
  padding: 6px 9px;
  border: 1px solid #71863b;
  border-radius: 5px;
  background: #cfed66;
  color: #182010;
  font-size: 11px;
  font-weight: 700;
}
kbd {
  font: inherit;
  opacity: 0.7;
}
</style>
