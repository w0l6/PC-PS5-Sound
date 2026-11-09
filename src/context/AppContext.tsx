import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { api, errorMessage, events } from "../lib/ipc";
import { clamp } from "../lib/useMeter";
import type {
  AppInfo,
  AudioDevice,
  AudioSnapshot,
  ListenState,
  NoticeKind,
  PageId,
  Settings,
} from "../lib/types";

interface ToastItem {
  id: number;
  kind: NoticeKind;
  message: string;
}

const defaultSettings: Settings = {
  theme: "dark",
  listenVolume: 80,
  feedbackGuard: true,
  showInactive: true,
  reduceMotion: false,
  compactCards: false,
};

const defaultListen: ListenState = {
  active: false,
  deviceId: null,
  deviceName: null,
  volume: defaultSettings.listenVolume,
  muted: false,
  outputName: null,
  peak: 0,
  feedbackGuard: true,
  status: "idle",
  error: null,
  startedAt: null,
};

interface AppContextValue {
  snapshot: AudioSnapshot | null;
  devices: AudioDevice[];
  listen: ListenState;
  settings: Settings;
  permission: string;
  appInfo: AppInfo | null;
  loading: boolean;
  refreshing: boolean;
  busy: string | null;
  page: PageId;
  toasts: ToastItem[];
  renameTarget: AudioDevice | null;
  setPage: (page: PageId) => void;
  setRenameTarget: (device: AudioDevice | null) => void;
  dismissToast: (id: number) => void;
  pushToast: (kind: NoticeKind, message: string) => void;
  refresh: () => Promise<void>;
  renameDevice: (deviceId: string, name: string | null) => Promise<boolean>;
  resetCustomNames: () => Promise<void>;
  startListen: (deviceId: string) => Promise<void>;
  stopListen: () => Promise<void>;
  setVolume: (value: number) => void;
  toggleMute: () => void;
  updateSettings: (patch: Partial<Settings>) => Promise<void>;
  openMicrophoneSettings: () => Promise<void>;
  minimize: () => Promise<void>;
  toggleMaximize: () => Promise<boolean | null>;
  closeWindow: () => Promise<void>;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppProvider({ children }: { children: ReactNode }) {
  const [snapshot, setSnapshot] = useState<AudioSnapshot | null>(null);
  const [listen, setListen] = useState<ListenState>(defaultListen);
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [page, setPage] = useState<PageId>("dashboard");
  const [toasts, setToasts] = useState<ToastItem[]>([]);
  const [renameTarget, setRenameTarget] = useState<AudioDevice | null>(null);
  const [systemDark, setSystemDark] = useState(() =>
    typeof window !== "undefined" && window.matchMedia
      ? window.matchMedia("(prefers-color-scheme: dark)").matches
      : true,
  );

  const pushToast = useCallback((kind: NoticeKind, message: string) => {
    const id = Date.now() + Math.random();
    setToasts((current) => [...current.slice(-3), { id, kind, message }]);
    window.setTimeout(() => {
      setToasts((current) => current.filter((toast) => toast.id !== id));
    }, 5200);
  }, []);

  const dismissToast = useCallback((id: number) => {
    setToasts((current) => current.filter((toast) => toast.id !== id));
  }, []);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => setSystemDark(media.matches);
    media.addEventListener("change", handler);
    return () => media.removeEventListener("change", handler);
  }, []);

  useEffect(() => {
    const resolved =
      settings.theme === "system" ? (systemDark ? "dark" : "light") : settings.theme;
    document.documentElement.dataset.theme = resolved;
  }, [settings.theme, systemDark]);

  useEffect(() => {
    document.documentElement.dataset.motion = settings.reduceMotion ? "reduced" : "full";
  }, [settings.reduceMotion]);

  useEffect(() => {
    let disposed = false;
    const unlisteners: UnlistenFn[] = [];

    const setup = async () => {
      const [devicesOff, listenOff, noticeOff] = await Promise.all([
        events.onDevices((next) => setSnapshot(next)),
        events.onListenState((next) => setListen(next)),
        events.onNotice((notice) => pushToast(notice.kind, notice.message)),
      ]);
      if (disposed) {
        devicesOff();
        listenOff();
        noticeOff();
        return;
      }
      unlisteners.push(devicesOff, listenOff, noticeOff);
    };

    void setup();
    return () => {
      disposed = true;
      unlisteners.forEach((off) => off());
    };
  }, [pushToast]);

  useEffect(() => {
    let disposed = false;

    const load = async () => {
      const results = await Promise.allSettled([
        api.getDevices(),
        api.getListenState(),
        api.getSettings(),
        api.getAppInfo(),
      ]);
      if (disposed) return;

      const [snapshotResult, listenResult, settingsResult, infoResult] = results;
      if (snapshotResult.status === "fulfilled") {
        setSnapshot(snapshotResult.value);
      }
      if (listenResult.status === "fulfilled") {
        setListen(listenResult.value);
      }
      if (settingsResult.status === "fulfilled") {
        setSettings(settingsResult.value);
      }
      if (infoResult.status === "fulfilled") {
        setAppInfo(infoResult.value);
      }
      const failure = results.find((result) => result.status === "rejected");
      if (failure && failure.status === "rejected") {
        pushToast("error", errorMessage(failure.reason));
      }
      setLoading(false);
    };

    void load();
    return () => {
      disposed = true;
    };
  }, [pushToast]);

  const refresh = useCallback(async () => {
    setRefreshing(true);
    try {
      const next = await api.refreshDevices();
      setSnapshot(next);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      window.setTimeout(() => setRefreshing(false), 320);
    }
  }, [pushToast]);

  const renameDevice = useCallback(
    async (deviceId: string, name: string | null) => {
      try {
        const next = await api.renameDevice(deviceId, name);
        setSnapshot(next);
        setRenameTarget(null);
        pushToast(
          "success",
          name && name.trim()
            ? "Nom personnalisé enregistré."
            : "Nom Windows d'origine restauré.",
        );
        return true;
      } catch (error) {
        pushToast("error", errorMessage(error));
        return false;
      }
    },
    [pushToast],
  );

  const resetCustomNames = useCallback(async () => {
    try {
      const next = await api.resetCustomNames();
      setSnapshot(next);
      pushToast("success", "Tous les noms personnalisés ont été réinitialisés.");
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }, [pushToast]);

  const startListen = useCallback(
    async (deviceId: string) => {
      setBusy(deviceId);
      try {
        const next = await api.startListen(deviceId);
        setListen(next);
      } catch (error) {
        pushToast("error", errorMessage(error));
      } finally {
        setBusy(null);
      }
    },
    [pushToast],
  );

  const stopListen = useCallback(async () => {
    setBusy("stop");
    try {
      const next = await api.stopListen();
      setListen(next);
    } catch (error) {
      pushToast("error", errorMessage(error));
    } finally {
      setBusy(null);
    }
  }, [pushToast]);

  const setVolume = useCallback(
    (value: number) => {
      const next = clamp(Math.round(value), 0, 100);
      setListen((current) => ({ ...current, volume: next }));
      setSettings((current) => ({ ...current, listenVolume: next }));
      void api.setListenVolume(next).catch((error) => {
        pushToast("error", errorMessage(error));
      });
    },
    [pushToast],
  );

  const toggleMute = useCallback(() => {
    const next = !listen.muted;
    setListen((current) => ({ ...current, muted: next }));
    void api.setListenMute(next).catch((error) => {
      pushToast("error", errorMessage(error));
    });
  }, [listen.muted, pushToast]);

  const updateSettings = useCallback(
    async (patch: Partial<Settings>) => {
      const next: Settings = { ...settings, ...patch };
      setSettings(next);
      try {
        const saved = await api.updateSettings(next);
        setSettings(saved);
      } catch (error) {
        pushToast("error", errorMessage(error));
      }
    },
    [settings, pushToast],
  );

  const openMicrophoneSettings = useCallback(async () => {
    try {
      await api.openMicrophoneSettings();
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }, [pushToast]);

  const minimize = useCallback(async () => {
    try {
      await api.windowMinimize();
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }, [pushToast]);

  const toggleMaximize = useCallback(async (): Promise<boolean | null> => {
    try {
      return await api.windowToggleMaximize();
    } catch (error) {
      pushToast("error", errorMessage(error));
      return null;
    }
  }, [pushToast]);

  const closeWindow = useCallback(async () => {
    try {
      await api.windowClose();
    } catch (error) {
      pushToast("error", errorMessage(error));
    }
  }, [pushToast]);

  const devices = useMemo(() => snapshot?.devices ?? [], [snapshot]);
  const permission = snapshot?.permission ?? "unknown";

  const value = useMemo<AppContextValue>(
    () => ({
      snapshot,
      devices,
      listen,
      settings,
      permission,
      appInfo,
      loading,
      refreshing,
      busy,
      page,
      toasts,
      renameTarget,
      setPage,
      setRenameTarget,
      dismissToast,
      pushToast,
      refresh,
      renameDevice,
      resetCustomNames,
      startListen,
      stopListen,
      setVolume,
      toggleMute,
      updateSettings,
      openMicrophoneSettings,
      minimize,
      toggleMaximize,
      closeWindow,
    }),
    [
      snapshot,
      devices,
      listen,
      settings,
      permission,
      appInfo,
      loading,
      refreshing,
      busy,
      page,
      toasts,
      renameTarget,
      dismissToast,
      pushToast,
      refresh,
      renameDevice,
      resetCustomNames,
      startListen,
      stopListen,
      setVolume,
      toggleMute,
      updateSettings,
      openMicrophoneSettings,
      minimize,
      toggleMaximize,
      closeWindow,
    ],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}

export function useApp(): AppContextValue {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error("useApp doit être utilisé dans AppProvider");
  }
  return context;
}
