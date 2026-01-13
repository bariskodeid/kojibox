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
  <header class="flex flex-col md:flex-row items-start md:items-center justify-between border-b-2 border-[var(--border-color)] bg-[var(--card-bg)] p-6 mb-0 sticky top-0 z-50 shadow-md">
    <div class="mb-4 md:mb-0">
      <h1 class="text-3xl font-black uppercase tracking-tighter m-0 leading-none">
        Kojibox<span class="text-[var(--accent-color)]">_</span>
      </h1>
      <p class="font-mono text-xs text-[var(--secondary-color)] uppercase tracking-widest mt-1">
        Portable Dev Environment v0.1.0
      </p>
    </div>
    
    <div class="flex flex-wrap gap-2 items-center">
      <div class="flex mr-2 bg-[var(--code-bg)] p-1 border border-[var(--border-color)]">
        <button
          class="px-3 py-1 text-xs font-bold uppercase transition-colors"
          :class="logFilter === 'all' ? 'bg-[var(--text-color)] text-[var(--bg-color)]' : 'text-[var(--text-color)] hover:bg-[var(--secondary-color)]'"
          @click="emit('update:logFilter', 'all')"
        >
          ALL LOGS
        </button>
        <button
          class="px-3 py-1 text-xs font-bold uppercase transition-colors"
          :class="logFilter === 'error' ? 'bg-[var(--error-color)] text-white' : 'text-[var(--text-color)] hover:bg-[var(--secondary-color)]'"
          @click="emit('update:logFilter', 'error')"
        >
          ERRORS
        </button>
      </div>

      <button class="btn btn-ghost border-2 border-[var(--border-color)] px-3" @click="toggleDark" title="Toggle Theme">
        {{ isDark ? '☀' : '☾' }}
      </button>
      
      <button class="btn" @click="emit('open-terminal')">
        <span class="font-mono">>_</span> TERM
      </button>
      
      <button class="btn" @click="emit('refresh')">
        REFRESH
      </button>
      
      <div class="dropdown relative group">
        <button class="btn">MENU ▼</button>
        <div class="absolute right-0 mt-0 w-48 bg-[var(--card-bg)] border-2 border-[var(--border-color)] shadow-[4px_4px_0_var(--border-color)] hidden group-hover:block z-50">
            <a class="block px-4 py-3 text-xs font-bold uppercase border-b border-[var(--border-color)] hover:bg-[var(--accent-color)] hover:text-white cursor-pointer transition-colors" @click="emit('check-updates')">Check Updates</a>
            <a class="block px-4 py-3 text-xs font-bold uppercase border-b border-[var(--border-color)] hover:bg-[var(--accent-color)] hover:text-white cursor-pointer transition-colors" @click="emit('export-diagnostics')">Export Diag</a>
            <a class="block px-4 py-3 text-xs font-bold uppercase hover:bg-[var(--accent-color)] hover:text-white cursor-pointer transition-colors" @click="emit('open-about')">About</a>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
/* Scoped styles removed in favor of Tailwind classes */
</style>
