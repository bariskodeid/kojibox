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
  <div class="toast-container">
    <transition-group name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :data-kind="toast.kind"
        @click="emit('remove', toast.id)"
      >
        <span class="message">{{ toast.message }}</span>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 24px;
  right: 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  z-index: 1000;
  pointer-events: none;
}

.toast {
  pointer-events: auto;
  background: #ffffff;
  border: 2px solid #1b1b1b;
  padding: 12px 16px;
  min-width: 280px;
  box-shadow: 4px 4px 0 #1b1b1b;
  cursor: pointer;
  display: flex;
  align-items: center;
}

.toast[data-kind="error"] {
  border-color: #d96a6a;
  background: #fff5f5;
}

.toast[data-kind="success"] {
  border-color: #0b7a3e;
  background: #f0fff4;
}

.message {
  font-size: 13px;
  font-weight: 500;
}

/* Transitions */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateX(30px);
}
</style>
