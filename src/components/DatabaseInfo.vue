<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ServiceState, ServiceConfig } from "../types";

const props = defineProps<{
  services: ServiceState[];
  configs: ServiceConfig[];
}>();

const errorMsg = ref<string | null>(null);

const databases = computed(() => {
// ... existing computed ...
  return props.services
    .filter(s => (s.id === "postgres" || s.id === "mariadb") && s.state === "running")
    .map(s => {
      const config = props.configs.find(c => c.id === s.id);
      const port = config?.ports.main || (s.id === "postgres" ? 5432 : 3306);
      const user = s.id === "postgres" ? "postgres" : "root";
      const pass = s.id === "postgres" ? "(none)" : "(none)";
      const db = s.id === "postgres" ? "postgres" : "test";
      
      const connString = s.id === "postgres"
        ? `postgresql://${user}@127.0.0.1:${port}/${db}`
        : `mysql://${user}@127.0.0.1:${port}/${db}`;

      return {
        id: s.id,
        name: s.id === "postgres" ? "PostgreSQL" : "MariaDB",
        port,
        user,
        pass,
        db,
        connString
      };
    });
});

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    alert("Copied to clipboard!");
  } catch (err) {
    console.error(err);
  }
}

async function exportDb(serviceId: string, dbName: string) {
    const filename = `${serviceId}_${dbName}_${Date.now()}.sql`;
    // Ideally open save dialog, for MVP dump to app root or downloads
    const path = `dump/${filename}`; 
    try {
        errorMsg.value = null;
        await invoke("db_dump", { service: serviceId, dbName, path });
        alert(`Dumped to ${path}`);
    } catch (e) {
        errorMsg.value = String(e);
        alert("Dump failed: " + e);
    }
}
</script>

<template>
  <section class="card" v-if="databases.length">
    <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
        <h3 class="text-lg font-black uppercase">Data Access</h3>
        <span class="tech-label">SQL_CONN</span>
    </div>
    
    <div class="grid grid-cols-1 gap-4">
      <div v-for="db in databases" :key="db.id" class="border border-[var(--border-color)] p-3 bg-[var(--code-bg)]">
        <div class="flex justify-between items-center mb-2">
            <h4 class="font-bold uppercase tracking-tight">{{ db.name }}</h4>
            <span class="tech-label text-[var(--success-color)]">ONLINE</span>
        </div>
        
        <div class="grid grid-cols-2 gap-x-2 gap-y-1 text-xs font-mono mb-3">
          <span class="text-[var(--secondary-color)]">HOST</span> <span>127.0.0.1</span>
          <span class="text-[var(--secondary-color)]">PORT</span> <span>{{ db.port }}</span>
          <span class="text-[var(--secondary-color)]">USER</span> <span>{{ db.user }}</span>
          <span class="text-[var(--secondary-color)]">PASS</span> <span>{{ db.pass }}</span>
        </div>
        
        <div class="grid grid-cols-2 gap-2">
            <button class="btn btn-sm text-[10px]" @click="copyToClipboard(db.connString)">COPY DSN</button>
            <a :href="db.connString" target="_blank" class="btn btn-sm text-[10px] text-center decoration-none">OPEN CLIENT</a>
            <button class="btn btn-sm text-[10px] col-span-2 border-dashed" @click="exportDb(db.id, db.db)">EXPORT .SQL</button>
        </div>
      </div>
    </div>
    <p v-if="errorMsg" class="error mt-4">{{ errorMsg }}</p>
  </section>
</template>

<style scoped>
/* Scoped styles removed */
</style>
