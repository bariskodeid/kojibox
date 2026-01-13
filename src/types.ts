export type ServiceState = {
  id: string;
  state: string;
  pid: number | null;
  lastError: string | null;
  lastUpdated: string;
};

export type LogEntry = {
  ts: string;
  level: string;
  message: string;
  service?: string;
};

export type InstallerStatus = {
  phase: string;
  progress: number;
};

export type MetricsSnapshot = {
  ts: string;
  uptimeSec: number;
  portsInUse: number[];
  cpuPercent: number;
  memMB: number;
};

export type AppConfig = {
  schemaVersion: number;
  installPath: string;
  updateChannel: string;
  telemetryOptIn: boolean;
  updateFeedUrl: string;
  updatePublicKeys: string[];
};

export type ProjectConfig = {
  schemaVersion: number;
  id: string;
  name: string;
  path: string;
  domain: string;
  stack: string;
  overrides: Record<string, string>;
};

export type ServiceConfig = {
  schemaVersion: number;
  id: string;
  enabled: boolean;
  ports: Record<string, number>;
  env: Record<string, string>;
  args: string[];
};

export type DomainMapping = {
  domain: string;
  projectId: string;
  targetPort: number;
};

export type ProxyRule = {
  host: string;
  path: string;
  target: string;
  tls: boolean;
};

export type CertMeta = {
  name: string;
  path: string;
  expiresAt: string;
};

export type TrustResult = {
  applied: boolean;
  command: string;
  notes: string[];
  error: string | null;
};

export type ServiceBinary = {
  name: string;
  version: string;
  os: string;
  arch: string;
  checksum: string;
  size: number;
  binPath: string;
  defaultPorts: { name: string; port: number; protocol: string }[];
  env: Record<string, string>;
  args: string[];
};

export type RuntimeManifest = {
  version: string;
  services: ServiceBinary[];
  bundle: { createdAt: string; source: string; signature: string };
};

export type RuntimeDownloadStatus = {
  phase: string;
  progress: number;
  service: string | null;
  error: string | null;
};
