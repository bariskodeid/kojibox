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
  <article class="card group relative">
    <!-- Technical Corner Marker -->
    <div class="absolute top-0 right-0 p-1 bg-[var(--border-color)]">
        <div class="w-2 h-2 bg-[var(--accent-color)]"></div>
    </div>

    <div class="card-head mb-4 border-b-2 border-[var(--border-color)] pb-2 flex justify-between items-end">
      <div>
        <span class="tech-label">SERVICE_ID</span>
        <h2 class="text-2xl font-black uppercase tracking-tighter">{{ service.id }}</h2>
      </div>
      <div class="text-right">
        <span class="tech-label">STATUS</span>
        <p class="status font-mono font-bold uppercase" :data-state="service.state">
            <span class="w-2 h-2 inline-block mr-1 rounded-none" :class="{
                'bg-green-500': service.state === 'running',
                'bg-yellow-500': service.state === 'starting',
                'bg-red-500': service.state === 'stopped' || service.state === 'error'
            }"></span>
            {{ service.state }}
        </p>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-4 mb-4">
        <div>
            <span class="tech-label">PID</span>
            <span class="font-mono text-sm">{{ service.pid || '---' }}</span>
        </div>
        <div>
            <span class="tech-label">HEALTH</span>
            <span class="font-mono text-sm" :class="health === 'ok' ? 'text-[var(--success-color)]' : 'text-[var(--error-color)]'">
                {{ health || "UNKNOWN" }}
            </span>
        </div>
    </div>

    <p v-if="service.lastError" class="error mb-4 font-mono text-xs p-2 border-l-4 border-[var(--error-color)] bg-[var(--code-bg)]">
      ERR: {{ service.lastError }}
    </p>

    <div class="actions grid grid-cols-3 gap-2 mb-4">
      <button
        class="btn col-span-1"
        :class="{ 'bg-[var(--success-color)] text-white border-transparent': service.state !== 'running' }"
        :disabled="busy"
        @click="emit('start', service.id)"
      >
        Start
      </button>
      <button
        class="btn col-span-1"
        :disabled="busy"
        @click="emit('stop', service.id)"
      >
        Stop
      </button>
      <button
        class="btn col-span-1"
        :disabled="busy"
        @click="emit('restart', service.id)"
      >
        Rst
      </button>
      <button v-if="isBinaryError" class="btn col-span-3 border-[var(--error-color)] text-[var(--error-color)]" @click="emit('fix-runtime', service.id)">
          ⚠ Fix Runtime
      </button>
    </div>

    <div class="logs bg-[var(--code-bg)] border-2 border-[var(--border-color)] p-2">
      <div class="logs-head flex justify-between items-center mb-2 border-b border-[var(--border-color)] pb-1">
        <p class="logs-title font-mono text-[10px] uppercase font-bold text-[var(--secondary-color)]">STDOUT_STREAM</p>
        <div class="log-actions flex gap-1">
            <button class="px-1 py-0.5 text-[9px] uppercase border border-[var(--border-color)] hover:bg-[var(--accent-color)] hover:text-white" @click="emit('clear-logs', service.id)">CLR</button>
            <button class="px-1 py-0.5 text-[9px] uppercase border border-[var(--border-color)] hover:bg-[var(--accent-color)] hover:text-white" @click="emit('export-logs', service.id)">EXP</button>
        </div>
      </div>
      <p class="logs-path font-mono text-[9px] text-[var(--secondary-color)] mb-1 truncate" v-if="logPath">PATH: {{ logPath }}</p>
      <ul class="font-mono text-[10px] h-24 overflow-y-auto custom-scrollbar">
        <li v-for="(entry, index) in filteredLogs" :key="index" class="mb-0.5 whitespace-pre-wrap leading-tight">
          <span class="text-[var(--secondary-color)] mr-1">{{ formatTs(entry.ts) }}</span>
          <span :class="entry.level === 'error' ? 'text-[var(--error-color)]' : 'text-[var(--success-color)]'" class="font-bold mr-1">[{{ entry.level }}]</span>
          <span class="text-[var(--text-color)]">{{ entry.message }}</span>
        </li>
        <li v-if="filteredLogs.length === 0" class="text-[var(--secondary-color)] opacity-50 italic">// No output</li>
      </ul>
    </div>
  </article>
</template>

<style scoped>
/* Scoped styles replaced by Tailwind classes in template */
</style>
