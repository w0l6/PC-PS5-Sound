import type { UnlistenFn } from "@tauri-apps/api/event";
import { events } from "./ipc";

type MeterListener = (levels: Record<string, number>) => void;

const listeners = new Set<MeterListener>();
let unlisten: UnlistenFn | null = null;
let starting: Promise<void> | null = null;

async function start(): Promise<void> {
  if (unlisten || starting) {
    if (starting) await starting;
    return;
  }
  starting = events
    .onMeter((payload) => {
      for (const listener of listeners) {
        listener(payload.levels);
      }
    })
    .then((stop) => {
      unlisten = stop;
    })
    .finally(() => {
      starting = null;
    });
  await starting;
}

export function subscribeMeter(listener: MeterListener): () => void {
  listeners.add(listener);
  void start();
  return () => {
    listeners.delete(listener);
  };
}
