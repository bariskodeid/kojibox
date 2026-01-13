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
  <div v-if="needsSetup" class="wizard-backdrop">
    <section class="wizard">
      <header>
        <h2>First Run Setup</h2>
        <p>Set install path, ports, and default services.</p>
      </header>
      
      <!-- Step 1: Basic Config -->
      <div v-if="wizardStep === 1" class="wizard-step">
        <label>
          Install Path
          <input v-model="wizardConfig.installPath" placeholder="app" />
        </label>
        <label>
          Update Channel
          <select v-model="wizardConfig.updateChannel">
            <option value="stable">stable</option>
            <option value="beta">beta</option>
          </select>
        </label>
        <label>
          Update Feed URL
          <input v-model="wizardConfig.updateFeedUrl" />
        </label>
        <label class="inline">
          <input type="checkbox" v-model="wizardConfig.telemetryOptIn" />
          Allow telemetry (opt-in)
        </label>
      </div>

      <!-- Step 2: Ports -->
      <div v-else-if="wizardStep === 2" class="wizard-step">
        <h3>Default Ports</h3>
        <div class="wizard-grid">
          <label v-for="(_port, id) in wizardPorts" :key="id">
            {{ id }}
            <input v-model.number="wizardPorts[id]" type="number" min="1" max="65535" />
          </label>
        </div>
      </div>

      <!-- Step 3: Services -->
      <div v-else class="wizard-step">
        <h3>Enable Services</h3>
        <div class="wizard-grid">
          <label v-for="(_enabled, id) in wizardServices" :key="id" class="inline">
            <input type="checkbox" v-model="wizardServices[id]" />
            {{ id }}
          </label>
        </div>
      </div>

      <p class="error" v-if="wizardError">{{ wizardError }}</p>

      <footer class="wizard-actions">
        <button class="ghost" :disabled="wizardStep === 1" @click="prevWizard">Back</button>
        <button class="secondary" v-if="wizardStep < 3" @click="nextWizard">Next</button>
        <button class="primary" v-else @click="finishWizard">Finish</button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.wizard-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(17, 17, 17, 0.65);
  display: grid;
  place-items: center;
  z-index: 20;
  padding: 20px;
}

.wizard {
  background: #ffffff;
  border: 2px solid #1b1b1b;
  padding: 20px;
  max-width: 560px;
  width: 100%;
  display: grid;
  gap: 12px;
}

.wizard h2 {
  margin: 0 0 4px;
}

.wizard-step {
    display: grid;
    gap: 12px;
}

.wizard-step label {
  display: grid;
  gap: 6px;
  font-size: 13px;
}

.wizard-grid {
  display: grid;
  gap: 10px;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
}

.wizard-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.inline {
  display: flex;
  align-items: center;
  gap: 8px;
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

.error {
  background: #fbe3e3;
  border: 1px solid #d96a6a;
  padding: 12px 16px;
  margin-bottom: 16px;
}

input,
select {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
}

button {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
