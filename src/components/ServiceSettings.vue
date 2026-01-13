<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ServiceConfig, ServiceState } from "../types";

const props = defineProps<{
  configs: ServiceConfig[];
  serviceStates: ServiceState[];
}>();

const emit = defineEmits<{
  (e: "save", config: ServiceConfig): void;
  (e: "apply", config: ServiceConfig, restart: boolean): void;
  (e: "reset", id: string): void;
  (e: "edit-config", id: string): void;
}>();

const serviceConfigError = ref<string | null>(null);
const envDraft = ref<Record<string, { key: string; value: string }[]>>({});
const argsDraft = ref<Record<string, string>>({});
const serviceDirty = ref<Record<string, boolean>>({});
const applyWithoutRestart = ref<Record<string, boolean>>({});

// Initialize or update drafts based on props, preserving dirty states
watch(
  () => props.configs,
  (newConfigs) => {
    for (const config of newConfigs) {
      if (!serviceDirty.value[config.id]) {
        // Only update draft if not dirty
        envDraft.value[config.id] = Object.entries(config.env).map(([key, value]) => ({
          key,
          value,
        }));
        argsDraft.value[config.id] = config.args.join(" ");
        // Initialize boolean flags if missing
        if (applyWithoutRestart.value[config.id] === undefined) {
            applyWithoutRestart.value[config.id] = false;
        }
      }
    }
  },
  { immediate: true, deep: true }
);

function findServiceState(id: string) {
  return props.serviceStates.find((service) => service.id === id);
}

function markDirty(id: string) {
  serviceDirty.value[id] = true;
}

function isValidEnvKey(key: string) {
  return /^[A-Z_][A-Z0-9_]*$/.test(key);
}

