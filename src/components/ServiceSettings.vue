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
  <section class="card">
    <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
        <h3 class="text-lg font-black uppercase">Service Config</h3>
        <span class="tech-label">DAEMON_OPT</span>
    </div>
    
    <div class="actions-inline mb-4 flex justify-end">
      <button class="btn" @click="saveAll">Save All Changes</button>
    </div>
    
    <div v-if="configs.length" class="space-y-4">
      <div v-for="config in configs" :key="config.id" class="border border-[var(--border-color)] p-3 bg-[var(--code-bg)]">
        <div class="flex justify-between items-center mb-3 pb-2 border-b border-[var(--border-color)] border-dashed">
            <div class="flex items-center gap-2">
                <strong class="uppercase">{{ config.id }}</strong>
                <span
                  v-if="findServiceState(config.id)"
                  class="badge"
                  :class="{
                      'bg-green-100 text-green-800 border-green-800': findServiceState(config.id)?.state === 'running',
                      'bg-red-100 text-red-800 border-red-800': findServiceState(config.id)?.state !== 'running'
                  }"
                >
                  {{ findServiceState(config.id)?.state }}
                </span>
            </div>
            <div class="flex items-center gap-2">
                <label class="flex items-center gap-1 cursor-pointer">
                    <input type="checkbox" v-model="applyWithoutRestart[config.id]" class="w-3 h-3 border-[var(--border-color)]" />
                    <span class="tech-label !mb-0 text-[9px]">HOT_APPLY</span>
                </label>
                <span v-if="serviceDirty[config.id]" class="badge bg-[var(--warning-color)] border-black">UNSAVED</span>
            </div>
        </div>

        <div class="grid grid-cols-12 gap-3 mb-3">
            <div class="col-span-2 flex items-center gap-2">
                <input type="checkbox" v-model="config.enabled" @change="markDirty(config.id)" class="w-4 h-4 border-2 border-[var(--border-color)]" />
                <span class="font-bold text-xs uppercase">Enabled</span>
            </div>
            <div class="col-span-3">
                <div class="flex items-center">
                    <span class="tech-label mr-2">PORT</span>
                    <input v-model.number="config.ports.main" @input="markDirty(config.id)" class="input py-1 text-xs font-mono w-full" />
                </div>
            </div>
            <div class="col-span-7">
                <div class="flex items-center">
                    <span class="tech-label mr-2">ARGS</span>
                    <input
                      v-model="argsDraft[config.id]"
                      placeholder="--flag --value"
                      @input="markDirty(config.id)"
                      class="input py-1 text-xs font-mono w-full"
                    />
                </div>
            </div>
        </div>
        
        <div class="mb-3">
          <label class="tech-label">ENV_VARS</label>
          <div class="space-y-1">
            <div v-for="(row, idx) in envDraft[config.id] || []" :key="idx" class="grid grid-cols-2 gap-2">
              <input v-model="row.key" placeholder="KEY" @input="markDirty(config.id)" class="input py-1 text-xs font-mono uppercase bg-white" />
              <input
                v-model="row.value"
                :type="isSensitiveKey(row.key) ? 'password' : 'text'"
                placeholder="VALUE"
                @input="markDirty(config.id)"
                class="input py-1 text-xs font-mono bg-white"
              />
            </div>
            <button class="btn btn-sm w-full border-dashed text-[10px] py-1 mt-1" @click="addEnvRow(config.id)">+ ENV</button>
          </div>
        </div>
        
        <div class="grid grid-cols-4 gap-2 mt-4">
          <button class="btn" @click="onSave(config)">Save</button>
          <button class="btn btn-primary col-span-2" @click="onApply(config)">Apply & Restart</button>
          <button class="btn" @click="emit('edit-config', config.id)">Raw Edit</button>
        </div>
        <div class="mt-2 text-right">
             <button class="text-[9px] uppercase underline text-[var(--secondary-color)] hover:text-[var(--error-color)]" @click="onReset(config.id)">Reset Default</button>
        </div>
      </div>
    </div>
    
    <p v-if="serviceConfigError" class="error mt-4 font-mono text-xs">{{ serviceConfigError }}</p>
  </section>
</template>

<style scoped>
/* Scoped styles removed */
</style>
