<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ProjectConfig } from "../types";

const props = defineProps<{
  projects: ProjectConfig[];
}>();

const selectedProject = ref<string>("");
const scripts = ref<Record<string, string>>({});
const runningTasks = ref<Set<string>>(new Set());
const errorMsg = ref<string | null>(null);

watch(selectedProject, async (id) => {
  if (!id) {
    scripts.value = {};
    return;
  }
  const project = props.projects.find(p => p.id === id);
  if (project) {
    try {
      errorMsg.value = null;
      const res: Record<string, string> = await invoke("task_list_scripts", { path: project.path });
      scripts.value = res;
    } catch (e) {
      scripts.value = {};
      errorMsg.value = String(e);
    }
  }
});

async function runScript(script: string) {
  if (!selectedProject.value) return;
  const project = props.projects.find(p => p.id === selectedProject.value);
  if (!project) return;
  
  try {
    errorMsg.value = null;
    await invoke("task_run", { 
        projectId: project.id, 
        path: project.path, 
        script 
    });
    runningTasks.value.add(project.id); // Simple tracking, ideally backend pushes state
  } catch (e) {
    errorMsg.value = String(e);
  }
}

async function stopTask() {
  if (!selectedProject.value) return;
  try {
    await invoke("task_stop", { projectId: selectedProject.value });
    runningTasks.value.delete(selectedProject.value);
  } catch (e) {
    errorMsg.value = String(e);
  }
}
</script>

<template>
  <section class="notice">
    <h3>Task Manager (Node.js)</h3>
    <div class="task-controls">
        <select v-model="selectedProject">
            <option value="" disabled>Select Project</option>
            <option v-for="p in projects" :key="p.id" :value="p.id">
                {{ p.name }} ({{ p.stack }})
            </option>
        </select>
    </div>
    
    <div v-if="selectedProject" class="scripts-list">
        <div v-if="runningTasks.has(selectedProject)" class="running-indicator">
            Task Running... 
            <button class="secondary small" @click="stopTask">Stop</button>
        </div>
        
        <div v-for="(cmd, name) in scripts" :key="name" class="script-row">
            <strong>{{ name }}</strong>
            <code class="cmd">{{ cmd }}</code>
            <button class="ghost small" @click="runScript(name)" :disabled="runningTasks.has(selectedProject)">Run</button>
        </div>
        
        <p v-if="Object.keys(scripts).length === 0 && !errorMsg">No scripts found in package.json</p>
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

.task-controls {
    margin-bottom: 12px;
}

select {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
  width: 100%;
}

.script-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px;
    background: #fff;
    border-bottom: 1px solid #eee;
    font-size: 13px;
}

.cmd {
    font-family: monospace;
    color: #666;
    margin: 0 8px;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

button.small {
    padding: 2px 8px;
    font-size: 11px;
}

button {
  border: 2px solid #1b1b1b;
  background: #fefefe;
  padding: 4px 8px;
  font-weight: 600;
  cursor: pointer;
}

.ghost { background: #ffffff; }
.secondary { background: #c7e5ff; }

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
}

.running-indicator {
    padding: 8px;
    background: #e6fffa;
    border: 1px solid #38b2ac;
    margin-bottom: 8px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-weight: bold;
    font-size: 13px;
}
</style>