function isSensitiveKey(key: string) {
  return /(SECRET|PASSWORD|TOKEN|KEY)/i.test(key);
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

function detectPortConflict(currentId: string, port: number) {
  if (!port) return null;
  for (const config of props.configs) {
    if (config.id !== currentId && config.ports.main === port) {
      return config.id;
    }
  }
  return null;
}

async function prepareConfig(config: ServiceConfig): Promise<ServiceConfig | null> {
  serviceConfigError.value = null;
  const envEntries = (envDraft.value[config.id] || []).filter((entry) => entry.key.trim());
  
  for (const entry of envEntries) {
    if (!isValidEnvKey(entry.key)) {
      serviceConfigError.value = `invalid env key: ${entry.key}`;
      return null;
    }
  }
  
  const envError = validateEnvEntries(envEntries);
  if (envError) {
    serviceConfigError.value = envError;
    return null;
  }

  const mainPort = config.ports.main;
  if (mainPort < 0 || mainPort > 65535) {
    serviceConfigError.value = "port out of range";
    return null;
  }
  
  if (mainPort > 0 && mainPort < 1024) {
      if (!confirm(`Port ${mainPort} is a privileged port (requires root/admin). Continue?`)) {
          return null;
      }
  }
  
  const conflict = detectPortConflict(config.id, mainPort);
  if (conflict) {
    serviceConfigError.value = `port conflict with ${conflict}`;
    return null;
  }

  if (mainPort > 0) {
      try {
          const isFree: boolean = await invoke("check_port_availability", { port: mainPort });
          // If service is running, it might occupy the port itself, so we skip check if running and port hasn't changed.
          // But here we don't know if port changed easily without old config.
          // Assuming if dirty, port might have changed.
          // Simple heuristic: if port is occupied and service is NOT running, warn.
          const state = findServiceState(config.id);
          if (!isFree && state?.state !== 'running') {
               if (!confirm(`Port ${mainPort} appears to be in use by another application. Continue?`)) {
                   return null;
               }
          }
      } catch (e) {
          console.error("Port check failed", e);
      }
  }

  return {
    ...config,
    env: Object.fromEntries(envEntries.map(e => [e.key, e.value])),
    args: (argsDraft.value[config.id] || "")
      .split(" ")
      .map((arg) => arg.trim())
      .filter(Boolean),
  };
}

async function onSave(config: ServiceConfig) {
  const prepared = await prepareConfig(config);
  if (prepared) {
    emit("save", prepared);
    serviceDirty.value[config.id] = false;
  }
}

async function onApply(config: ServiceConfig) {
  const prepared = await prepareConfig(config);
  if (prepared) {
     emit("apply", prepared, !applyWithoutRestart.value[config.id]);
     serviceDirty.value[config.id] = false;
  }
}

function onReset(id: string) {
    if (confirm(`Reset config for ${id}?`)) {
        emit("reset", id);
        serviceDirty.value[id] = false;
    }
}

function saveAll() {
    for (const config of props.configs) {
        if (serviceDirty.value[config.id]) {
            onSave(config);
        }
    }
}

function addEnvRow(id: string) {
  if (!envDraft.value[id]) envDraft.value[id] = [];
  envDraft.value[id].push({ key: "", value: "" });
  markDirty(id);
}
</script>

<template>
  <section class="notice">
    <h3>Service Settings</h3>
    <div class="actions-inline">
      <button class="ghost" @click="saveAll">Save All Changes</button>
    </div>
    
    <div class="project-form" v-if="configs.length">
      <div v-for="config in configs" :key="config.id" class="service-block">
        <div class="service-head">
            <strong>{{ config.id }}</strong>
            <span
              v-if="findServiceState(config.id)"
              class="status"
              :data-state="findServiceState(config.id)?.state"
            >
              {{ findServiceState(config.id)?.state }}
            </span>
            <span v-if="applyWithoutRestart[config.id]" class="hint">no-restart</span>
        </div>

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
              <input
                v-model="row.value"
                :type="isSensitiveKey(row.key) ? 'password' : 'text'"
                placeholder="VALUE"
                @input="markDirty(config.id)"
              />
            </div>
            <button class="ghost small" @click="addEnvRow(config.id)">Add Env</button>
          </div>
        </div>
        
        <div class="service-config">
          <label>Apply Without Restart</label>
          <input type="checkbox" v-model="applyWithoutRestart[config.id]" />
        </div>
        
        <div class="service-actions">
          <button class="ghost" @click="onSave(config)">Save</button>
          <button class="primary" @click="onApply(config)">Apply & Restart</button>
          <button class="ghost" @click="emit('edit-config', config.id)">Edit Config</button>
          <button class="ghost" @click="onReset(config.id)">Reset</button>
          <span v-if="serviceDirty[config.id]" class="dirty">unsaved</span>
        </div>
        <hr class="separator"/>
      </div>
    </div>
    
    <p v-if="serviceConfigError" class="error-inline">{{ serviceConfigError }}</p>
  </section>
</template>

<style scoped>
.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.service-block {
    margin-bottom: 24px;
}

.service-head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}

.status {
  font-weight: 600;
  text-transform: uppercase;
  font-size: 12px;
}

.status[data-state="running"] { color: #0b7a3e; }
.status[data-state="starting"] { color: #c17b1b; }
.status[data-state="stopped"] { color: #b23b3b; }

.actions-inline {
  margin-bottom: 12px;
}

.project-form {
  display: grid;
  gap: 6px;
}

.service-config {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  margin: 6px 0;
}

.service-config label {
    min-width: 60px;
    font-size: 13px;
    padding-top: 6px;
}

.service-actions {
    display: flex;
    gap: 8px;
    align-items: center;
    margin-top: 12px;
}

.separator {
    border: 0;
    border-bottom: 1px dashed #ccc;
    margin-top: 24px;
}

input {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
}

.env-list {
  display: grid;
  gap: 6px;
  flex: 1;
}

.env-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.service-json pre {
  background: #f3f3f3;
  border: 1px solid #d0d0d0;
  padding: 8px;
  overflow: auto;
  max-height: 160px;
  font-size: 12px;
}

.dirty {
  font-size: 12px;
  text-transform: uppercase;
  color: #c17b1b;
  font-weight: bold;
}

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
}

.hint {
  font-size: 11px;
  text-transform: uppercase;
  color: #6b6b6b;
}

button {
  border: 2px solid #1b1b1b;
  background: #fefefe;
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
}

button.small {
    padding: 2px 6px;
    font-size: 11px;
}

.primary { background: #ffd36a; }
.ghost { background: #ffffff; }
</style>
