<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Types
type AppConfig = {
  schemaVersion: number;
  installPath: string;
  updateChannel: string;
  telemetryOptIn: boolean;
  updateFeedUrl: string;
  updatePublicKeys: string[];
};

defineProps<{
  needsSetup: boolean;
}>();

const emit = defineEmits<{
  (e: "setup-complete"): void;
  (e: "cancel"): void; // Optional if needed
}>();

const wizardStep = ref(1);
const wizardError = ref<string | null>(null);
const wizardConfig = ref<AppConfig>({
  schemaVersion: 1,
  installPath: "app",
  updateChannel: "stable",
  telemetryOptIn: false,
  updateFeedUrl: "https://updates.kojibox.dev/feed.json",
  updatePublicKeys: [],
});

const wizardPorts = ref<Record<string, number>>({
  php: 9000,
  node: 3000,
  postgres: 5432,
  mariadb: 3306,
  mailpit: 8025,
});

const wizardServices = ref<Record<string, boolean>>({
  php: true,
  node: true,
  postgres: true,
  mariadb: true,
  mailpit: true,
});

function validateWizardPorts() {
  const seen = new Map<number, string>();
  for (const [id, port] of Object.entries(wizardPorts.value)) {
    if (!Number.isFinite(port) || port < 1 || port > 65535) {
      return `invalid port for ${id}`;
    }
    if (seen.has(port)) {
      return `port conflict between ${id} and ${seen.get(port)}`;
    }
    seen.set(port, id);
  }
  return null;
}

async function finishWizard() {
  const conflict = validateWizardPorts();
  if (conflict) {
    wizardError.value = conflict;
    return;
  }
  if (!wizardConfig.value.installPath.trim()) {
    wizardError.value = "install path is required";
    return;
  }
  if (!/^[a-zA-Z0-9_\-./\\]+$/.test(wizardConfig.value.installPath)) {
      wizardError.value = "install path contains invalid characters";
      return;
  }
  try {
    wizardError.value = null;
    await invoke("config_set_app", { app: wizardConfig.value });
    
    // We need to fetch the current registry to preserve ranges if they exist, 
    // though for first run it might be empty.
    let registry: { 
      schemaVersion: number; 
      assigned: Record<string, number>; 
      ranges: Record<string, { from: number; to: number }> 
    };
    
    try {
        registry = await invoke("config_get_ports");
    } catch {
        // Fallback if config doesn't exist yet
        registry = { schemaVersion: 1, assigned: {}, ranges: {} };
    }

    registry.assigned = { ...wizardPorts.value };
    await invoke("config_set_ports", { registry });

    for (const [id, enabled] of Object.entries(wizardServices.value)) {
      await invoke("config_set_service", {
        service: {
          schemaVersion: 1,
          id,
          enabled,
          ports: { main: wizardPorts.value[id] },
          env: {},
          args: [],
        },
      });
    }
    
    // Trigger installer start
    await invoke("installer_start");
    
    emit("setup-complete");
  } catch (error) {
    wizardError.value = String(error);
  }
}

function nextWizard() {
  if (wizardStep.value === 1 && !wizardConfig.value.installPath.trim()) {
    wizardError.value = "install path is required";
    return;
  }
  wizardError.value = null;
  wizardStep.value = Math.min(3, wizardStep.value + 1);
}

function prevWizard() {
  wizardError.value = null;
  wizardStep.value = Math.max(1, wizardStep.value - 1);
}
</script>

