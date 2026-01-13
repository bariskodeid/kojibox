<script setup lang="ts">
import { ref, computed } from "vue";
import type { ServiceState, LogEntry } from "../types";

const props = defineProps<{
  services: ServiceState[];
  logsByService: Record<string, LogEntry[]>;
}>();

const emit = defineEmits<{
  (e: "export", params: { service: string | null; level: string | null; limit: number }): void;
}>();

const logViewerService = ref("all");
const logViewerLevel = ref("all");
const logViewerLimit = ref(200);

function formatTs(ts: string) {
  const value = Number(ts);
  if (!Number.isFinite(value)) return ts;
  return new Date(value * 1000).toLocaleTimeString();
}

const viewerEntries = computed(() => {
  const entries: Array<LogEntry & { service: string }> = [];
  for (const [serviceId, items] of Object.entries(props.logsByService)) {
    for (const entry of items) {
      entries.push({ ...entry, service: entry.service || serviceId });
    }
  }
  entries.sort((a, b) => Number(a.ts) - Number(b.ts));
  
  let filtered = entries;
  if (logViewerService.value !== "all") {
    filtered = filtered.filter((entry) => entry.service === logViewerService.value);
  }
  if (logViewerLevel.value !== "all") {
    filtered = filtered.filter((entry) => entry.level === logViewerLevel.value);
  }
  
  const limit = logViewerLimit.value || 200;
  if (filtered.length > limit) {
    filtered = filtered.slice(filtered.length - limit);
  }
  return filtered;
});

function onExport() {
  emit("export", {
    service: logViewerService.value === "all" ? null : logViewerService.value,
    level: logViewerLevel.value === "all" ? null : logViewerLevel.value,
    limit: logViewerLimit.value,
  });
}
</script>

<template>
  <section class="card">
    <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
        <h3 class="text-lg font-black uppercase">Global Logs</h3>
        <span class="tech-label">SYS_OUT</span>
    </div>

    <div class="flex gap-4 mb-4 items-end bg-[var(--code-bg)] p-2 border border-[var(--border-color)]">
      <div class="flex-1">
        <label class="tech-label">SERVICE_FILTER</label>
        <select v-model="logViewerService" class="input font-mono uppercase text-xs h-8 py-1">
          <option value="all">ALL SERVICES</option>
          <option v-for="service in services" :key="service.id" :value="service.id">
            {{ service.id.toUpperCase() }}
          </option>
        </select>
      </div>
      <div class="w-32">
        <label class="tech-label">LOG_LEVEL</label>
        <select v-model="logViewerLevel" class="input font-mono uppercase text-xs h-8 py-1">
          <option value="all">ALL LEVELS</option>
          <option value="info">INFO</option>
          <option value="error">ERROR</option>
        </select>
      </div>
      <div class="w-24">
        <label class="tech-label">LIMIT</label>
        <input v-model.number="logViewerLimit" type="number" min="1" class="input font-mono text-xs h-8 py-1" />
      </div>
      <button class="btn h-8" @click="onExport">EXPORT</button>
    </div>

    <div class="bg-black text-white p-2 border-2 border-[var(--border-color)] h-64 overflow-y-auto font-mono text-[10px] custom-scrollbar shadow-inner">
      <ul class="space-y-0.5">
        <li v-for="(entry, index) in viewerEntries" :key="index" class="whitespace-pre-wrap break-all hover:bg-gray-900">
          <span class="text-gray-500 mr-2">{{ formatTs(entry.ts) }}</span>
          <span :class="entry.level === 'error' ? 'text-red-500 font-bold' : 'text-green-500'" class="mr-2">[{{ entry.level.toUpperCase() }}]</span>
          <span class="text-blue-400 mr-2">[{{ entry.service }}]</span>
          <span class="text-gray-300">{{ entry.message }}</span>
        </li>
        <li v-if="viewerEntries.length === 0" class="text-gray-600 italic text-center mt-4">// END OF BUFFER</li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
/* Scoped styles removed */
</style>
