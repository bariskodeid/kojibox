<script setup lang="ts">
import { ref } from "vue";
import type { DomainMapping, ProxyRule, CertMeta, TrustResult } from "../types";

defineProps<{
  domains: DomainMapping[];
  proxyRules: ProxyRule[];
  certs: CertMeta[];
  trustResult: TrustResult | null;
}>();

const emit = defineEmits<{
  (e: "save-domain", mapping: DomainMapping): void;
  (e: "delete-domain", domain: string): void;
  (e: "apply-hosts"): void;
  (e: "rollback-hosts"): void;
  
  (e: "save-proxy", rule: ProxyRule): void;
  (e: "delete-proxy", index: number): void;
  (e: "apply-proxy"): void;
  
  (e: "generate-cert", domains: string[]): void;
  (e: "trust-cert", path: string, os: boolean, apply: boolean): void;
}>();

const toolingError = ref<string | null>(null);
const newDomain = ref<DomainMapping>({
  domain: "",
  projectId: "",
  targetPort: 3000,
});
const newRule = ref<ProxyRule>({
  host: "",
  path: "/",
  target: "http://127.0.0.1:3000",
  tls: false,
});
const certDomainsInput = ref("");

function onSaveDomain() {
  toolingError.value = null;
  if (!newDomain.value.domain.trim()) {
    toolingError.value = "domain is required";
    return;
  }
  emit("save-domain", { ...newDomain.value });
  newDomain.value = { domain: "", projectId: "", targetPort: 3000 };
}

function isValidUrl(str: string) {
  try {
    new URL(str);
    return true;
  } catch {
    return false;
  }
}

function onSaveProxy() {
  toolingError.value = null;
  if (!isValidUrl(newRule.value.target)) {
      toolingError.value = "invalid target URL";
      return;
  }
  emit("save-proxy", { ...newRule.value });
  newRule.value = { host: "", path: "/", target: "http://127.0.0.1:3000", tls: false };
}

function onGenerateCert() {
  toolingError.value = null;
  const domains = certDomainsInput.value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
  if (!domains.length) {
    toolingError.value = "domains required";
    return;
  }
  emit("generate-cert", domains);
  certDomainsInput.value = "";
}

</script>

