// Espelho dos DTOs serializados pelo backend (src-tauri/src/domain/models.rs e application/*).

export type ProviderKind = "r2" | "s3" | "custom";

export interface CloudConfig {
  provider: ProviderKind;
  accountId: string;
  region: string;
  endpoint: string;
  bucket: string;
  prefix: string;
  accessKeyId: string;
  secretAccessKey: string;
}

export interface Pcsx2Settings {
  enabled: boolean;
  autoSync: boolean;
  dataDir: string | null;
}

export interface AppSettings {
  cloud: CloudConfig | null;
  pcsx2: Pcsx2Settings;
  backupsToKeep: number;
}

export interface ConnectionStep {
  name: string;
  ok: boolean;
  durationMs: number;
  error: string | null;
}

export interface ConnectionReport {
  ok: boolean;
  endpoint: string;
  bucket: string;
  steps: ConnectionStep[];
}

export type UnitStatus = "synced" | "pending" | "conflict" | "deferred" | "error" | "unknown";
export type UnitKind = "file" | "directory";
export type Resolution = "keepLocal" | "keepCloud";

export interface UnitView {
  id: string;
  sourceId: string;
  group: string;
  title: string;
  subtitle: string | null;
  tags: string[];
  kind: UnitKind;
  existsLocally: boolean;
  size: number;
  modifiedAt: string | null;
  status: UnitStatus;
  message: string | null;
  lastSyncedAt: string | null;
  remoteMachine: string | null;
  /** Conflito porque o save foi excluído em outra máquina. */
  remoteDeleted: boolean;
  /** Caminho local da capa (pasta de capas do emulador). */
  coverPath: string | null;
}

export interface SourceInfo {
  id: string;
  name: string;
  detected: boolean;
  dataDir: string | null;
  savesDir: string | null;
  configFound: boolean;
  portable: boolean;
  cards: string[];
  message: string | null;
}

export interface SourceOverview {
  info: SourceInfo;
  enabled: boolean;
  autoSync: boolean;
  scanError: string | null;
  units: UnitView[];
}

export type ActivityKind = "upload" | "download" | "delete" | "conflict" | "deferred" | "error" | "info";

export interface ActivityEntry {
  id: string;
  at: string;
  sourceId: string;
  unitId: string | null;
  title: string;
  kind: ActivityKind;
  message: string;
}

export type SyncTrigger = "manual" | "watcher" | "startup" | "retry" | "resolution" | "deletion";
export type UnitPhase = "checking" | "uploading" | "downloading" | "deleting" | "done";

export interface SyncSummary {
  sourceId: string;
  uploaded: number;
  downloaded: number;
  deleted: number;
  inSync: number;
  conflicts: number;
  deferred: number;
  errors: number;
  finishedAt: string | null;
  error: string | null;
}

export type SyncEvent =
  | { type: "started"; sourceId: string; trigger: SyncTrigger; total: number }
  | {
      type: "unit";
      sourceId: string;
      unitId: string;
      title: string;
      phase: UnitPhase;
      status: UnitStatus | null;
      index: number;
      total: number;
    }
  | { type: "finished"; summary: SyncSummary }
  | { type: "localChanged"; sourceId: string };
