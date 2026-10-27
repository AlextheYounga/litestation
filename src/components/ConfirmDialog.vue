<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, useId, watch } from "vue";

const props = withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    message: string;
    confirmLabel?: string;
    destructive?: boolean;
  }>(),
  { confirmLabel: "Delete row", destructive: true },
);
const emit = defineEmits<{ confirm: []; cancel: [] }>();
const instanceId = useId();
const titleId = `confirm-title-${instanceId}`;
const messageId = `confirm-message-${instanceId}`;

const dialog = ref<HTMLElement>();
const cancelButton = ref<HTMLButtonElement>();
let invokingElement: HTMLElement | null = null;

watch(
  () => props.open,
  async (open) => {
    if (open) {
      invokingElement =
        document.activeElement instanceof HTMLElement
          ? document.activeElement
          : null;
      await nextTick();
      cancelButton.value?.focus();
    } else {
      invokingElement?.focus();
      invokingElement = null;
    }
  },
);

onMounted(async () => {
  if (!props.open) return;
  await nextTick();
  cancelButton.value?.focus();
});

function cancel(): void {
  emit("cancel");
}

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") {
    event.preventDefault();
    cancel();
    return;
  }
  if (event.key !== "Tab") return;

  const buttons = [
    ...(dialog.value?.querySelectorAll<HTMLButtonElement>("button") ?? []),
  ];
  if (!buttons.length) return;
  const current = buttons.indexOf(document.activeElement as HTMLButtonElement);
  const next = event.shiftKey
    ? current <= 0
      ? buttons.length - 1
      : current - 1
    : current === buttons.length - 1
      ? 0
      : current + 1;
  event.preventDefault();
  buttons[next].focus();
}

onBeforeUnmount(() => invokingElement?.focus());
</script>

<template>
  <div
    v-if="open"
    class="backdrop"
    role="presentation"
    @mousedown.self="cancel"
  >
    <section
      ref="dialog"
      class="dialog"
      role="alertdialog"
      aria-modal="true"
       :aria-labelledby="titleId"
       :aria-describedby="messageId"
      @keydown="handleKeydown"
    >
      <h2 :id="titleId">{{ title }}</h2>
      <p :id="messageId">{{ message }}</p>
      <div>
        <button ref="cancelButton" type="button" @click="cancel">Cancel</button>
        <button
          type="button"
          :class="{ danger: destructive }"
          @click="emit('confirm')"
        >
          {{ confirmLabel }}
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  z-index: 3;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 20px;
  background: #0009;
}
.dialog {
  max-width: 390px;
  padding: 20px;
  border: 1px solid var(--line);
  border-radius: 9px;
  background: var(--surface-strong);
}
h2 {
  margin: 0 0 10px;
  font-size: 17px;
}
p {
  color: var(--muted);
  font-size: 13px;
  line-height: 1.5;
}
.dialog div {
  display: flex;
  justify-content: end;
  gap: 8px;
}
.dialog button {
  padding: 7px 10px;
  border: 1px solid var(--line);
  border-radius: 5px;
  background: var(--surface);
  color: inherit;
}
.dialog .danger {
  border-color: #9d554c;
  background: #8a3f38;
  color: #fff;
}
</style>
