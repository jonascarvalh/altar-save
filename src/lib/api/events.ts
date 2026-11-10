import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SyncEvent } from "./types";

export const SYNC_EVENT = "sync://event";

export function onSyncEvent(handler: (event: SyncEvent) => void): Promise<UnlistenFn> {
  return listen<SyncEvent>(SYNC_EVENT, (e) => handler(e.payload));
}
