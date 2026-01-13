<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type {
  ServiceState,
  LogEntry,
  InstallerStatus,
  MetricsSnapshot,
  AppConfig,
  ProjectConfig,
  ServiceConfig,
  DomainMapping,
  ProxyRule,
  CertMeta,
  TrustResult,
  RuntimeManifest,
  RuntimeDownloadStatus,
  ServiceBinary,
} from "./types";

import SetupWizard from "./components/SetupWizard.vue";
import AppHeader from "./components/AppHeader.vue";
import ServiceCard from "./components/ServiceCard.vue";
import LogViewer from "./components/LogViewer.vue";
import ProjectManager from "./components/ProjectManager.vue";
import ServiceSettings from "./components/ServiceSettings.vue";
import ToolingManager from "./components/ToolingManager.vue";
import AppStatus from "./components/AppStatus.vue";
import GlobalConfig from "./components/GlobalConfig.vue";
import PhpManager from "./components/PhpManager.vue";
import DatabaseInfo from "./components/DatabaseInfo.vue";
import TaskManager from "./components/TaskManager.vue";
import ToastContainer, { type Toast } from "./components/ToastContainer.vue";
import AboutModal from "./components/AboutModal.vue";

// State
const services = ref<ServiceState[]>([]);
const busy = ref<string | null>(null);
const errorMsg = ref<string | null>(null);
const logsByService = ref<Record<string, LogEntry[]>>({});
const healthByService = ref<Record<string, string>>({});
const logFilter = ref<"all" | "error">("all");
const diagPath = ref<string | null>(null);
const showAbout = ref(false);

const updateStatus = ref<{ available: boolean; version: string } | null>(null);
const installerStatus = ref<InstallerStatus | null>(null);
const updateProgress = ref<{ phase: string; progress: number } | null>(null);

const logPathByService = ref<Record<string, string>>({});
const appConfig = ref<AppConfig | null>(null);
const configError = ref<string | null>(null);
const metricsSnapshot = ref<MetricsSnapshot | null>(null);
const needsSetup = ref(false);

const projects = ref<ProjectConfig[]>([]);
const serviceConfigs = ref<ServiceConfig[]>([]);

const domainMappings = ref<DomainMapping[]>([]);
const proxyRules = ref<ProxyRule[]>([]);
const certs = ref<CertMeta[]>([]);
const trustResult = ref<TrustResult | null>(null);

const runtimeManifest = ref<RuntimeManifest | null>(null);
const runtimeDownloadStatus = ref<RuntimeDownloadStatus | null>(null);
const runtimeService = ref("php");
const runtimeVersion = ref("8.3.2");

const toasts = ref<Toast[]>([]);
let toastIdCounter = 0;
let refreshTimer: number | null = null;

const mailpitUrl = computed(() => {
  const config = serviceConfigs.value.find((service) => service.id === "mailpit");
  if (!config || !config.enabled) return null;
  const port = config.ports.main || 8025;
  return `http://127.0.0.1:${port}`;
});

function addToast(message: string, kind: "info" | "error" | "success" = "info") {
  const id = ++toastIdCounter;
  toasts.value.push({ id, message, kind });
  setTimeout(() => removeToast(id), 5000);
}

function removeToast(id: number) {
  toasts.value = toasts.value.filter(t => t.id !== id);
}

// Keep simpler alias for existing code
const toast = computed({
    get: () => toasts.value.length ? { message: toasts.value[toasts.value.length - 1].message, kind: toasts.value[toasts.value.length - 1].kind } : null,
    set: (val) => { if (val) addToast(val.message, val.kind) }
});

// Actions

