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
  <section class="card" v-if="localConfig">
    <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
        <h3 class="text-lg font-black uppercase">System Config</h3>
        <span class="tech-label">CORE_PREFS</span>
    </div>

    <div class="grid grid-cols-1 gap-4">
      <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="tech-label">ROOT_PATH</label>
            <input v-model="localConfig.installPath" class="input font-mono text-xs" />
          </div>
          <div>
            <label class="tech-label">SCHEMA_VER</label>
            <input v-model.number="localConfig.schemaVersion" disabled class="input font-mono text-xs opacity-50 cursor-not-allowed" />
          </div>
      </div>
      
      <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="tech-label">UPDATE_CHANNEL</label>
            <select v-model="localConfig.updateChannel" class="input text-xs font-mono uppercase">
              <option value="stable">STABLE</option>
              <option value="beta">BETA</option>
            </select>
          </div>
          <div>
            <label class="tech-label">FEED_URL</label>
            <input v-model="localConfig.updateFeedUrl" class="input font-mono text-xs" />
          </div>
      </div>
      
      <div class="flex justify-end pt-2 border-t border-[var(--border-color)] border-dashed">
        <button class="btn btn-primary" @click="onSave">SAVE CONFIGURATION</button>
      </div>
    </div>
    <p v-if="configError" class="error mt-4 font-mono text-xs">{{ configError }}</p>
  </section>
</template>

<style scoped>
/* Scoped styles removed */
</style>
