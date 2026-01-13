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
  <section class="notice">
    <h3>PHP Extensions</h3>
    <div class="actions-inline">
        <button class="ghost" @click="loadExtensions">Refresh</button>
    </div>
    
    <div class="ext-grid">
      <label v-for="ext in extensions" :key="ext.name" class="ext-item">
        <input 
            type="checkbox" 
            :checked="ext.enabled" 
            @change="toggleExtension(ext.name, ($event.target as HTMLInputElement).checked)"
        />
        {{ ext.name }}
      </label>
    </div>
    
    <p v-if="errorMsg" class="error-inline">{{ errorMsg }}</p>
  </section>
</template>

<style scoped>
.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.actions-inline {
    margin-bottom: 12px;
}

.ext-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 8px;
}

.ext-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  background: #fff;
  padding: 6px;
  border: 1px solid #ddd;
}

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
  margin-top: 8px;
}

button {
  border: 2px solid #1b1b1b;
  background: #fefefe;
  padding: 4px 8px;
  font-weight: 600;
  cursor: pointer;
}

.ghost { background: #ffffff; }
</style>
