<script setup lang="ts">
import { ref } from "vue";
import type { ProjectConfig } from "../types";

const props = defineProps<{
  projects: ProjectConfig[];
}>();

const emit = defineEmits<{
  (e: "save", project: ProjectConfig): void;
  (e: "delete", id: string): void;
}>();

const newProject = ref<ProjectConfig>({
  schemaVersion: 1,
  id: "",
  name: "",
  path: "",
  domain: "",
  stack: "php",
  overrides: {},
});

const projectError = ref<string | null>(null);
const isEditingProject = ref(false);
const projectOverridesDraft = ref<{ key: string; value: string }[]>([]);

function isValidDomain(domain: string) {
  return /^[a-zA-Z0-9.-]+\.[a-zA-Z0-9-]+$/.test(domain);
}

function isSensitiveKey(key: string) {
  return /(SECRET|PASSWORD|TOKEN|KEY)/i.test(key);
}

function editProject(project: ProjectConfig) {
  newProject.value = { ...project };
  projectOverridesDraft.value = Object.entries(project.overrides || {}).map(
    ([key, value]) => ({
      key,
      value,
    })
  );
  projectError.value = null;
  isEditingProject.value = true;
}

function resetProjectForm() {
  newProject.value = {
    schemaVersion: 1,
    id: "",
    name: "",
    path: "",
    domain: "",
    stack: "php",
    overrides: {},
  };
  projectError.value = null;
  isEditingProject.value = false;
  projectOverridesDraft.value = [];
}

function isValidPath(path: string) {
  // Basic check: should not contain invalid chars for typical filesystems
  // Allow alphanumeric, slash, backslash, hyphen, underscore, dot, colon (windows drive)
  return /^[a-zA-Z0-9_\-./\\:]+$/.test(path);
}

function onSave() {
  if (!newProject.value.id) {
    projectError.value = "project id is required";
    return;
  }
  
  // Check for duplicates (only for new projects)
  if (!isEditingProject.value) {
      if (props.projects.some(p => p.id === newProject.value.id)) {
          projectError.value = "project id already exists";
          return;
      }
  }

  if (!isValidDomain(newProject.value.domain)) {
    projectError.value = "invalid domain";
    return;
  }
  
  if (!newProject.value.path.trim()) {
      projectError.value = "path is required";
      return;
  }

  if (!isValidPath(newProject.value.path)) {
    projectError.value = "path contains invalid characters";
    return;
  }
  
  if (!["php", "node"].includes(newProject.value.stack)) {
    projectError.value = "invalid stack";
    return;
  }

  const overrides = projectOverridesDraft.value
    .filter((entry) => entry.key.trim())
    .reduce<Record<string, string>>((acc, entry) => {
      acc[entry.key] = entry.value;
      return acc;
    }, {});
  
  newProject.value.overrides = overrides;
  
  emit("save", newProject.value);
  resetProjectForm();
}

function onDelete(id: string) {
  if (confirm(`Delete project ${id}?`)) {
    emit("delete", id);
  }
}
</script>

<template>
  <section class="card">
    <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
        <h3 class="text-lg font-black uppercase">Project Registry</h3>
        <span class="tech-label">Workspace</span>
    </div>

    <div class="border-2 border-[var(--border-color)] p-4 mb-6 bg-[var(--code-bg)]">
      <h4 class="font-bold uppercase text-xs mb-3 border-b border-[var(--border-color)] pb-1">
          {{ isEditingProject ? 'Edit Configuration' : 'Register New Project' }}
      </h4>
      <div class="grid grid-cols-1 gap-3">
        <div class="grid grid-cols-2 gap-3">
            <input v-model="newProject.id" class="input font-mono uppercase" placeholder="PROJECT_ID" :disabled="isEditingProject" />
            <input v-model="newProject.name" class="input" placeholder="Display Name" />
        </div>
        <input v-model="newProject.path" class="input font-mono text-xs" placeholder="/absolute/path/to/project" />
        <div class="grid grid-cols-2 gap-3">
            <input v-model="newProject.domain" class="input font-mono lowercase" placeholder="project.test" />
            <select v-model="newProject.stack" class="input uppercase">
                <option value="php">PHP Stack</option>
                <option value="node">Node.js Stack</option>
            </select>
        </div>
        
        <div class="mt-2">
          <label class="tech-label mb-1">ENV_OVERRIDES</label>
          <div class="space-y-2">
            <div v-for="(row, idx) in projectOverridesDraft" :key="idx" class="flex gap-2">
              <input v-model="row.key" class="input font-mono text-xs uppercase" placeholder="KEY" />
              <input
                v-model="row.value"
                :type="isSensitiveKey(row.key) ? 'password' : 'text'"
                class="input font-mono text-xs"
                placeholder="VALUE"
              />
            </div>
            <button class="btn btn-sm w-full border-dashed" type="button" @click="projectOverridesDraft.push({ key: '', value: '' })">
              + Add Variable
            </button>
          </div>
        </div>
        
        <div class="flex gap-2 mt-2">
          <button class="btn btn-primary flex-1" @click="onSave">{{ isEditingProject ? 'Update' : 'Register' }}</button>
          <button v-if="isEditingProject" class="btn flex-1" @click="resetProjectForm">Cancel</button>
        </div>
      </div>
    </div>
    
    <p v-if="projectError" class="error mb-4 font-mono text-xs">{{ projectError }}</p>
    
    <ul class="space-y-2">
      <li v-for="project in projects" :key="project.id" class="flex items-center justify-between p-3 border border-[var(--border-color)] bg-[var(--card-bg)] hover:shadow-md transition-all">
        <div>
            <div class="flex items-center gap-2">
                <strong class="uppercase text-sm">{{ project.name }}</strong>
                <span class="badge bg-[var(--code-bg)]">{{ project.stack }}</span>
            </div>
            <div class="font-mono text-xs text-[var(--secondary-color)]">{{ project.domain }}</div>
        </div>
        <div class="flex gap-2">
           <button class="btn px-2 py-1 text-[10px]" @click="editProject(project)">EDIT</button>
           <button class="btn px-2 py-1 text-[10px] border-[var(--error-color)] text-[var(--error-color)]" @click="onDelete(project.id)">DEL</button>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
/* Scoped styles removed */
</style>
