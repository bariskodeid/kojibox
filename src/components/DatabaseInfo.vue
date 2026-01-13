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
  <section class="notice" v-if="databases.length">
    <h3>Database Connections</h3>
    <div class="db-grid">
      <div v-for="db in databases" :key="db.id" class="db-card">
        <h4>{{ db.name }}</h4>
        <div class="db-detail">
          <span>Host:</span> <strong>127.0.0.1</strong>
        </div>
        <div class="db-detail">
          <span>Port:</span> <strong>{{ db.port }}</strong>
        </div>
        <div class="db-detail">
          <span>User:</span> <strong>{{ db.user }}</strong>
        </div>
        <div class="db-detail">
          <span>Pass:</span> <strong>{{ db.pass }}</strong>
        </div>
        <div class="db-actions">
            <button class="ghost small" @click="copyToClipboard(db.connString)">Copy Connection String</button>
            <a :href="db.connString" target="_blank" class="button ghost small">Open in Client</a>
            <button class="secondary small" @click="exportDb(db.id, db.db)">Export SQL</button>
        </div>
      </div>
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

.db-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  margin-top: 8px;
}

.db-card {
  background: #fff;
  border: 1px solid #ccc;
  padding: 12px;
}

.db-card h4 {
  margin: 0 0 8px;
  text-transform: uppercase;
  font-size: 12px;
}

.db-detail {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  margin-bottom: 4px;
}

.db-actions {
    display: grid;
    gap: 8px;
    margin-top: 8px;
}

button, .button {
  border: 2px solid #1b1b1b;
  background: #fefefe;
  padding: 4px 8px;
  font-weight: 600;
  cursor: pointer;
  width: 100%;
  text-align: center;
  text-decoration: none;
  display: block;
  font-size: 11px;
  color: #1b1b1b;
  box-sizing: border-box;
}

.ghost { background: #ffffff; }
</style>
