import { create } from "zustand";
import { api } from "@/lib/api/commands";
import type { AppSettings, CloudConfig, Pcsx2Settings } from "@/lib/api/types";

interface SettingsState {
  settings: AppSettings | null;
  loading: boolean;
  load: () => Promise<void>;
  saveCloud: (config: CloudConfig) => Promise<AppSettings>;
  clearCloud: () => Promise<AppSettings>;
  updatePcsx2: (settings: Pcsx2Settings) => Promise<AppSettings>;
}

export const useSettings = create<SettingsState>((set) => {
  const apply = (settings: AppSettings) => {
    set({ settings });
    return settings;
  };

  return {
    settings: null,
    loading: false,
    load: async () => {
      set({ loading: true });
      try {
        apply(await api.getSettings());
      } finally {
        set({ loading: false });
      }
    },
    saveCloud: async (config) => apply(await api.saveCloudConfig(config)),
    clearCloud: async () => apply(await api.clearCloudConfig()),
    updatePcsx2: async (pcsx2) => apply(await api.updatePcsx2Settings(pcsx2)),
  };
});
