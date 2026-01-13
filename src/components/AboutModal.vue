<script setup lang="ts">
defineProps<{
  show: boolean;
  version: string;
}>();

const emit = defineEmits<{
  (e: "close"): void;
}>();

const licenses = [
    { name: "PHP", url: "https://www.php.net/license/" },
    { name: "MariaDB", url: "https://mariadb.com/kb/en/mariadb-license/" },
    { name: "PostgreSQL", url: "https://www.postgresql.org/about/licence/" },
    { name: "Node.js", url: "https://github.com/nodejs/node/blob/master/LICENSE" },
    { name: "Mailpit", url: "https://github.com/axllent/mailpit/blob/main/LICENSE" },
];
</script>

<template>
  <div v-if="show" class="fixed inset-0 bg-black/50 grid place-items-center z-[2000]" @click="emit('close')">
    <div class="card w-full max-w-md shadow-[8px_8px_0_var(--border-color)]" @click.stop>
      <div class="border-b-2 border-[var(--border-color)] pb-4 mb-4">
          <h2 class="text-3xl font-black uppercase">KOJIBOX</h2>
          <span class="tech-label">Portable Environment</span>
      </div>
      
      <div class="grid grid-cols-2 gap-4 mb-6 text-xs font-mono">
          <div>
              <span class="block text-[var(--secondary-color)]">VERSION</span>
              <strong>{{ version }}</strong>
          </div>
          <div>
              <span class="block text-[var(--secondary-color)]">BUILD</span>
              <strong>BETA</strong>
          </div>
      </div>
      
      <h3 class="text-sm font-bold uppercase mb-2 border-b border-[var(--border-color)]">Licenses</h3>
      <ul class="mb-6 space-y-1">
          <li v-for="l in licenses" :key="l.name">
              <a :href="l.url" target="_blank" class="text-xs font-mono hover:bg-[var(--accent-color)] hover:text-white px-1 -ml-1 transition-colors">
                  {{ l.name }} ↗
              </a>
          </li>
      </ul>
      
      <div class="flex justify-end">
          <button class="btn" @click="emit('close')">ACKNOWLEDGE</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Scoped styles replaced by Tailwind classes */
</style>
