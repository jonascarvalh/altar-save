import { create } from "zustand";
import { api } from "@/lib/api/commands";
import type { Resolution, SourceOverview, SyncEvent, SyncSummary, UnitPhase } from "@/lib/api/types";

interface RunProgress {
  total: number;
  index: number;
  title: string | null;
  phase: UnitPhase | null;
}

interface SyncState {
  library: SourceOverview[];
  libraryLoaded: boolean;
  /** Sincronizações em andamento, por fonte. */
  running: Record<string, RunProgress>;
  /** Fase atual de cada save durante o sync. */
  unitPhases: Record<string, UnitPhase>;
  lastSummary: Record<string, SyncSummary>;
  watching: Record<string, boolean>;
  /** Incrementado a cada sync concluído; telas observam para recarregar dados. */
  revision: number;

  loadLibrary: () => Promise<void>;
  loadWatchStatus: () => Promise<void>;
  syncNow: (sourceId: string) => Promise<SyncSummary>;
  resolve: (unitId: string, resolution: Resolution) => Promise<SyncSummary>;
  deleteSave: (unitId: string) => Promise<SyncSummary>;
  handleEvent: (event: SyncEvent) => void;
}

export const useSync = create<SyncState>((set, get) => ({
  library: [],
  libraryLoaded: false,
  running: {},
  unitPhases: {},
  lastSummary: {},
  watching: {},
  revision: 0,

  loadLibrary: async () => {
    const library = await api.getLibrary();
    set({ library, libraryLoaded: true });
  },

  loadWatchStatus: async () => {
    const status = await api.getWatchStatus();
    set({ watching: Object.fromEntries(status) });
  },

  syncNow: (sourceId) => api.syncNow(sourceId),
  resolve: (unitId, resolution) => api.resolveConflict(unitId, resolution),
  deleteSave: (unitId) => api.deleteSave(unitId),

  handleEvent: (event) => {
    switch (event.type) {
      case "started":
        set((s) => ({
          running: { ...s.running, [event.sourceId]: { total: event.total, index: 0, title: null, phase: null } },
        }));
        break;
      case "unit":
        set((s) => {
          const unitPhases = { ...s.unitPhases };
          if (event.phase === "done") delete unitPhases[event.unitId];
          else unitPhases[event.unitId] = event.phase;
          return {
            unitPhases,
            running: {
              ...s.running,
              [event.sourceId]: { total: event.total, index: event.index, title: event.title, phase: event.phase },
            },
          };
        });
        break;
      case "finished": {
        const { sourceId } = event.summary;
        set((s) => {
          const running = { ...s.running };
          delete running[sourceId];
          return {
            running,
            unitPhases: {},
            lastSummary: { ...s.lastSummary, [sourceId]: event.summary },
            revision: s.revision + 1,
          };
        });
        void get().loadLibrary();
        break;
      }
      case "localChanged":
        void get().loadLibrary();
        break;
    }
  },
}));
