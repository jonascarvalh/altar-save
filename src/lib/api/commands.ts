import { invoke } from "@tauri-apps/api/core";
import { toAppError } from "./errors";
import type {
  ActivityEntry,
  AppSettings,
  CloudConfig,
  ConnectionReport,
  Pcsx2Settings,
  Resolution,
  SourceInfo,
  SourceOverview,
  SyncSummary,
} from "./types";

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toAppError(error);
  }
}

/** Wrappers tipados dos comandos Tauri (ver src-tauri/src/commands.rs). */
export const api = {
  getSettings: () => call<AppSettings>("get_settings"),
  saveCloudConfig: (config: CloudConfig) => call<AppSettings>("save_cloud_config", { config }),
  clearCloudConfig: () => call<AppSettings>("clear_cloud_config"),
  testConnection: (config: CloudConfig) => call<ConnectionReport>("test_connection", { config }),

  updatePcsx2Settings: (settings: Pcsx2Settings) => call<AppSettings>("update_pcsx2_settings", { settings }),
  detectPcsx2: (dataDir: string | null) => call<SourceInfo>("detect_pcsx2", { dataDir }),
  pcsx2Candidates: () => call<SourceInfo[]>("pcsx2_candidates"),

  getLibrary: () => call<SourceOverview[]>("get_library"),
  syncNow: (sourceId: string) => call<SyncSummary>("sync_now", { sourceId }),
  resolveConflict: (unitId: string, resolution: Resolution) =>
    call<SyncSummary>("resolve_conflict", { unitId, resolution }),
  deleteSave: (unitId: string) => call<SyncSummary>("delete_save", { unitId }),

  getActivity: (limit?: number) => call<ActivityEntry[]>("get_activity", { limit }),
  getWatchStatus: () => call<[string, boolean][]>("get_watch_status"),

  openBackupsFolder: () => call<void>("open_backups_folder"),
  openSavesFolder: (sourceId: string) => call<void>("open_saves_folder", { sourceId }),
};
