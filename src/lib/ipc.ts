import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppInfo,
  AudioSnapshot,
  ListenState,
  MeterPayload,
  Notice,
  Settings,
} from "./types";

export const api = {
  getDevices: () => invoke<AudioSnapshot>("get_devices"),
  refreshDevices: () => invoke<AudioSnapshot>("refresh_devices"),
  renameDevice: (deviceId: string, customName: string | null) =>
    invoke<AudioSnapshot>("rename_device", { deviceId, customName }),
  resetCustomNames: () => invoke<AudioSnapshot>("reset_custom_names"),
  getListenState: () => invoke<ListenState>("get_listen_state"),
  startListen: (deviceId: string) =>
    invoke<ListenState>("start_listen", { deviceId }),
  stopListen: () => invoke<ListenState>("stop_listen"),
  setListenVolume: (value: number) =>
    invoke<ListenState>("set_listen_volume", { value }),
  setListenMute: (muted: boolean) =>
    invoke<ListenState>("set_listen_mute", { muted }),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) =>
    invoke<Settings>("update_settings", { settings }),
  getPermission: () => invoke<string>("get_permission"),
  openMicrophoneSettings: () => invoke<void>("open_microphone_settings"),
  getAppInfo: () => invoke<AppInfo>("get_app_info"),
  windowMinimize: () => invoke<void>("window_minimize"),
  windowToggleMaximize: () => invoke<boolean>("window_toggle_maximize"),
  windowClose: () => invoke<void>("window_close"),
};

export const events = {
  onDevices: (handler: (snapshot: AudioSnapshot) => void): Promise<UnlistenFn> =>
    listen<AudioSnapshot>("audio:devices", (event) => handler(event.payload)),
  onListenState: (handler: (state: ListenState) => void): Promise<UnlistenFn> =>
    listen<ListenState>("listen:state", (event) => handler(event.payload)),
  onNotice: (handler: (notice: Notice) => void): Promise<UnlistenFn> =>
    listen<Notice>("audio:notice", (event) => handler(event.payload)),
  onMeter: (handler: (payload: MeterPayload) => void): Promise<UnlistenFn> =>
    listen<MeterPayload>("audio:meter", (event) => handler(event.payload)),
};

export function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object") {
    const candidate = error as { message?: unknown; code?: unknown };
    if (typeof candidate.message === "string") return candidate.message;
    if (typeof candidate.code === "string") return candidate.code;
  }
  return "Une erreur inattendue est survenue.";
}
