<script setup lang="ts">
defineProps<{
  activeTab: string;
}>();

const emit = defineEmits<{
  (e: "change", tab: string): void;
}>();

const tabs = [
  { id: "dashboard", label: "Dashboard" },
  { id: "projects", label: "Projects" },
  { id: "tooling", label: "Tooling" },
  { id: "settings", label: "Config" },
];
</script>

<template>
  <nav class="flex w-full mb-8 border-b-2 border-[var(--border-color)] bg-[var(--bg-color)]">
    <button
      v-for="tab in tabs"
      :key="tab.id"
      @click="emit('change', tab.id)"
      class="relative px-8 py-3 -mb-[2px] font-mono font-bold uppercase tracking-wider text-sm border-2 border-transparent transition-all focus:outline-none"
      :class="{
        'bg-[var(--card-bg)] text-[var(--accent-color)] border-[var(--border-color)] border-b-[var(--card-bg)] z-10': activeTab === tab.id,
        'text-[var(--secondary-color)] hover:text-[var(--text-color)] hover:bg-[var(--code-bg)] border-b-[var(--border-color)]': activeTab !== tab.id
      }"
    >
      <span class="mr-2 opacity-50 text-xs">/</span>{{ tab.label }}
    </button>
    <!-- Filler line to complete the border -->
    <div class="flex-grow border-b-2 border-[var(--border-color)]"></div>
  </nav>
</template>