<template>
  <div class="space-y-6">
    <!-- Domains Section -->
    <section class="card">
      <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
          <h3 class="text-lg font-black uppercase">Domain Routing</h3>
          <span class="tech-label">VHOSTS</span>
      </div>

      <div class="border-2 border-[var(--border-color)] p-3 mb-4 bg-[var(--code-bg)]">
        <div class="grid grid-cols-12 gap-2 mb-2">
            <div class="col-span-5">
                <input v-model="newDomain.domain" placeholder="domain.test" class="input font-mono text-xs" />
            </div>
            <div class="col-span-4">
                <input v-model="newDomain.projectId" placeholder="project_id" class="input font-mono text-xs uppercase" />
            </div>
            <div class="col-span-3">
                <input v-model.number="newDomain.targetPort" type="number" placeholder="8000" class="input font-mono text-xs" />
            </div>
        </div>
        <div class="grid grid-cols-3 gap-2">
            <button class="btn btn-primary" @click="onSaveDomain">ADD ROUTE</button>
            <button class="btn" @click="emit('apply-hosts')">SYNC HOSTS</button>
            <button class="btn" @click="emit('rollback-hosts')">ROLLBACK</button>
        </div>
      </div>

      <ul class="space-y-1">
        <li v-for="mapping in domains" :key="mapping.domain" class="flex items-center justify-between p-2 border-b border-[var(--border-color)] text-xs font-mono hover:bg-[var(--code-bg)]">
          <div>
              <span class="font-bold text-[var(--accent-color)]">{{ mapping.domain }}</span>
              <span class="text-[var(--secondary-color)] mx-2">-></span>
              <span>{{ mapping.projectId }} :{{ mapping.targetPort }}</span>
          </div>
          <button class="text-[var(--error-color)] font-bold hover:underline" @click="emit('delete-domain', mapping.domain)">DEL</button>
        </li>
      </ul>
    </section>

    <!-- Proxy Rules Section -->
    <section class="card">
      <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
          <h3 class="text-lg font-black uppercase">Reverse Proxy</h3>
          <span class="tech-label">INGRESS</span>
      </div>

      <div class="border-2 border-[var(--border-color)] p-3 mb-4 bg-[var(--code-bg)]">
        <div class="grid grid-cols-1 gap-2 mb-2">
            <div class="grid grid-cols-2 gap-2">
                <input v-model="newRule.host" placeholder="host" class="input font-mono text-xs" />
                <input v-model="newRule.path" placeholder="/path" class="input font-mono text-xs" />
            </div>
            <div class="flex gap-2">
                <input v-model="newRule.target" placeholder="http://target:port" class="input font-mono text-xs flex-1" />
                <label class="flex items-center gap-1 border-2 border-[var(--border-color)] px-2 bg-white">
                  <input type="checkbox" v-model="newRule.tls" class="w-3 h-3" />
                  <span class="text-[10px] font-bold">TLS</span>
                </label>
            </div>
        </div>
        <div class="grid grid-cols-2 gap-2">
             <button class="btn btn-primary" @click="onSaveProxy">ADD RULE</button>
             <button class="btn" @click="emit('apply-proxy')">APPLY CONFIG</button>
        </div>
      </div>

      <ul class="space-y-1">
        <li v-for="(rule, index) in proxyRules" :key="index" class="flex items-center justify-between p-2 border-b border-[var(--border-color)] text-xs font-mono hover:bg-[var(--code-bg)]">
          <div class="truncate mr-2">
            <span class="font-bold">{{ rule.host }}</span>{{ rule.path }}
            <span class="text-[var(--secondary-color)]">-></span> {{ rule.target }}
            <span v-if="rule.tls" class="ml-2 badge bg-[var(--accent-color)] text-white border-transparent">TLS</span>
          </div>
          <button class="text-[var(--error-color)] font-bold hover:underline" @click="emit('delete-proxy', index)">DEL</button>
        </li>
      </ul>
    </section>

    <!-- Certs Section -->
    <section class="card">
      <div class="border-b-2 border-[var(--border-color)] pb-2 mb-4 flex justify-between items-center">
          <h3 class="text-lg font-black uppercase">TLS Certificates</h3>
          <span class="tech-label">MKCERT</span>
      </div>

      <div class="flex gap-2 mb-4">
        <input v-model="certDomainsInput" placeholder="domains (comma separated)" class="input font-mono text-xs" />
        <button class="btn" @click="onGenerateCert">GENERATE</button>
      </div>

      <ul class="space-y-2">
        <li v-for="cert in certs" :key="cert.path" class="border border-[var(--border-color)] p-2 bg-[var(--code-bg)]">
          <div class="flex justify-between items-center mb-2">
              <strong class="text-xs font-mono">{{ cert.name }}</strong>
              <span class="text-[9px] text-[var(--secondary-color)]">EXP: {{ cert.expiresAt }}</span>
          </div>
          <div class="grid grid-cols-3 gap-1">
            <button class="btn btn-sm px-1 py-0.5 text-[9px]" @click="emit('trust-cert', cert.path, false, false)">INFO</button>
            <button class="btn btn-sm px-1 py-0.5 text-[9px]" @click="emit('trust-cert', cert.path, true, false)">CMD</button>
            <button class="btn btn-sm px-1 py-0.5 text-[9px] border-[var(--success-color)] text-[var(--success-color)]" @click="emit('trust-cert', cert.path, true, true)">TRUST</button>
          </div>
        </li>
      </ul>
      
      <div v-if="trustResult" class="mt-4 p-2 border border-[var(--border-color)] bg-[var(--card-bg)] shadow-md">
        <div class="font-bold text-xs uppercase mb-1 border-b border-[var(--border-color)] pb-1">Command Output</div>
        <pre class="text-[10px] font-mono whitespace-pre-wrap text-[var(--secondary-color)] bg-[var(--code-bg)] p-2 mb-2 select-all">{{ trustResult.command }}</pre>
        <ul v-if="trustResult.notes.length" class="text-[10px] list-disc pl-4 mb-2">
            <li v-for="n in trustResult.notes" :key="n">{{ n }}</li>
        </ul>
        <p v-if="trustResult.error" class="text-[var(--error-color)] text-xs font-bold">{{ trustResult.error }}</p>
        <p v-if="trustResult.applied" class="text-[var(--success-color)] text-xs font-bold">SUCCESS: Certificate added to trust store.</p>
      </div>
    </section>
    
    <p v-if="toolingError" class="error mt-4 font-mono text-xs">{{ toolingError }}</p>

  </div>
</template>

<style scoped>
/* Scoped styles removed */
</style>
