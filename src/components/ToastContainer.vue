<script setup lang="ts">
export interface Toast {
  id: number;
  message: string;
  kind: "info" | "error" | "success";
}

defineProps<{
  toasts: Toast[];
}>();

const emit = defineEmits<{
  (e: "remove", id: number): void;
}>();
</script>

<template>
  <div class="fixed bottom-6 right-6 flex flex-col gap-3 z-[1000] pointer-events-none">
    <transition-group name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="pointer-events-auto bg-[var(--card-bg)] border-2 border-[var(--border-color)] p-4 min-w-[300px] shadow-[4px_4px_0_var(--border-color)] cursor-pointer flex items-center gap-3"
        :class="{
            'border-[var(--error-color)]': toast.kind === 'error',
            'border-[var(--success-color)]': toast.kind === 'success'
        }"
        @click="emit('remove', toast.id)"
      >
        <div class="w-3 h-3 bg-[var(--text-color)]" :class="{
            'bg-[var(--error-color)]': toast.kind === 'error',
            'bg-[var(--success-color)]': toast.kind === 'success',
            'bg-[var(--accent-color)]': toast.kind === 'info'
        }"></div>
        <span class="font-mono text-xs font-bold uppercase tracking-wide">{{ toast.message }}</span>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s cubic-bezier(0, 0, 0.2, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(50px);
}
</style>
