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
  <div>
    <!-- Domains Section -->
    <section class="notice">
      <h3>Domains</h3>
      <div class="project-form">
        <input v-model="newDomain.domain" placeholder="domain (app.test)" />
        <input v-model="newDomain.projectId" placeholder="project id" />
        <input v-model.number="newDomain.targetPort" type="number" placeholder="port" />
        <div class="actions">
            <button class="ghost" @click="onSaveDomain">Save Mapping</button>
            <button class="ghost" @click="emit('apply-hosts')">Apply Hosts</button>
            <button class="ghost" @click="emit('rollback-hosts')">Rollback Hosts</button>
        </div>
      </div>
      <ul class="list">
        <li v-for="mapping in domains" :key="mapping.domain">
          <strong>{{ mapping.domain }}</strong> -> {{ mapping.projectId }} :{{ mapping.targetPort }}
          <button class="ghost small" @click="emit('delete-domain', mapping.domain)">Delete</button>
        </li>
      </ul>
    </section>

    <!-- Proxy Rules Section -->
    <section class="notice">
      <h3>Proxy Rules</h3>
      <div class="project-form">
        <input v-model="newRule.host" placeholder="host (app.test)" />
        <input v-model="newRule.path" placeholder="/path" />
        <input v-model="newRule.target" placeholder="http://127.0.0.1:3000" />
        <label class="inline">
          <input type="checkbox" v-model="newRule.tls" />
          tls
        </label>
        <div class="actions">
             <button class="ghost" @click="onSaveProxy">Add Rule</button>
             <button class="ghost" @click="emit('apply-proxy')">Apply Rules</button>
        </div>
      </div>
      <ul class="list">
        <li v-for="(rule, index) in proxyRules" :key="index">
          <strong>{{ rule.host }}</strong>{{ rule.path }} -> {{ rule.target }}
          <span class="hint" v-if="rule.tls">tls</span>
          <button class="ghost small" @click="emit('delete-proxy', index)">Delete</button>
        </li>
      </ul>
    </section>

    <!-- Certs Section -->
    <section class="notice">
      <h3>Certificates</h3>
      <div class="project-form">
        <input v-model="certDomainsInput" placeholder="domains (a.test,b.test)" />
        <button class="ghost" @click="onGenerateCert">Generate</button>
      </div>
      <ul class="list">
        <li v-for="cert in certs" :key="cert.path">
          <strong>{{ cert.name }}</strong> (exp {{ cert.expiresAt }})
          <div class="cert-actions">
            <button class="ghost small" @click="emit('trust-cert', cert.path, false, false)">Instructions</button>
            <button class="ghost small" @click="emit('trust-cert', cert.path, true, false)">OS Command</button>
            <button class="ghost small" @click="emit('trust-cert', cert.path, true, true)">Apply Now</button>
          </div>
        </li>
      </ul>
      <div v-if="trustResult" class="notice sub-notice">
        <p><strong>Command:</strong> {{ trustResult.command }}</p>
        <p v-if="trustResult.notes.length">
          <strong>Notes:</strong> {{ trustResult.notes.join(' ') }}
        </p>
        <p v-if="trustResult.error" class="error-inline">{{ trustResult.error }}</p>
        <p v-if="trustResult.applied">Applied successfully.</p>
      </div>
    </section>
    
    <p v-if="toolingError" class="error-inline">{{ toolingError }}</p>

  </div>
</template>

<style scoped>
.notice {
  background: #e8f4e8;
  border: 1px solid #6fb56f;
  padding: 12px 16px;
  margin-bottom: 16px;
}

.sub-notice {
    margin-top: 12px;
    background: #fff;
    border: 1px dashed #6fb56f;
}

.project-form {
  display: grid;
  gap: 6px;
  margin-top: 8px;
}

.inline {
  display: flex;
  align-items: center;
  gap: 8px;
}

.list {
  margin: 16px 0 0;
  padding-left: 16px;
}

.list li {
    margin-bottom: 8px;
}

.actions {
    display: flex;
    gap: 8px;
    margin-top: 8px;
}

.cert-actions {
    display: inline-flex;
    gap: 4px;
    margin-left: 8px;
}

input {
  border: 2px solid #1b1b1b;
  padding: 6px 10px;
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

.ghost { background: #ffffff; }

.error-inline {
  background: #ffe2e2;
  border: 1px solid #d96a6a;
  padding: 6px 8px;
  font-size: 12px;
}

.hint {
  font-size: 11px;
  text-transform: uppercase;
  color: #6b6b6b;
  margin-left: 8px;
}
</style>
