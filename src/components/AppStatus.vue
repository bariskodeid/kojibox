<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { 
    InstallerStatus, 
    MetricsSnapshot, 
    RuntimeManifest, 
    RuntimeDownloadStatus 
} from "../types";

const props = defineProps<{
  installerStatus: InstallerStatus | null;
  updateStatus: { available: boolean; version: string } | null;
  updateProgress: { phase: string; progress: number } | null;
  metrics: MetricsSnapshot | null;
  runtimeManifest: RuntimeManifest | null;
  runtimeDownloadStatus: RuntimeDownloadStatus | null;
  runtimeService: string;
  runtimeVersion: string;
}>();

const emit = defineEmits<{
  (e: "apply-update"): void;
  (e: "start-installer"): void;
  (e: "ensure-runtime", service: string, version: string): void;
  (e: "refresh-runtime"): void;
  (e: "update:runtimeService", value: string): void;
  (e: "update:runtimeVersion", value: string): void;
}>();

const availableVersions = ref<string[]>([]);

watch(() => props.runtimeService, async (newService) => {
    if (newService) {
        try {
            const versions: string[] = await invoke("runtime_list_versions", { service: newService });
            availableVersions.value = versions;
        } catch (e) {
            console.error(e);
            availableVersions.value = [];
        }
    }
}, { immediate: true });

function isUpdateRunning() {
  return !!(props.updateProgress && props.updateProgress.phase !== "idle" && props.updateProgress.phase !== "complete");
}

function onEnsureRuntime() {
    emit("ensure-runtime", props.runtimeService, props.runtimeVersion);
}

</script>

<template>
  <div class="space-y-4">
    <!-- Runtime Management -->
    <section class="card" id="runtime-section">
      <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
          <h3 class="text-lg font-black uppercase">Runtime Control</h3>
          <span class="tech-label">SYS_RT</span>
      </div>
      
      <div class="grid grid-cols-1 gap-4">
        <div>
            <span class="tech-label mb-1">SERVICE_SELECT</span>
            <select :value="runtimeService" @change="emit('update:runtimeService', ($event.target as HTMLSelectElement).value)">
              <option value="php">PHP</option>
              <option value="node">NODE</option>
              <option value="postgres">POSTGRES</option>
              <option value="mariadb">MARIADB</option>
              <option value="mailpit">MAILPIT</option>
            </select>
        </div>
        
        <div>
            <span class="tech-label mb-1">TARGET_VERSION</span>
            <div class="flex gap-2">
                <input class="input font-mono" :value="runtimeVersion" @input="emit('update:runtimeVersion', ($event.target as HTMLInputElement).value)" placeholder="ex: 8.3.2" list="version-list" />
                <datalist id="version-list">
                    <option v-for="v in availableVersions" :key="v" :value="v" />
                </datalist>
            </div>
        </div>

        <div class="grid grid-cols-2 gap-2 mt-2">
            <button class="btn" @click="onEnsureRuntime">Ensure</button>
            <button class="btn" @click="emit('refresh-runtime')">Sync Manifest</button>
        </div>
      </div>
      
      <div v-if="runtimeDownloadStatus" class="mt-4 p-2 border border-[var(--border-color)] bg-[var(--code-bg)]">
        <div class="flex justify-between text-xs font-mono mb-1">
          <span class="uppercase font-bold">{{ runtimeDownloadStatus.phase }}</span>
          <span v-if="runtimeDownloadStatus.service">[{{ runtimeDownloadStatus.service }}]</span>
        </div>
        <div class="h-2 w-full bg-[var(--card-bg)] border border-[var(--border-color)]">
          <div class="h-full bg-[var(--accent-color)] transition-all duration-200" :style="{ width: `${runtimeDownloadStatus.progress * 100}%` }"></div>
        </div>
        <p v-if="runtimeDownloadStatus.error" class="text-[var(--error-color)] text-xs font-mono mt-1">> ERROR: {{ runtimeDownloadStatus.error }}</p>
      </div>
    </section>

    <!-- Metrics -->
    <section class="card" v-if="metrics">
      <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
          <h3 class="text-lg font-black uppercase">Telemetry</h3>
          <span class="tech-label">METRICS</span>
      </div>
      <div class="grid grid-cols-2 gap-4">
        <div>
          <span class="tech-label">UPTIME</span>
          <p class="font-mono text-xl font-bold">{{ metrics.uptimeSec }}s</p>
        </div>
        <div>
          <span class="tech-label">LOAD</span>
          <p class="font-mono text-xl font-bold">{{ metrics.cpuPercent.toFixed(1) }}%</p>
        </div>
        <div>
          <span class="tech-label">MEM</span>
          <p class="font-mono text-xl font-bold">{{ metrics.memMB }} MB</p>
        </div>
        <div>
          <span class="tech-label">PORTS</span>
          <p class="font-mono text-xs truncate" :title="metrics.portsInUse.join(', ')">{{ metrics.portsInUse.length }} active</p>
        </div>
      </div>
    </section>

    <!-- Update Status -->
    <section v-if="updateStatus && updateStatus.available" class="notice" data-kind="info">
        <div class="flex justify-between items-center">
            <span class="font-bold uppercase">Update Available</span>
            <span class="font-mono bg-[var(--accent-color)] text-white px-2 py-0.5 text-xs">{{ updateStatus.version }}</span>
        </div>
        <div v-if="updateProgress" class="mt-2">
             <div class="text-xs font-mono mb-1 uppercase">{{ updateProgress.phase }}</div>
             <div class="h-1 w-full bg-white/50"><div class="h-full bg-black" :style="{ width: `${updateProgress.progress * 100}%` }"></div></div>
        </div>
        <button v-else class="btn btn-sm mt-2 w-full" :disabled="isUpdateRunning()" @click="emit('apply-update')">INSTALL UPDATE</button>
    </section>
  </div>
</template>

<style scoped>
/* Styles replaced by global classes */
</style>
