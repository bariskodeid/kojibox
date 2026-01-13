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
  <section class="card">
    <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
        <h3 class="text-lg font-black uppercase">Task Runner</h3>
        <span class="tech-label">NODE_PROC</span>
    </div>

    <div class="mb-4">
        <select v-model="selectedProject" class="input font-mono uppercase text-xs">
            <option value="" disabled>Select Target Project</option>
            <option v-for="p in projects" :key="p.id" :value="p.id">
                {{ p.name }} [{{ p.stack.toUpperCase() }}]
            </option>
        </select>
    </div>
    
    <div v-if="selectedProject" class="border border-[var(--border-color)]">
        <div v-if="runningTasks.has(selectedProject)" class="p-2 bg-[var(--accent-color)] text-white flex justify-between items-center">
            <span class="font-mono font-bold text-xs animate-pulse">● TASK_ACTIVE</span>
            <button class="bg-black text-white px-2 py-0.5 text-xs font-bold uppercase border border-white hover:bg-white hover:text-black transition-colors" @click="stopTask">ABORT</button>
        </div>
        
        <div v-for="(cmd, name) in scripts" :key="name" class="flex items-center justify-between p-2 border-b border-[var(--border-color)] last:border-b-0 hover:bg-[var(--code-bg)]">
            <div class="overflow-hidden mr-2">
                <div class="font-bold text-xs uppercase">{{ name }}</div>
                <div class="font-mono text-[10px] text-[var(--secondary-color)] truncate">{{ cmd }}</div>
            </div>
            <button class="btn px-2 py-1 text-[10px] h-6" @click="runScript(name)" :disabled="runningTasks.has(selectedProject)">RUN</button>
        </div>
        
        <div v-if="Object.keys(scripts).length === 0 && !errorMsg" class="p-4 text-center text-xs font-mono text-[var(--secondary-color)]">
            // NO SCRIPTS FOUND
        </div>
    </div>
    <div v-else class="p-8 text-center text-xs font-mono text-[var(--secondary-color)] border border-dashed border-[var(--border-color)]">
        SELECT PROJECT TO LOAD SCRIPTS
    </div>
    
    <p v-if="errorMsg" class="error mt-4 text-xs font-mono">{{ errorMsg }}</p>
  </section>
</template>

<style scoped>
/* Scoped styles removed */
</style>
