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
  <section class="notice">
    <h3>Log Viewer</h3>
    <div class="actions-inline">
      <label>
        Service
        <select v-model="logViewerService">
          <option value="all">all</option>
          <option v-for="service in services" :key="service.id" :value="service.id">
            {{ service.id }}
          </option>
        </select>
      </label>
      <label>
        Level
        <select v-model="logViewerLevel">
          <option value="all">all</option>
          <option value="info">info</option>
          <option value="error">error</option>
        </select>
      </label>
      <label>
        Limit
        <input v-model.number="logViewerLimit" type="number" min="1" />
      </label>
      <button class="ghost" @click="onExport">Export Logs</button>
    </div>
    <div class="logs viewer">
      <ul>
        <li v-for="(entry, index) in viewerEntries" :key="index">
          <span class="log-ts">{{ formatTs(entry.ts) }}</span>
          <span class="log-level" :data-level="entry.level">{{ entry.level }}</span>
          <span class="log-service">[{{ entry.service }}]</span>
          <span class="log-message">{{ entry.message }}</span>
        </li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.actions-inline {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-items: flex-end;
}

.actions-inline label {
  display: block;
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

select, input {
  display: block;
  margin-top: 4px;
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
}

button {
  border: 2px solid #1b1b1b;
  background: #ffffff;
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
  height: 38px; /* Match input height */
}

.logs {
  border-top: 1px dashed #1b1b1b;
  padding-top: 8px;
  font-size: 12px;
  color: #333;
  margin-top: 12px;
}

.logs.viewer ul {
  max-height: 240px;
  overflow: auto;
  margin: 0;
  padding-left: 16px;
}

.log-ts {
  color: #777;
  font-size: 10px;
  margin-right: 6px;
}

.log-level {
  font-weight: 700;
  text-transform: uppercase;
  font-size: 10px;
  margin-right: 6px;
}

.log-level[data-level="error"] {
  color: #b23b3b;
}

.log-level[data-level="info"] {
  color: #0b7a3e;
}

.log-service {
  font-size: 10px;
  color: #6b6b6b;
  margin-right: 6px;
}

.log-message {
  color: #333;
}
</style>
