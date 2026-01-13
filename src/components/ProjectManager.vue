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
  <section class="notice">
    <h3>Projects</h3>
    <div class="project-form">
      <input v-model="newProject.id" placeholder="id" :disabled="isEditingProject" />
      <input v-model="newProject.name" placeholder="name" />
      <input v-model="newProject.path" placeholder="path" />
      <input v-model="newProject.domain" placeholder="domain" />
      <input v-model.number="newProject.schemaVersion" disabled title="Schema Version" />
      <select v-model="newProject.stack">
        <option value="php">php</option>
        <option value="node">node</option>
      </select>
      
      <div class="project-overrides">
        <label>Overrides</label>
        <div class="env-list">
          <div v-for="(row, idx) in projectOverridesDraft" :key="idx" class="env-row">
            <input v-model="row.key" placeholder="KEY" />
            <input
              v-model="row.value"
              :type="isSensitiveKey(row.key) ? 'password' : 'text'"
              placeholder="VALUE"
            />
          </div>
          <button
            class="ghost"
            type="button"
            @click="projectOverridesDraft.push({ key: '', value: '' })"
          >
            Add Override
          </button>
        </div>
      </div>
      
      <div class="actions">
        <button class="ghost" @click="onSave">Save Project</button>
        <button v-if="isEditingProject" class="ghost" @click="resetProjectForm">
          Cancel Edit
        </button>
      </div>
    </div>
    
    <p v-if="projectError" class="error-inline">{{ projectError }}</p>
    
    <ul class="project-list">
      <li v-for="project in projects" :key="project.id">
        <strong>{{ project.name }}</strong> ({{ project.stack }}) - {{ project.domain }}
        <span class="hint" v-if="Object.keys(project.overrides || {}).length">
          overrides {{ Object.keys(project.overrides || {}).length }}
        </span>
        <span class="hint">v{{ project.schemaVersion }}</span>
        <div class="list-actions">
           <button class="ghost small" @click="editProject(project)">Edit</button>
           <button class="ghost small" @click="onDelete(project.id)">Delete</button>
        </div>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.project-form {
  display: grid;
  gap: 6px;
  margin-top: 8px;
}

input, select {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
}

.project-overrides {
  margin-top: 8px;
  border: 1px dashed #ccc;
  padding: 8px;
}

.project-overrides label {
  display: block;
  font-size: 12px;
  margin-bottom: 4px;
  text-transform: uppercase;
}

.env-list {
  display: grid;
  gap: 6px;
}

.env-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 6px;
}

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
  margin-top: 8px;
}

.project-list {
  margin: 16px 0 0;
  padding-left: 16px;
}

.project-list li {
    margin-bottom: 8px;
}

.hint {
  font-size: 11px;
  text-transform: uppercase;
  color: #6b6b6b;
  margin-left: 8px;
}

.actions {
    margin-top: 8px;
    display: flex;
    gap: 8px;
}

.list-actions {
    display: inline-flex;
    gap: 4px;
    margin-left: 8px;
}

button {
  border: 2px solid #1b1b1b;
  background: #fefefe;
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
}

button.small {
    padding: 2px 6px;
    font-size: 11px;
}

.ghost {
  background: #ffffff;
}
</style>
