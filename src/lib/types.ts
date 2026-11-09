export type DeviceState =
  | "active"
  | "unplugged"
  | "disabled"
  | "notpresent"
  | "unknown";

export interface AudioDevice {
  id: string;
  name: string;
  displayName: string;
  customName: string | null;
  deviceDesc: string;
  interfaceName: string;
  state: DeviceState | string;
  stateLabel: string;
  isDefault: boolean;
  isDefaultCommunication: boolean;
  volumePercent: number;
  muted: boolean;
  formFactor: string;
  formFactorLabel: string;
}

export interface AudioSnapshot {
  devices: AudioDevice[];
  defaultOutput: string | null;
  defaultOutputFormFactor: string;
  permission: string;
  monitorStatus: string;
  capturedAt: number;
}

export interface MeterPayload {
  levels: Record<string, number>;
}

export interface ListenState {
  active: boolean;
  deviceId: string | null;
  deviceName: string | null;
  volume: number;
  muted: boolean;
  outputName: string | null;
  peak: number;
  feedbackGuard: boolean;
  status: "idle" | "starting" | "listening" | "stopping" | "error" | string;
  error: string | null;
  startedAt: number | null;
}

export interface Settings {
  theme: "dark" | "light" | "system";
  listenVolume: number;
  feedbackGuard: boolean;
  showInactive: boolean;
  reduceMotion: boolean;
  compactCards: boolean;
}

export interface AppInfo {
  version: string;
  identifier: string;
  dataPath: string;
}

export interface AppError {
  code: string;
  message: string;
}

export type NoticeKind = "info" | "success" | "warning" | "error";

export interface Notice {
  kind: NoticeKind;
  message: string;
}

export type PageId = "dashboard" | "microphones" | "settings";