async function openTerminal() {
    try {
        await invoke("open_terminal");
    } catch (e) {
        addToast(String(e), "error");
    }
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

async function checkFirstRun() {
  try {
    const exists: boolean = await invoke("config_app_exists");
    needsSetup.value = !exists;
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function saveAppConfig(newConfig: AppConfig) {
  try {
    configError.value = null;
    await invoke("config_set_app", { app: newConfig });
    await loadAppConfig();
    toast.value = { message: "App config saved", kind: "info" };
  } catch (error) {
    configError.value = String(error);
    toast.value = { message: "Failed to save app config", kind: "error" };
  }
}

async function loadProjects() {
  try {
    projects.value = await invoke("projects_list");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function saveProject(project: ProjectConfig) {
  try {
    await invoke("projects_save", { project });
    await loadProjects();
    toast.value = { message: `Saved project ${project.name}`, kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function deleteProject(id: string) {
  try {
    await invoke("projects_delete", { id });
    await loadProjects();
    toast.value = { message: `Deleted project ${id}`, kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
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
          schemaVersion: 1,
          id: service.id,
          enabled: true,
          ports: { main: 0 },
          env: {},
          args: [],
        };
      }
    }),
  );
  // We don't overwrite user drafts here, the component handles that via watcher
  serviceConfigs.value = results;
}

async function saveServiceConfig(config: ServiceConfig) {
  try {
    await invoke("config_set_service", { service: config });
    toast.value = { message: `Saved config for ${config.id}`, kind: "info" };
    // Force reload to sync
    await loadServiceConfigs();
  } catch (error) {
    toast.value = { message: String(error) || "Save failed", kind: "error" };
  }
}

async function applyServiceConfig(config: ServiceConfig, restart: boolean) {
  await saveServiceConfig(config);
  if (!restart) {
    await invoke("services_apply_config_no_restart", { id: config.id });
    toast.value = { message: `Applied config for ${config.id}`, kind: "info" };
    return;
  }
  await invoke("services_apply_config", { id: config.id });
  toast.value = { message: `Applied config and restarted ${config.id}`, kind: "info" };
  await loadServices();
}

async function resetServiceConfig(id: string) {
  try {
    await invoke("config_reset_service", { id });
    await loadServiceConfigs();
    toast.value = { message: `Reset config for ${id}`, kind: "info" };
  } catch (error) {
    toast.value = { message: String(error) || "Reset failed", kind: "error" };
  }
}

async function openConfig(id: string) {
    // Map service id to config file path relative to root (or absolute)
    // Ideally backend exposes this path. For now hardcode typical paths.
    let path = `runtime/config/${id}`;
    if (id === 'php') path += '/php.ini';
    else if (id === 'mariadb') path += '/my.cnf';
    else if (id === 'postgres') path = 'runtime/data/postgres/postgresql.conf'; // usually in data dir
    
    // We need absolute path for system open
    // Since we don't have easy absolute path resolution here, let's ask backend to open it 
    // by resolving relative to CWD.
    // system_open_file takes path.
    
    // But wait, system_open_file implemented in backend takes String and passes to Command.
    // If we pass relative path, it depends on CWD. App CWD is project root in dev, or install dir in prod.
    // Should be fine.
    
    try {
        await invoke("system_open_file", { path });
    } catch (e) {
        toast.value = { message: "Failed to open config: " + e, kind: "error" };
    }
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

async function exportLogs(id: string) {
  try {
    const path: string = await invoke("logs_export", {
      service: id,
      level: logFilter.value === "error" ? "error" : null,
      limit: 200,
    });
    toast.value = { message: `Exported logs to ${path}`, kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function clearLogs(id: string) {
    if (!confirm(`Clear logs for ${id}?`)) return;
    try {
        await invoke("logs_clear", { service: id });
        toast.value = { message: `Logs cleared for ${id}`, kind: "info" };
        await loadLogs();
    } catch (error) {
        toast.value = { message: String(error), kind: "error" };
    }
}

async function exportViewerLogs(params: { service: string | null; level: string | null; limit: number }) {
  try {
    const path: string = await invoke("logs_export", params);
    toast.value = { message: `Exported logs to ${path}`, kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
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

async function loadMetrics() {
  try {
    metricsSnapshot.value = await invoke("metrics_snapshot");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function loadDomains() {
  try {
    domainMappings.value = await invoke("domains_list");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function saveDomain(mapping: DomainMapping) {
  try {
    await invoke("domains_upsert", { mapping });
    await loadDomains();
    toast.value = { message: "Domain saved", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function deleteDomain(domain: string) {
  try {
    await invoke("domains_remove", { domain });
    await loadDomains();
    toast.value = { message: "Domain removed", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function applyHosts() {
  try {
    await invoke("hosts_apply", { mappings: domainMappings.value });
    toast.value = { message: "Hosts applied", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function rollbackHosts() {
  try {
    await invoke("hosts_rollback");
    toast.value = { message: "Hosts rolled back", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function loadProxyRules() {
  try {
    proxyRules.value = await invoke("proxy_rules");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function saveProxy(rule: ProxyRule) {
  try {
    // Append
    const newRules = [...proxyRules.value, rule];
    await invoke("proxy_apply", { rules: newRules });
    proxyRules.value = newRules;
    toast.value = { message: "Proxy rule added", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function deleteProxy(index: number) {
  try {
    const newRules = proxyRules.value.filter((_, idx) => idx !== index);
    await invoke("proxy_apply", { rules: newRules });
    proxyRules.value = newRules;
    toast.value = { message: "Proxy rule deleted", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function applyProxy() {
  try {
    await invoke("proxy_apply", { rules: proxyRules.value });
    toast.value = { message: "Proxy rules applied", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function loadCerts() {
  try {
    certs.value = await invoke("certs_list");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function generateCert(domains: string[]) {
  try {
    await invoke("certs_generate", { domains });
    await loadCerts();
    toast.value = { message: "Certificate generated", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function trustCert(path: string, os: boolean, apply: boolean) {
  try {
    if (!os) {
        const result: string = await invoke("certs_trust", { certPath: path });
        toast.value = { message: `Trust instructions: ${result}`, kind: "info" };
    } else {
        trustResult.value = await invoke("certs_trust_os", { certPath: path, apply });
        if (trustResult.value?.applied) {
            toast.value = { message: "Trust applied via OS command", kind: "info" };
        }
    }
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function loadRuntimeManifest() {
  try {
    runtimeManifest.value = await invoke("runtime_get_manifest");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function loadRuntimeDownloadStatus() {
  try {
    runtimeDownloadStatus.value = await invoke("runtime_download_status");
  } catch (error) {
    errorMsg.value = String(error);
  }
}

async function refreshRuntimeManifest() {
  try {
    runtimeManifest.value = await invoke("runtime_refresh_manifest");
    toast.value = { message: "Manifest refreshed", kind: "info" };
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function ensureRuntime(name: string, version: string) {
  try {
    const result: ServiceBinary = await invoke("runtime_ensure_service", { name, version });
    toast.value = { message: `Ready: ${result.binPath}`, kind: "info" };
    await loadRuntimeManifest();
  } catch (error) {
    toast.value = { message: String(error), kind: "error" };
  }
}

async function onFixRuntime(id: string) {
    // Scroll to runtime section
    const runtimeSection = document.getElementById('runtime-section');
    if (runtimeSection) {
        runtimeSection.scrollIntoView({ behavior: 'smooth' });
    }
    // Preset runtime form
    runtimeService.value = id;
    // Trigger ensure immediately
    await ensureRuntime(id, runtimeVersion.value);
}

function onSetupComplete() {
  needsSetup.value = false;
  loadAppConfig();
  loadServices();
}

onMounted(() => {
  loadServices();
  loadInstallerStatus();
  loadUpdateProgress();
  loadAppConfig();
  checkFirstRun();
  loadProjects();
  loadHealth();
  loadMetrics();
  loadDomains();
  loadProxyRules();
  loadCerts();
  loadRuntimeManifest();
  loadRuntimeDownloadStatus();
  
  refreshTimer = window.setInterval(async () => {
    // Silent background refresh
    await Promise.all([
        invoke("services_list").then((res) => { services.value = res as ServiceState[] }),
        loadServiceConfigs(), // Updates if not dirty
        loadLogs(),
        loadLogPaths(),
        loadHealth(),
        loadInstallerStatus(),
        loadUpdateProgress(),
        loadProjects(),
        loadMetrics(),
        loadDomains(),
        loadProxyRules(),
        loadCerts(),
        loadRuntimeManifest(),
        loadRuntimeDownloadStatus()
    ]);
    
    // Auto-clear toast after 5s
    if (toast.value) {
        setTimeout(() => { toast.value = null }, 5000);
    }
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
    <SetupWizard :needs-setup="needsSetup" @setup-complete="onSetupComplete" />
    
    <AppHeader 
      v-model:logFilter="logFilter"
      @refresh="loadServices"
      @export-diagnostics="exportDiagnostics"
      @check-updates="checkUpdates"
      @open-terminal="openTerminal"
      @open-about="showAbout = true"
    />

    <AboutModal :show="showAbout" version="0.1.0" @close="showAbout = false" />

    <ToastContainer :toasts="toasts" @remove="removeToast" />

    <section v-if="diagPath" class="notice">
      Diagnostics saved to: {{ diagPath }}
    </section>

    <GlobalConfig 
        :app-config="appConfig" 
        :config-error="configError" 
        @save="saveAppConfig"
    />

    <AppStatus
        :installer-status="installerStatus"
        :update-status="updateStatus"
        :update-progress="updateProgress"
        :metrics="metricsSnapshot"
        :runtime-manifest="runtimeManifest"
        :runtime-download-status="runtimeDownloadStatus"
        v-model:runtime-service="runtimeService"
        v-model:runtime-version="runtimeVersion"
        @apply-update="applyUpdate"
        @start-installer="startInstaller"
        @ensure-runtime="ensureRuntime"
        @refresh-runtime="refreshRuntimeManifest"
    />

    <DatabaseInfo 
        :services="services"
        :configs="serviceConfigs"
    />

    <TaskManager :projects="projects" />

    <ProjectManager 
        :projects="projects"
        @save="saveProject"
        @delete="deleteProject"
    />

    <ServiceSettings 
        :configs="serviceConfigs" 
        :service-states="services"
        @save="saveServiceConfig"
        @apply="applyServiceConfig"
        @reset="resetServiceConfig"
        @edit-config="openConfig"
    />

    <PhpManager @restart-php="restartService('php')" />

    <ToolingManager
        :domains="domainMappings"
        :proxy-rules="proxyRules"
        :certs="certs"
        :trust-result="trustResult"
        @save-domain="saveDomain"
        @delete-domain="deleteDomain"
        @apply-hosts="applyHosts"
        @rollback-hosts="rollbackHosts"
        @save-proxy="saveProxy"
        @delete-proxy="deleteProxy"
        @apply-proxy="applyProxy"
        @generate-cert="generateCert"
        @trust-cert="trustCert"
    />

    <LogViewer 
        :services="services"
        :logs-by-service="logsByService"
        @export="exportViewerLogs"
    />

    <section class="notice" v-if="mailpitUrl">
      <h3>Mailpit</h3>
      <iframe class="mailpit-frame" :src="mailpitUrl" title="Mailpit"></iframe>
    </section>

    <section class="grid">
      <ServiceCard 
        v-for="service in services" 
        :key="service.id"
        :service="service"
        :logs="logsByService[service.id] || []"
        :health="healthByService[service.id] || 'unknown'"
        :log-filter="logFilter"
        :log-path="logPathByService[service.id]"
        :busy="busy === service.id"
        @start="startService"
        @stop="stopService"
        @restart="restartService"
        @export-logs="exportLogs"
        @clear-logs="clearLogs"
        @fix-runtime="onFixRuntime"
      />
    </section>
    
    <section v-if="errorMsg" class="error">
      {{ errorMsg }}
    </section>
  </main>
</template>

<style>
:root {
  --bg-color: #f1f0ea;
  --text-color: #1b1b1b;
  --card-bg: #ffffff;
  --border-color: #1b1b1b;
  --accent-color: #ffd36a;
  --secondary-color: #c7e5ff;
  --ghost-bg: #fefefe;
  --success-color: #0b7a3e;
  --warning-color: #c17b1b;
  --error-color: #b23b3b;
  --code-bg: #f3f3f3;
  --hint-color: #5b5b5b;
  
  font-family: "Fira Sans", "Trebuchet MS", sans-serif;
  font-size: 16px;
  color: var(--text-color);
  background: var(--bg-color);
  text-rendering: optimizeLegibility;
}

.dark {
  --bg-color: #1a1a1a;
  --text-color: #e0e0e0;
  --card-bg: #2a2a2a;
  --border-color: #4a4a4a;
  --accent-color: #ffca28;
  --secondary-color: #3d5afe;
  --ghost-bg: #2a2a2a;
  --success-color: #66bb6a;
  --warning-color: #ffa726;
  --error-color: #ef5350;
  --code-bg: #333333;
  --hint-color: #9e9e9e;
}

body {
  margin: 0;
  background: var(--bg-color);
  color: var(--text-color);
}

.app {
  padding: 32px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
  margin-top: 24px;
}

.notice {
  background: var(--card-bg); /* Use card bg for notices in dark mode or lighter variant? */
  border: 1px solid var(--success-color);
  padding: 12px 16px;
  margin-bottom: 16px;
  color: var(--text-color);
}

/* Override notice specific colors */
:root .notice {
    background: #e8f4e8;
}
.dark .notice {
    background: #1e3320;
    border-color: #2e7d32;
}

.notice[data-kind="error"], .error {
  background: #fbe3e3;
  border: 1px solid var(--error-color);
  padding: 12px 16px;
  margin-bottom: 16px;
}
.dark .notice[data-kind="error"], .dark .error {
    background: #3e2020;
}

.notice[data-kind="info"] {
  border-color: var(--success-color);
}

.mailpit-frame {
  width: 100%;
  min-height: 320px;
  border: 1px solid var(--border-color);
  background: #fff;
}

@media (max-width: 720px) {
  .app {
    padding: 20px;
  }
}

/* Update generic components to use vars */
button {
    color: var(--text-color);
    border-color: var(--border-color);
}
input, select {
    background: var(--card-bg);
    color: var(--text-color);
    border-color: var(--border-color);
}
</style>
