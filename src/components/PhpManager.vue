<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const extensions = ref<Array<{ name: string; enabled: boolean }>>([]);
const errorMsg = ref<string | null>(null);

const emit = defineEmits<{
  (e: "restart-php"): void;
}>();

async function loadExtensions() {
  try {
    errorMsg.value = null;
    const list: Array<[string, boolean]> = await invoke("php_extensions_list");
    extensions.value = list.map(([name, enabled]) => ({ name, enabled }));
  } catch (err) {
    errorMsg.value = String(err);
  }
}

async function toggleExtension(name: string, enabled: boolean) {
  try {
    errorMsg.value = null;
    await invoke("php_extensions_toggle", { name, enabled });
    await loadExtensions();
    if (confirm(`Extension ${name} ${enabled ? 'enabled' : 'disabled'}. Restart PHP service now?`)) {
        emit("restart-php");
    }
  } catch (err) {
    errorMsg.value = String(err);
  }
}

onMounted(loadExtensions);
</script>

<template>
  <section class="card">
    <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
        <h3 class="text-lg font-black uppercase">PHP Modules</h3>
        <button class="btn btn-sm text-[10px]" @click="loadExtensions">REFRESH</button>
    </div>
    
    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2 text-xs font-mono">
      <label v-for="ext in extensions" :key="ext.name" class="flex items-center gap-2 p-2 border border-[var(--border-color)] hover:bg-[var(--code-bg)] cursor-pointer select-none transition-colors">
        <input 
            type="checkbox" 
            :checked="ext.enabled" 
            @change="toggleExtension(ext.name, ($event.target as HTMLInputElement).checked)"
            class="w-3 h-3 rounded-none border border-[var(--border-color)] text-[var(--accent-color)] focus:ring-0"
        />
        <span :class="{'font-bold text-[var(--text-color)]': ext.enabled, 'text-[var(--secondary-color)]': !ext.enabled}">{{ ext.name }}</span>
      </label>
    </div>
    
    <p v-if="errorMsg" class="error mt-4 font-mono text-xs">{{ errorMsg }}</p>
  </section>
</template>

<style scoped>
/* Scoped styles removed */
</style>
