<script setup lang="ts">
import { computed } from "vue";
import type { ServiceState, LogEntry } from "../types";

const props = defineProps<{
  service: ServiceState;
  logs: LogEntry[];
  health: string;
  logFilter: "all" | "error";
  logPath?: string;
  busy: boolean;
}>();

const emit = defineEmits<{
  (e: "start", id: string): void;
  (e: "stop", id: string): void;
  (e: "restart", id: string): void;
  (e: "export-logs", id: string): void;
  (e: "clear-logs", id: string): void;
  (e: "fix-runtime", id: string): void;
}>();

const filteredLogs = computed(() => {
  return (props.logs || []).filter((item) =>
    props.logFilter === "all" ? true : item.level === "error"
  );
});

const isBinaryError = computed(() => {
    if (!props.service.lastError) return false;
    const err = props.service.lastError.toLowerCase();
    return err.includes("binary not found") || err.includes("not available") || err.includes("no such file");
});

function formatTs(ts: string) {
  const value = Number(ts);
  if (!Number.isFinite(value)) return ts;
  return new Date(value * 1000).toLocaleTimeString();
}
</script>

<template>
  <article class="card">
    <div class="card-head">
      <div>
        <h2>{{ service.id }}</h2>
        <p class="status" :data-state="service.state">{{ service.state }}</p>
      </div>
      <span class="pid" v-if="service.pid">pid {{ service.pid }}</span>
    </div>
    <p class="health" :data-health="health">
      health: {{ health || "unknown" }}
    </p>
    <p v-if="service.lastError" class="error-inline">
      {{ service.lastError }}
    </p>
    <div class="actions">
      <button v-if="isBinaryError" class="secondary" @click="emit('fix-runtime', service.id)">
          Fix Runtime
      </button>
      <button
        class="primary"
        :disabled="busy"
        @click="emit('start', service.id)"
      >
        Start
      </button>
      <button
        class="secondary"
        :disabled="busy"
        @click="emit('stop', service.id)"
      >
        Stop
      </button>
      <button
        class="ghost"
        :disabled="busy"
        @click="emit('restart', service.id)"
      >
        Restart
      </button>
    </div>
    <div class="logs">
      <div class="logs-head">
        <p class="logs-title">Recent logs</p>
        <div class="log-actions">
            <button class="ghost small" @click="emit('clear-logs', service.id)">Clear</button>
            <button class="ghost small" @click="emit('export-logs', service.id)">Export</button>
        </div>
      </div>
      <p class="logs-path" v-if="logPath">File: {{ logPath }}</p>
      <ul>
        <li v-for="(entry, index) in filteredLogs" :key="index">
          <span class="log-ts">{{ formatTs(entry.ts) }}</span>
          <span class="log-level" :data-level="entry.level">{{
            entry.level
          }}</span>
          <span class="log-message">{{ entry.message }}</span>
        </li>
      </ul>
    </div>
  </article>
</template>

<style scoped>
.card {
  background: #ffffff;
  border: 2px solid #1b1b1b;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  box-shadow: 4px 4px 0 #1b1b1b;
}

.card-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
}

.status {
  margin: 6px 0 0;
  font-weight: 600;
  text-transform: uppercase;
  font-size: 12px;
}

.status[data-state="running"] {
  color: #0b7a3e;
}

.status[data-state="starting"] {
  color: #c17b1b;
}

.status[data-state="stopped"] {
  color: #b23b3b;
}

.pid {
  font-size: 12px;
  color: #5b5b5b;
}

.health {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #4a4a4a;
}

.health[data-health="ok"] {
  color: #0b7a3e;
}

.health[data-health="error"] {
  color: #b23b3b;
}

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
}

.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

button {
  border: 2px solid #1b1b1b;
  background: #fefefe;
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.primary {
  background: #ffd36a;
}

.secondary {
  background: #c7e5ff;
}

.ghost {
  background: #ffffff;
}

.logs {
  border-top: 1px dashed #1b1b1b;
  padding-top: 8px;
  font-size: 12px;
  color: #333;
}

.logs-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.log-actions {
    display: flex;
    gap: 4px;
}

.logs-title {
  margin: 0 0 6px;
  text-transform: uppercase;
  font-weight: 700;
  font-size: 10px;
  letter-spacing: 0.08em;
}

button.small {
    padding: 2px 6px;
    font-size: 10px;
}

.logs-path {
  margin: 0 0 6px;
  font-size: 10px;
  color: #666;
}

.logs ul {
  margin: 0;
  padding-left: 16px;
  max-height: 120px;
  overflow: auto;
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