<template>
  <div v-if="needsSetup" class="fixed inset-0 bg-black/80 grid place-items-center z-[2000]">
    <section class="card w-full max-w-lg shadow-[8px_8px_0_var(--accent-color)] border-[var(--accent-color)]">
      <header class="mb-6 border-b-2 border-[var(--border-color)] pb-4">
        <h2 class="text-3xl font-black uppercase mb-1">Initial Setup</h2>
        <p class="font-mono text-xs text-[var(--secondary-color)] uppercase tracking-widest">System Configuration Wizard</p>
      </header>
      
      <!-- Step 1: Basic Config -->
      <div v-if="wizardStep === 1" class="space-y-4">
        <div>
            <label class="tech-label">INSTALL_ROOT</label>
            <input v-model="wizardConfig.installPath" placeholder="app" class="input font-mono" />
        </div>
        <div class="grid grid-cols-2 gap-4">
            <div>
                <label class="tech-label">UPDATE_CHANNEL</label>
                <select v-model="wizardConfig.updateChannel" class="input font-mono uppercase">
                  <option value="stable">STABLE</option>
                  <option value="beta">BETA</option>
                </select>
            </div>
            <div>
                <label class="tech-label">TELEMETRY</label>
                <label class="flex items-center gap-2 h-full cursor-pointer border border-[var(--border-color)] px-3 bg-[var(--code-bg)] hover:bg-white transition-colors">
                  <input type="checkbox" v-model="wizardConfig.telemetryOptIn" class="w-4 h-4 rounded-none border-2 border-black text-[var(--accent-color)] focus:ring-0" />
                  <span class="font-bold text-xs uppercase">OPT-IN ENABLED</span>
                </label>
            </div>
        </div>
        <div>
            <label class="tech-label">FEED_URL</label>
            <input v-model="wizardConfig.updateFeedUrl" class="input font-mono text-xs" />
        </div>
      </div>

      <!-- Step 2: Ports -->
      <div v-else-if="wizardStep === 2" class="space-y-4">
        <div class="flex justify-between items-center mb-2">
            <h3 class="font-bold uppercase">Port Assignment</h3>
            <span class="tech-label">NET_CONFIG</span>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <label v-for="(_port, id) in wizardPorts" :key="id" class="border border-[var(--border-color)] p-2 flex justify-between items-center bg-[var(--code-bg)]">
            <span class="font-bold text-xs uppercase">{{ id }}</span>
            <input v-model.number="wizardPorts[id]" type="number" min="1" max="65535" class="w-20 text-right font-mono text-xs border-b border-black bg-transparent focus:outline-none" />
          </label>
        </div>
      </div>

      <!-- Step 3: Services -->
      <div v-else class="space-y-4">
        <div class="flex justify-between items-center mb-2">
            <h3 class="font-bold uppercase">Service Selection</h3>
            <span class="tech-label">DAEMON_LIST</span>
        </div>
        <div class="grid grid-cols-2 gap-3">
          <label v-for="(_enabled, id) in wizardServices" :key="id" class="flex items-center gap-3 p-3 border border-[var(--border-color)] cursor-pointer hover:bg-[var(--code-bg)] transition-colors select-none">
            <input type="checkbox" v-model="wizardServices[id]" class="w-5 h-5 rounded-none border-2 border-black text-[var(--accent-color)] focus:ring-0" />
            <span class="font-bold uppercase text-sm">{{ id }}</span>
          </label>
        </div>
      </div>

      <p class="error mt-4 font-mono text-xs border-l-4 border-[var(--error-color)] bg-red-50 p-2" v-if="wizardError">
          ERR: {{ wizardError }}
      </p>

      <footer class="flex justify-between mt-8 pt-4 border-t-2 border-[var(--border-color)] border-dashed">
        <button class="btn w-24" :disabled="wizardStep === 1" @click="prevWizard">BACK</button>
        
        <div class="flex gap-1">
            <div v-for="i in 3" :key="i" class="w-3 h-3 border border-black" :class="i === wizardStep ? 'bg-[var(--accent-color)]' : 'bg-transparent'"></div>
        </div>

        <button class="btn btn-primary w-24" v-if="wizardStep < 3" @click="nextWizard">NEXT</button>
        <button class="btn btn-primary w-24 bg-[var(--success-color)] text-white border-[var(--success-color)] hover:bg-green-600" v-else @click="finishWizard">FINISH</button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
/* Scoped styles removed */
</style>
