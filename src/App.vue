<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

type ServiceState = {
  id: string;
  state: string;
  pid: number | null;
  lastError: string | null;
  lastUpdated: string;
};
type LogEntry = {
  ts: string;
  level: string;
  message: string;
};
type InstallerStatus = {
  phase: string;
  progress: number;
};
type AppConfig = {
  schemaVersion: number;
  installPath: string;
  updateChannel: string;
  telemetryOptIn: boolean;
  updateFeedUrl: string;
  updatePublicKeys: string[];
};
type ProjectConfig = {
  schemaVersion: number;
  id: string;
  name: string;
  path: string;
  domain: string;
  stack: string;
};
type ServiceConfig = {
  schemaVersion: number;
  id: string;
  enabled: boolean;
  ports: Record<string, number>;
  env: Record<string, string>;
  args: string[];
};

const services = ref<ServiceState[]>([]);
const busy = ref<string | null>(null);
const errorMsg = ref<string | null>(null);
const logsByService = ref<Record<string, LogEntry[]>>({});
const healthByService = ref<Record<string, string>>({});
const logFilter = ref<"all" | "error">("all");
const diagPath = ref<string | null>(null);
const updateStatus = ref<{ available: boolean; version: string } | null>(null);
const installerStatus = ref<InstallerStatus | null>(null);
const updateProgress = ref<{ phase: string; progress: number } | null>(null);
const logPathByService = ref<Record<string, string>>({});
const serviceConfigError = ref<string | null>(null);
const appConfig = ref<AppConfig | null>(null);
const configError = ref<string | null>(null);
const projects = ref<ProjectConfig[]>([]);
const newProject = ref<ProjectConfig>({
  schemaVersion: 1,
  id: "",
  name: "",
  path: "",
  domain: "",
  stack: "php",
});
const serviceConfigs = ref<ServiceConfig[]>([]);
const envDraft = ref<Record<string, { key: string; value: string }[]>>({});
const argsDraft = ref<Record<string, string>>({});
const serviceDirty = ref<Record<string, boolean>>({});
const applyWithoutRestart = ref<Record<string, boolean>>({});
let refreshTimer: number | null = null;
const isUpdateRunning = () =>
  updateProgress.value && updateProgress.value.phase !== "idle" && updateProgress.value.phase !== "complete";
const isInstallerRunning = () =>
  installerStatus.value && installerStatus.value.phase !== "idle" && installerStatus.value.phase !== "complete";

function formatTs(ts: string) {
  const value = Number(ts);
  if (!Number.isFinite(value)) return ts;
  return new Date(value * 1000).toLocaleTimeString();
}

