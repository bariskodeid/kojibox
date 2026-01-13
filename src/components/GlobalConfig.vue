<script setup lang="ts">
import { ref, watch } from "vue";
import type { AppConfig } from "../types";

const props = defineProps<{
  appConfig: AppConfig | null;
  configError: string | null;
}>();

const emit = defineEmits<{
  (e: "save", config: AppConfig): void;
}>();

const localConfig = ref<AppConfig | null>(null);

watch(
  () => props.appConfig,
  (newVal) => {
    if (newVal) {
      localConfig.value = JSON.parse(JSON.stringify(newVal));
    }
  },
  { immediate: true, deep: true }
);

function onSave() {
  if (localConfig.value) {
    emit("save", localConfig.value);
  }
}
</script>

<template>
  <section class="notice" v-if="localConfig">
    <div class="config">
      <div>
        <label>Install Path</label>
        <input v-model="localConfig.installPath" />
      </div>
      <div>
        <label>Schema Version</label>
        <input v-model.number="localConfig.schemaVersion" disabled />
      </div>
      <div>
        <label>Update Channel</label>
        <select v-model="localConfig.updateChannel">
          <option value="stable">stable</option>
          <option value="beta">beta</option>
        </select>
      </div>
      <div>
        <label>Update Feed URL</label>
        <input v-model="localConfig.updateFeedUrl" />
      </div>
      <div class="actions-inline">
        <button class="ghost" @click="onSave">Save Config</button>
      </div>
    </div>
    <p v-if="configError" class="error-inline">{{ configError }}</p>
  </section>
</template>

<style scoped>
.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
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

.ghost { background: #ffffff; }

.actions-inline {
  margin-top: 8px;
}

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
}
</style>
