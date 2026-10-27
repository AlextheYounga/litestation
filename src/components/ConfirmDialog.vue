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
    class="fixed inset-0 z-50 grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    role="presentation"
    @mousedown.self="cancel"
  >
    <section
      ref="dialog"
      class="dialog w-full max-w-sm rounded-xl border border-white/10 bg-[#151920] p-5 shadow-2xl shadow-black/60"
      role="alertdialog"
      aria-modal="true"
       :aria-labelledby="titleId"
       :aria-describedby="messageId"
      @keydown="handleKeydown"
    >
      <h2 :id="titleId" class="text-base font-semibold text-white">{{ title }}</h2>
      <p :id="messageId" class="mt-2 text-xs leading-5 text-slate-400">{{ message }}</p>
      <div class="mt-5 flex justify-end gap-2">
        <button ref="cancelButton" type="button" class="rounded-md border border-white/10 px-3 py-2 text-xs text-slate-300 hover:bg-white/5" @click="cancel">Cancel</button>
        <button
          type="button"
          class="rounded-md border border-rose-400/20 bg-rose-500/15 px-3 py-2 text-xs font-medium text-rose-200 hover:bg-rose-500/25"
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
/* Keep the SFC style module stable during Vite HMR; styling uses Tailwind. */
</style>