async function loadServices() {
  try {
    errorMsg.value = null;
    services.value = await invoke("services_list");
    await loadServiceConfigs();
    await loadLogs();
    await loadLogPaths();
    await loadHealth();
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function loadLogs() {
  const results = await Promise.all(
    services.value.map(async (service) => {
      const entries: LogEntry[] = await invoke("services_logs", {
        id: service.id,
        tail: 20,
      });
      return [service.id, entries] as const;
    }),
  );
  logsByService.value = Object.fromEntries(results);
}

async function loadLogPaths() {
  const results = await Promise.all(
    services.value.map(async (service) => {
      const path: string = await invoke("services_log_path", { id: service.id });
      return [service.id, path] as const;
    }),
  );
  logPathByService.value = Object.fromEntries(results);
}

async function loadAppConfig() {
  try {
    configError.value = null;
    appConfig.value = await invoke("config_get_app");
  } catch (error) {
    configError.value = String(error);
  }
}

const healthSummary = ref<Record<string, string>>({});

async function loadHealthSummary() {
  try {
    healthSummary.value = await invoke("health_summary");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function saveAppConfig() {
  if (!appConfig.value) return;
  try {
    configError.value = null;
    await invoke("config_set_app", { app: appConfig.value });
  } catch (error) {
    configError.value = String(error);
  }
}

async function loadProjects() {
  try {
    projects.value = await invoke("projects_list");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function saveProject() {
  if (!newProject.value.id) {
    errorMsg.value = "project id is required";
    return;
  }
  try {
    await invoke("projects_save", { project: newProject.value });
    await loadProjects();
    newProject.value = {
      schemaVersion: 1,
      id: "",
      name: "",
      path: "",
      domain: "",
      stack: "php",
    };
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function loadServiceConfigs() {
  const results = await Promise.all(
    services.value.map(async (service) => {
      try {
        const config: ServiceConfig = await invoke("config_get_service", { id: service.id });
        return config;
      } catch {
        return {
          id: service.id,
          enabled: true,
          ports: { main: 0 },
          env: {},
          args: [],
        };
      }
    }),
  );
  serviceConfigs.value = results;
  envDraft.value = Object.fromEntries(
    results.map((config) => [
      config.id,
      Object.entries(config.env).map(([key, value]) => ({ key, value })),
    ]),
  );
  argsDraft.value = Object.fromEntries(
    results.map((config) => [config.id, config.args.join(" ")]),
  );
  serviceDirty.value = Object.fromEntries(results.map((config) => [config.id, false]));
  applyWithoutRestart.value = Object.fromEntries(results.map((config) => [config.id, false]));
}

function findServiceState(id: string) {
  return services.value.find((service) => service.id === id);
}

function detectPortConflict(currentId: string, port: number) {
  if (!port) return null;
  for (const config of serviceConfigs.value) {
    if (config.id !== currentId && config.ports.main === port) {
      return config.id;
    }
  }
  return null;
}

async function saveServiceConfig(config: ServiceConfig) {
  try {
    serviceConfigError.value = null;
    const envEntries = (envDraft.value[config.id] || []).filter((entry) => entry.key.trim());
    for (const entry of envEntries) {
      if (!isValidEnvKey(entry.key)) {
        serviceConfigError.value = `invalid env key: ${entry.key}`;
        return;
      }
    }
    const envError = validateEnvEntries(envEntries);
    if (envError) {
      serviceConfigError.value = envError;
      return;
    }
    config.env = Object.fromEntries(envEntries);
    config.args = (argsDraft.value[config.id] || "")
      .split(" ")
      .map((arg) => arg.trim())
      .filter(Boolean);
    const mainPort = config.ports.main;
    if (mainPort < 0 || mainPort > 65535) {
      serviceConfigError.value = "port out of range";
      return;
    }
    const conflict = detectPortConflict(config.id, mainPort);
    if (conflict) {
      serviceConfigError.value = `port conflict with ${conflict}`;
      return;
    }
    await invoke("config_set_service", { service: config });
    serviceDirty.value[config.id] = false;
    toast.value = { message: `Saved config for ${config.id}`, kind: "info" };
  } catch (error) {
    serviceConfigError.value = String(error);
    toast.value = { message: serviceConfigError.value || "Save failed", kind: "error" };
  }
}

const toast = ref<{ message: string; kind: "info" | "error" } | null>(null);

async function applyServiceConfig(config: ServiceConfig) {
  await saveServiceConfig(config);
  if (applyWithoutRestart.value[config.id]) {
    toast.value = { message: `Applied config for ${config.id}`, kind: "info" };
    return;
  }
  await invoke("services_apply_config", { id: config.id });
  toast.value = { message: `Applied config and restarted ${config.id}`, kind: "info" };
}

async function resetServiceConfig(id: string) {
  try {
    serviceConfigError.value = null;
    if (!confirm(`Reset config for ${id}?`)) {
      return;
    }
    const config: ServiceConfig = await invoke("config_reset_service", { id });
    const index = serviceConfigs.value.findIndex((item) => item.id === id);
    if (index >= 0) {
      serviceConfigs.value[index] = config;
    }
    await loadServiceConfigs();
    toast.value = { message: `Reset config for ${id}`, kind: "info" };
  } catch (error) {
    serviceConfigError.value = String(error);
    toast.value = { message: serviceConfigError.value || "Reset failed", kind: "error" };
  }
}

async function saveAllServiceConfigs() {
  for (const config of serviceConfigs.value) {
    if (serviceDirty.value[config.id]) {
      await saveServiceConfig(config);
    }
  }
  toast.value = { message: "Saved all service configs", kind: "info" };
}

function isValidEnvKey(key: string) {
  return /^[A-Z_][A-Z0-9_]*$/.test(key);
}

function markDirty(id: string) {
  serviceDirty.value[id] = true;
}

function validateEnvEntries(entries: { key: string; value: string }[]) {
  const seen = new Set<string>();
  for (const entry of entries) {
    if (!entry.key.trim()) continue;
    if (seen.has(entry.key)) {
      return `duplicate env key: ${entry.key}`;
    }
    if (!entry.value.trim()) {
      return `env value required for ${entry.key}`;
    }
    seen.add(entry.key);
  }
  return null;
}

function addEnvRow(id: string) {
  const rows = envDraft.value[id] || [];
  rows.push({ key: "", value: "" });
  envDraft.value[id] = rows;
  serviceDirty.value[id] = true;
}

async function loadHealth() {
  const results = await Promise.all(
    services.value.map(async (service) => {
      try {
        const result: string = await invoke("services_health", { id: service.id });
        return [service.id, result] as const;
      } catch (error) {
        return [service.id, "error"] as const;
      }
    }),
  );
  healthByService.value = Object.fromEntries(results);
}

async function startService(id: string) {
  try {
    busy.value = id;
    errorMsg.value = null;
    await invoke("services_start", { id });
    await loadServices();
  } catch (error) {
    errorMsg.value = String(error);
  } finally {
    busy.value = null;
  }
}

async function stopService(id: string) {
  try {
    busy.value = id;
    errorMsg.value = null;
    await invoke("services_stop", { id });
    await loadServices();
  } catch (error) {
    errorMsg.value = String(error);
  } finally {
    busy.value = null;
  }
}

async function restartService(id: string) {
  try {
    busy.value = id;
    errorMsg.value = null;
    await invoke("services_restart", { id });
    await loadServices();
  } catch (error) {
    errorMsg.value = String(error);
  } finally {
    busy.value = null;
  }
}

async function exportDiagnostics() {
  try {
    errorMsg.value = null;
    const path: string = await invoke("diagnostics_create");
    diagPath.value = path;
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function checkUpdates() {
  try {
    errorMsg.value = null;
    updateStatus.value = await invoke("updater_check");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function applyUpdate() {
  try {
    errorMsg.value = null;
    await invoke("updater_apply");
    await loadUpdateProgress();
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function loadUpdateProgress() {
  try {
    updateProgress.value = await invoke("updater_progress");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function loadInstallerStatus() {
  try {
    installerStatus.value = await invoke("installer_status");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function startInstaller() {
  try {
    errorMsg.value = null;
    await invoke("installer_start");
    await loadInstallerStatus();
  } catch (error) {
    errorMsg.value = String(error);
  }
}

onMounted(loadServices);
onMounted(loadInstallerStatus);
onMounted(loadUpdateProgress);
onMounted(loadAppConfig);
onMounted(loadProjects);
onMounted(loadHealthSummary);
onMounted(() => {
  refreshTimer = window.setInterval(async () => {
    await loadServices();
    await loadInstallerStatus();
    await loadUpdateProgress();
    await loadAppConfig();
    await loadProjects();
    await loadHealthSummary();
    toast.value = null;
  }, 5000);
});
onUnmounted(() => {
  if (refreshTimer) {
    window.clearInterval(refreshTimer);
    refreshTimer = null;
  }
});
</script>

<template>
  <main class="app">
    <header class="header">
      <div>
        <h1>Kojibox</h1>
        <p class="subtitle">Portable dev stack manager</p>
      </div>
      <div class="header-actions">
        <div class="filter">
          <button
            class="ghost"
            :data-active="logFilter === 'all'"
            @click="logFilter = 'all'"
          >
            All Logs
          </button>
          <button
            class="ghost"
            :data-active="logFilter === 'error'"
            @click="logFilter = 'error'"
          >
            Errors
          </button>
        </div>
        <button class="ghost" @click="loadServices">Refresh</button>
        <button class="ghost" @click="exportDiagnostics">Export Diagnostics</button>
        <button class="ghost" @click="checkUpdates">Check Updates</button>
      </div>
    </header>

    <section v-if="toast" class="notice" :data-kind="toast.kind">
      {{ toast.message }}
    </section>

    <section v-if="diagPath" class="notice">
      Diagnostics saved to: {{ diagPath }}
    </section>

    <section class="notice" v-if="appConfig">
      <div class="config">
        <div>
          <label>Install Path</label>
          <input v-model="appConfig.installPath" />
        </div>
        <div>
          <label>Schema Version</label>
          <input v-model.number="appConfig.schemaVersion" disabled />
        </div>
        <div>
          <label>Update Channel</label>
          <select v-model="appConfig.updateChannel">
            <option value="stable">stable</option>
            <option value="beta">beta</option>
          </select>
        </div>
        <div>
          <label>Update Feed URL</label>
          <input v-model="appConfig.updateFeedUrl" />
        </div>
        <div class="actions-inline">
          <button class="ghost" @click="saveAppConfig">Save Config</button>
        </div>
      </div>
      <p v-if="configError" class="error-inline">{{ configError }}</p>
    </section>

    <section class="notice">
      <h3>Projects</h3>
      <div class="project-form">
        <input v-model="newProject.id" placeholder="id" />
        <input v-model="newProject.name" placeholder="name" />
        <input v-model="newProject.path" placeholder="path" />
        <input v-model="newProject.domain" placeholder="domain" />
        <select v-model="newProject.stack">
          <option value="php">php</option>
          <option value="node">node</option>
        </select>
        <button class="ghost" @click="saveProject">Save Project</button>
      </div>
      <ul class="project-list">
        <li v-for="project in projects" :key="project.id">
          <strong>{{ project.name }}</strong> ({{ project.stack }}) - {{ project.domain }}
        </li>
      </ul>
    </section>

    <section class="notice">
      <h3>Service Settings (stub)</h3>
      <div class="actions-inline">
        <button class="ghost" @click="saveAllServiceConfigs">Save All</button>
      </div>
      <div class="project-form" v-if="serviceConfigs.length">
        <div v-for="config in serviceConfigs" :key="config.id">
          <strong>{{ config.id }}</strong>
          <span
            v-if="findServiceState(config.id)"
            class="status"
            :data-state="findServiceState(config.id)?.state"
          >
            {{ findServiceState(config.id)?.state }}
          </span>
          <span v-if="applyWithoutRestart[config.id]" class="hint">no-restart</span>
          <details class="service-json">
            <summary>Show JSON</summary>
            <pre>{{ JSON.stringify(config, null, 2) }}</pre>
          </details>
          <div class="service-config">
            <label>Enabled</label>
            <input type="checkbox" v-model="config.enabled" @change="markDirty(config.id)" />
          </div>
          <div class="service-config">
            <label>Port</label>
            <input v-model.number="config.ports.main" @input="markDirty(config.id)" />
            <span class="hint" v-if="config.ports.main === 0">auto-assign</span>
          </div>
          <div class="service-config">
            <label>Args</label>
            <input
              v-model="argsDraft[config.id]"
              placeholder="--flag --value"
              @input="markDirty(config.id)"
            />
          </div>
          <div class="service-config">
            <label>Env</label>
            <div class="env-list">
              <div v-for="(row, idx) in envDraft[config.id] || []" :key="idx" class="env-row">
                <input v-model="row.key" placeholder="KEY" @input="markDirty(config.id)" />
                <input v-model="row.value" placeholder="VALUE" @input="markDirty(config.id)" />
              </div>
              <button class="ghost" @click="addEnvRow(config.id)">Add Env</button>
            </div>
          </div>
          <div class="service-config">
            <label>Apply Without Restart</label>
            <input type="checkbox" v-model="applyWithoutRestart[config.id]" />
          </div>
          <div class="service-config">
            <button class="ghost" @click="saveServiceConfig(config)">Save</button>
            <button class="primary" @click="applyServiceConfig(config)">Apply & Restart</button>
            <button class="ghost" @click="resetServiceConfig(config.id)">Reset</button>
            <span v-if="serviceDirty[config.id]" class="dirty">unsaved</span>
          </div>
        </div>
      </div>
      <p v-if="serviceConfigError" class="error-inline">{{ serviceConfigError }}</p>
    </section>

    <section v-if="updateStatus" class="notice">
      Update: {{
        updateStatus.available
          ? `available ${updateStatus.version}`
          : "up to date"
      }}
    </section>

    <section class="notice">
      <h3>Health Summary</h3>
      <ul class="project-list">
        <li v-for="(value, key) in healthSummary" :key="key">
          <strong>{{ key }}</strong>: {{ value }}
        </li>
      </ul>
    </section>

    <section class="notice" v-if="updateProgress">
      Update progress: {{ updateProgress.phase }} ({{
        Math.round(updateProgress.progress * 100)
      }}%)
      <button class="ghost" :disabled="isUpdateRunning()" @click="applyUpdate">Apply Update</button>
      <div class="progress">
        <div class="progress-bar" :style="{ width: `${updateProgress.progress * 100}%` }"></div>
      </div>
    </section>

    <section class="notice" v-if="installerStatus">
      Installer: {{ installerStatus.phase }} ({{ Math.round(installerStatus.progress * 100) }}%)
      <button class="ghost" :disabled="isInstallerRunning()" @click="startInstaller">
        Run Installer
      </button>
      <div class="progress">
        <div class="progress-bar" :style="{ width: `${installerStatus.progress * 100}%` }"></div>
      </div>
    </section>

    <section v-if="errorMsg" class="error">
      {{ errorMsg }}
    </section>

    <section class="grid">
      <article v-for="service in services" :key="service.id" class="card">
        <div class="card-head">
          <div>
            <h2>{{ service.id }}</h2>
            <p class="status" :data-state="service.state">{{ service.state }}</p>
          </div>
          <span class="pid" v-if="service.pid">pid {{ service.pid }}</span>
        </div>
        <p class="health" :data-health="healthByService[service.id]">
          health: {{ healthByService[service.id] || "unknown" }}
        </p>
        <p v-if="service.lastError" class="error-inline">
          {{ service.lastError }}
        </p>
        <div class="actions">
          <button
            class="primary"
            :disabled="busy === service.id"
            @click="startService(service.id)"
          >
            Start
          </button>
          <button
            class="secondary"
            :disabled="busy === service.id"
            @click="stopService(service.id)"
          >
            Stop
          </button>
          <button
            class="ghost"
            :disabled="busy === service.id"
            @click="restartService(service.id)"
          >
            Restart
          </button>
        </div>
        <div class="logs">
          <p class="logs-title">Recent logs</p>
          <p class="logs-path" v-if="logPathByService[service.id]">
            File: {{ logPathByService[service.id] }}
          </p>
          <ul>
            <li
              v-for="(entry, index) in (logsByService[service.id] || []).filter((item) =>
                logFilter === 'all' ? true : item.level === 'error',
              )"
              :key="index"
            >
              <span class="log-ts">{{ formatTs(entry.ts) }}</span>
              <span class="log-level" :data-level="entry.level">{{ entry.level }}</span>
              <span class="log-message">{{ entry.message }}</span>
            </li>
          </ul>
        </div>
      </article>
    </section>
  </main>
</template>

<style>
:root {
  font-family: "Fira Sans", "Trebuchet MS", sans-serif;
  font-size: 16px;
  color: #1b1b1b;
  background: #f1f0ea;
  text-rendering: optimizeLegibility;
}

body {
  margin: 0;
}

.app {
  padding: 32px;
}

.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 2px solid #1b1b1b;
  padding-bottom: 16px;
  margin-bottom: 24px;
}

.subtitle {
  margin: 4px 0 0;
  color: #5b5b5b;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter {
  display: flex;
  gap: 8px;
}

.filter button[data-active="true"] {
  background: #1b1b1b;
  color: #ffffff;
}

.error {
  background: #fbe3e3;
  border: 1px solid #d96a6a;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.notice[data-kind="error"] {
  background: #fbe3e3;
  border-color: #d96a6a;
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

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
}

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

input,
select {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
}

.config {
  display: grid;
  gap: 8px;
}

.config label {
  display: block;
  font-size: 12px;
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.actions-inline {
  margin-top: 8px;
}

.project-form {
  display: grid;
  gap: 6px;
  margin-top: 8px;
}

.service-config {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 0;
}

.hint {
  font-size: 11px;
  text-transform: uppercase;
  color: #6b6b6b;
}

.service-json {
  margin: 6px 0;
}

.service-json pre {
  background: #f3f3f3;
  border: 1px solid #d0d0d0;
  padding: 8px;
  overflow: auto;
  max-height: 160px;
  font-size: 12px;
}

.env-list {
  display: grid;
  gap: 6px;
}

.env-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.dirty {
  font-size: 12px;
  text-transform: uppercase;
  color: #c17b1b;
}

.project-list {
  margin: 8px 0 0;
  padding-left: 16px;
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

.logs-title {
  margin: 0 0 6px;
  text-transform: uppercase;
  font-weight: 700;
  font-size: 10px;
  letter-spacing: 0.08em;
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

.log-message {
  color: #333;
}
@media (max-width: 720px) {
  .app {
    padding: 20px;
  }
}
</style>
