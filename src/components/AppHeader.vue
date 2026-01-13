<script setup lang="ts">
import { ref, onMounted } from "vue";

defineProps<{
  logFilter: "all" | "error";
}>();

const emit = defineEmits<{
  (e: "update:logFilter", value: "all" | "error"): void;
  (e: "refresh"): void;
  (e: "export-diagnostics"): void;
  (e: "check-updates"): void;
  (e: "open-terminal"): void;
  (e: "open-about"): void;
}>();

const isDark = ref(false);

function toggleDark() {
    isDark.value = !isDark.value;
    if (isDark.value) {
        document.documentElement.classList.add('dark');
        localStorage.setItem('theme', 'dark');
    } else {
        document.documentElement.classList.remove('dark');
        localStorage.setItem('theme', 'light');
    }
}

onMounted(() => {
    const theme = localStorage.getItem('theme');
    if (theme === 'dark') {
        isDark.value = true;
        document.documentElement.classList.add('dark');
    }
});
</script>

<template>
  <header class="header">
    <div>
      <h1>Kojibox</h1>
      <p class="subtitle">Portable dev stack manager</p>
    </div>
    <div class="header-actions">
      <div class="filter">
        <button
          class="ghost"
          :data-active="logFilter === 'all'"
          @click="emit('update:logFilter', 'all')"
        >
          All Logs
        </button>
        <button
          class="ghost"
          :data-active="logFilter === 'error'"
          @click="emit('update:logFilter', 'error')"
        >
          Errors
        </button>
      </div>
      <button class="ghost" @click="toggleDark">{{ isDark ? '☀' : '🌙' }}</button>
      <button class="ghost" @click="emit('open-terminal')">Terminal</button>
      <button class="ghost" @click="emit('refresh')">Refresh</button>
      <div class="dropdown">
          <button class="ghost">Menu ▼</button>
          <div class="dropdown-content">
              <a @click="emit('check-updates')">Check Updates</a>
              <a @click="emit('export-diagnostics')">Export Diagnostics</a>
              <a @click="emit('open-about')">About</a>
          </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 2px solid var(--border-color);
  padding-bottom: 16px;
  margin-bottom: 24px;
}

.subtitle {
  margin: 4px 0 0;
  color: var(--hint-color);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.filter {
  display: flex;
  gap: 8px;
}

.filter button[data-active="true"] {
  background: var(--border-color);
  color: var(--bg-color); /* Invert for contrast */
}

button {
  border: 2px solid var(--border-color);
  background: var(--ghost-bg);
  color: var(--text-color);
  padding: 6px 10px;
  font-weight: 600;
  cursor: pointer;
}

.dropdown {
  position: relative;
  display: inline-block;
}

.dropdown-content {
  display: none;
  position: absolute;
  right: 0;
  background-color: var(--card-bg);
  min-width: 160px;
  box-shadow: 0px 8px 16px 0px rgba(0,0,0,0.2);
  z-index: 1;
  border: 1px solid var(--border-color);
}

.dropdown-content a {
  color: var(--text-color);
  padding: 12px 16px;
  text-decoration: none;
  display: block;
  cursor: pointer;
  font-size: 13px;
}

.dropdown-content a:hover {background-color: var(--code-bg);}

.dropdown:hover .dropdown-content {display: block;}
</style>
