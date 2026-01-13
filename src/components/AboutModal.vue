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
  <div v-if="show" class="modal-backdrop" @click="emit('close')">
    <div class="modal" @click.stop>
      <h2>About Kojibox</h2>
      <p>Version {{ version }}</p>
      <p>Portable Web Development Environment</p>
      
      <h3>Third Party Licenses</h3>
      <ul>
          <li v-for="l in licenses" :key="l.name">
              <a :href="l.url" target="_blank">{{ l.name }}</a>
          </li>
      </ul>
      
      <div class="actions">
          <button class="ghost" @click="emit('close')">Close</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: grid;
  place-items: center;
  z-index: 2000;
}

.modal {
  background: var(--card-bg);
  color: var(--text-color);
  border: 2px solid var(--border-color);
  padding: 24px;
  max-width: 400px;
  width: 100%;
}

h2 { margin-top: 0; }

ul { padding-left: 20px; }
li { margin-bottom: 4px; }
a { color: var(--accent-color); }

.actions {
    margin-top: 20px;
    text-align: right;
}

button {
  border: 2px solid var(--border-color);
  background: var(--ghost-bg);
  color: var(--text-color);
  padding: 6px 12px;
  font-weight: 600;
  cursor: pointer;
}
</style>
