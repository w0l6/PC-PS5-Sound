import {
  AudioLines,
  LayoutDashboard,
  Mic,
  Moon,
  Radio,
  Settings2,
  Square,
  Sun,
} from "lucide-react";
import { useApp } from "../context/AppContext";
import { meterPercent } from "../lib/useMeter";
import type { PageId } from "../lib/types";
import { PermissionPill } from "./PermissionBanner";

const navItems: Array<{
  id: PageId;
  label: string;
  icon: typeof LayoutDashboard;
}> = [
  { id: "dashboard", label: "Tableau de bord", icon: LayoutDashboard },
  { id: "microphones", label: "Microphones", icon: Mic },
  { id: "settings", label: "Paramètres", icon: Settings2 },
];

export function Sidebar() {
  const {
    page,
    setPage,
    devices,
    listen,
    settings,
    updateSettings,
    stopListen,
    busy,
    appInfo,
  } = useApp();

  const activeCount = devices.filter((device) => device.state === "active").length;
  const nextTheme = settings.theme === "light" ? "dark" : "light";

  return (
    <aside className="sidebar">
      <div className="brand">
        <span className="brand-mark">
          <AudioLines size={18} />
        </span>
        <span className="brand-copy">
          <strong>Audio Manager</strong>
          <span>Windows audio console</span>
        </span>
      </div>

      <nav className="nav" aria-label="Navigation principale">
        <span className="nav-label">Principal</span>
        {navItems.map((item) => {
          const Icon = item.icon;
          return (
            <button
              key={item.id}
              type="button"
              className={`nav-item${page === item.id ? " is-active" : ""}`}
              onClick={() => setPage(item.id)}
              aria-current={page === item.id ? "page" : undefined}
            >
              <Icon size={17} />
              <span>{item.label}</span>
              {item.id === "microphones" && (
                <span className="nav-badge">
                  {activeCount}/{devices.length}
                </span>
              )}
              {item.id === "dashboard" && listen.active && <span className="nav-live-dot" />}
            </button>
          );
        })}
      </nav>

      <div className="sidebar-spacer" />

      <section className={`side-session${listen.active ? " is-live" : ""}`}>
        <div className="side-session-head">
          <span className="side-session-title">
            <Radio size={13} />
            Session d'écoute
          </span>
          {listen.active && <span className="live-dot" />}
        </div>

        {listen.active ? (
          <>
            <strong className="side-session-name">{listen.deviceName ?? "Microphone"}</strong>
            <div className="side-session-meter" aria-hidden="true">
              <span style={{ width: `${meterPercent(listen.peak)}%` }} />
            </div>
            <div className="side-session-foot">
              <span>
                {Math.round(listen.volume)}% · {listen.muted ? "Muet" : "Actif"}
              </span>
              <button
                type="button"
                className="btn btn-tiny btn-danger"
                onClick={() => void stopListen()}
                disabled={busy === "stop"}
              >
                <Square size={11} />
                Stop
              </button>
            </div>
          </>
        ) : (
          <>
            <p className="side-session-idle">Aucune écoute en cours</p>
            <button
              type="button"
              className="btn btn-small btn-ghost"
              onClick={() => setPage("microphones")}
            >
              <Mic size={13} />
              Choisir un micro
            </button>
          </>
        )}
      </section>

      <div className="sidebar-footer">
        <button
          type="button"
          className="theme-switch"
          onClick={() => void updateSettings({ theme: nextTheme })}
          title={`Passer en thème ${nextTheme === "dark" ? "sombre" : "clair"}`}
        >
          {nextTheme === "dark" ? <Moon size={15} /> : <Sun size={15} />}
          <span>{nextTheme === "dark" ? "Thème sombre" : "Thème clair"}</span>
        </button>
        <div className="sidebar-meta">
          <PermissionPill />
          <span className="version">v{appInfo?.version ?? "0.1.0"}</span>
        </div>
      </div>
    </aside>
  );
}
