import {
  ExternalLink,
  Mic,
  MicOff,
  RefreshCw,
  Search,
  SlidersHorizontal,
} from "lucide-react";
import { useMemo, useState } from "react";
import { DeviceCard } from "../components/DeviceCard";
import { PermissionBanner } from "../components/PermissionBanner";
import { useApp } from "../context/AppContext";

type Filter = "all" | "active" | "inactive";

const filters: Array<{ id: Filter; label: string }> = [
  { id: "all", label: "Tous" },
  { id: "active", label: "Connectés" },
  { id: "inactive", label: "Inactifs" },
];

export function MicrophonesPage() {
  const {
    devices,
    settings,
    refreshing,
    refresh,
    updateSettings,
    openMicrophoneSettings,
    listen,
  } = useApp();
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<Filter>("all");

  const activeCount = devices.filter((device) => device.state === "active").length;
  const inactiveCount = devices.length - activeCount;
  const hiddenCount = settings.showInactive ? 0 : inactiveCount;

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return devices
      .filter((device) => settings.showInactive || device.state === "active")
      .filter((device) => {
        if (filter === "active") return device.state === "active";
        if (filter === "inactive") return device.state !== "active";
        return true;
      })
      .filter((device) => {
        if (!needle) return true;
        return [device.displayName, device.name, device.interfaceName, device.deviceDesc, device.formFactorLabel]
          .join(" ")
          .toLowerCase()
          .includes(needle);
      });
  }, [devices, settings.showInactive, filter, query]);

  return (
    <>
      <header className="page-header">
        <div className="page-heading">
          <span className="eyebrow">Périphériques d'entrée</span>
          <h1>Microphones</h1>
          <p className="page-sub">
            Tous les périphériques d'enregistrement Windows, avec niveau temps réel et écoute
            logicielle indépendante.
          </p>
        </div>
        <div className="page-actions">
          <button
            type="button"
            className="btn"
            onClick={() => void refresh()}
            disabled={refreshing}
          >
            <RefreshCw size={15} className={refreshing ? "spin" : undefined} />
            {refreshing ? "Analyse…" : "Actualiser"}
          </button>
          <button type="button" className="btn" onClick={() => void openMicrophoneSettings()}>
            <ExternalLink size={15} />
            Autorisation Windows
          </button>
        </div>
      </header>

      <PermissionBanner />

      <div className="toolbar">
        <label className="search-field">
          <Search size={15} />
          <input
            type="search"
            placeholder="Rechercher un microphone, une interface…"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            aria-label="Rechercher un microphone"
          />
        </label>

        <div className="filter-chips" role="group" aria-label="Filtrer les microphones">
          {filters.map((item) => {
            const count =
              item.id === "all" ? devices.length : item.id === "active" ? activeCount : inactiveCount;
            return (
              <button
                key={item.id}
                type="button"
                className={`filter-chip${filter === item.id ? " is-active" : ""}`}
                onClick={() => setFilter(item.id)}
              >
                {item.label}
                <span>{count}</span>
              </button>
            );
          })}
        </div>

        <span className="toolbar-count">
          {visible.length} affiché{visible.length > 1 ? "s" : ""}
        </span>
      </div>

      {hiddenCount > 0 && (
        <div className="inline-note">
          <SlidersHorizontal size={14} />
          <span>
            {hiddenCount} périphérique{hiddenCount > 1 ? "s" : ""} inactif
            {hiddenCount > 1 ? "s" : ""} masqué{hiddenCount > 1 ? "s" : ""}.
          </span>
          <button
            type="button"
            className="btn btn-tiny"
            onClick={() => void updateSettings({ showInactive: true })}
          >
            Afficher
          </button>
        </div>
      )}

      {listen.active && (
        <div className="listening-strip">
          <span className="live-dot" />
          <span>
            Écoute en cours : <strong>{listen.deviceName}</strong> — un seul microphone peut être
            écouté à la fois.
          </span>
        </div>
      )}

      {devices.length === 0 ? (
        <div className="empty-state large">
          <span className="empty-orb">
            <MicOff size={26} />
          </span>
          <strong>Aucun périphérique d'enregistrement</strong>
          <p>
            Windows ne signale aucun microphone. Vérifiez le branchement, activez le
            périphérique dans Paramètres → Système → Son → Entrée, puis actualisez.
          </p>
          <div className="empty-actions">
            <button type="button" className="btn btn-primary" onClick={() => void refresh()}>
              <RefreshCw size={14} />
              Actualiser
            </button>
            <button type="button" className="btn" onClick={() => void openMicrophoneSettings()}>
              <ExternalLink size={14} />
              Autorisation Windows
            </button>
          </div>
        </div>
      ) : visible.length === 0 ? (
        <div className="empty-state">
          <span className="empty-orb">
            <Search size={22} />
          </span>
          <strong>Aucun résultat</strong>
          <p>Modifiez la recherche ou le filtre pour retrouver vos microphones.</p>
          <button
            type="button"
            className="btn"
            onClick={() => {
              setQuery("");
              setFilter("all");
            }}
          >
            <Mic size={14} />
            Réinitialiser les filtres
          </button>
        </div>
      ) : (
        <div className={`device-grid${settings.compactCards ? " is-compact" : ""}`}>
          {visible.map((device) => (
            <DeviceCard key={device.id} device={device} />
          ))}
        </div>
      )}
    </>
  );
}
