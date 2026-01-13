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

function isInstallerRunning() {
  return !!(props.installerStatus && props.installerStatus.phase !== "idle" && props.installerStatus.phase !== "complete");
}

function onEnsureRuntime() {
    emit("ensure-runtime", props.runtimeService, props.runtimeVersion);
}

</script>

<template>
  <div>
    <!-- Update Status -->
    <section v-if="updateStatus" class="notice">
      Update: {{
        updateStatus.available
          ? `available ${updateStatus.version}`
          : "up to date"
      }}
    </section>

    <!-- Update Progress -->
    <section class="notice" v-if="updateProgress">
      Update progress: {{ updateProgress.phase }} ({{
        Math.round(updateProgress.progress * 100)
      }}%)
      <button class="ghost" :disabled="isUpdateRunning()" @click="emit('apply-update')">Apply Update</button>
      <div class="progress">
        <div class="progress-bar" :style="{ width: `${updateProgress.progress * 100}%` }"></div>
      </div>
    </section>

    <!-- Installer Status -->
    <section class="notice" v-if="installerStatus">
      Installer: {{ installerStatus.phase }} ({{ Math.round(installerStatus.progress * 100) }}%)
      <button class="ghost" :disabled="isInstallerRunning()" @click="emit('start-installer')">
        Run Installer
      </button>
      <div class="progress">
        <div class="progress-bar" :style="{ width: `${installerStatus.progress * 100}%` }"></div>
      </div>
    </section>

    <!-- Metrics -->
    <section class="notice" v-if="metrics">
      <h3>Metrics</h3>
      <div class="metrics-grid">
        <div>
          <strong>Uptime</strong>
          <p>{{ metrics.uptimeSec }}s</p>
        </div>
        <div>
          <strong>CPU</strong>
          <p>{{ metrics.cpuPercent.toFixed(1) }}%</p>
        </div>
        <div>
          <strong>Memory</strong>
          <p>{{ metrics.memMB }} MB</p>
        </div>
        <div>
          <strong>Ports</strong>
          <p>{{ metrics.portsInUse.join(", ") || "none" }}</p>
        </div>
      </div>
    </section>

    <!-- Runtime Management -->
    <section class="notice" id="runtime-section">
      <h3>Runtime</h3>
      <div class="project-form">
        <select :value="runtimeService" @change="emit('update:runtimeService', ($event.target as HTMLSelectElement).value)">
          <option value="php">php</option>
          <option value="node">node</option>
          <option value="postgres">postgres</option>
          <option value="mariadb">mariadb</option>
          <option value="mailpit">mailpit</option>
        </select>
        
        <div class="version-input-group">
            <input :value="runtimeVersion" @input="emit('update:runtimeVersion', ($event.target as HTMLInputElement).value)" placeholder="version" list="version-list" />
            <datalist id="version-list">
                <option v-for="v in availableVersions" :key="v" :value="v" />
            </datalist>
        </div>

        <div class="actions">
            <button class="ghost" @click="onEnsureRuntime">Ensure Service</button>
            <button class="ghost" @click="emit('refresh-runtime')">Refresh Manifest</button>
        </div>
      </div>
      
      <div v-if="runtimeDownloadStatus" class="runtime-progress">
        <div class="progress-label">
          runtime: {{ runtimeDownloadStatus.phase }}
          <span v-if="runtimeDownloadStatus.service">
            ({{ runtimeDownloadStatus.service }})
          </span>
        </div>
        <div class="progress">
          <div class="progress-bar" :style="{ width: `${runtimeDownloadStatus.progress * 100}%` }"></div>
        </div>
        <p v-if="runtimeDownloadStatus.error" class="error-inline">{{ runtimeDownloadStatus.error }}</p>
      </div>

      <details class="service-json" v-if="runtimeManifest">
        <summary>Show Manifest</summary>
        <pre>{{ JSON.stringify(runtimeManifest, null, 2) }}</pre>
      </details>
    </section>
  </div>
</template>

<style scoped>
.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.progress {
  height: 6px;
  background: #d8ead8;
  margin-top: 8px;
  border: 1px solid #6fb56f;
}

.progress-bar {
  height: 100%;
  background: #3d8b3d;
  transition: width 0.2s ease;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

.metrics-grid p {
  margin: 6px 0 0;
  font-size: 13px;
}

.project-form {
    display: grid;
    gap: 6px;
}

.actions {
    display: flex;
    gap: 8px;
}

input, select {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
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

.ghost { background: #ffffff; }

.service-json {
    margin-top: 12px;
}

.service-json pre {
  background: #f3f3f3;
  border: 1px solid #d0d0d0;
  padding: 8px;
  overflow: auto;
  max-height: 160px;
  font-size: 12px;
}

.progress-label {
  font-size: 11px;
  margin: 8px 0 4px;
  text-transform: uppercase;
}

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
}

.runtime-progress {
    margin-top: 12px;
}
</style>
