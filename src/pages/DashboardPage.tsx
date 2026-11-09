import {
  Activity,
  AudioLines,
  CheckCircle2,
  ChevronRight,
  Database,
  ExternalLink,
  Headphones,
  Mic,
  Play,
  RefreshCw,
  ShieldAlert,
  ShieldCheck,
  Speaker,
  Square,
  Volume2,
  VolumeX,
} from "lucide-react";
import { useApp } from "../context/AppContext";
import { DeviceCard } from "../components/DeviceCard";
import { LevelMeter } from "../components/LevelMeter";
import { PermissionBanner } from "../components/PermissionBanner";
import { StatCard } from "../components/StatCard";
import { VolumeSlider } from "../components/VolumeSlider";

export function DashboardPage() {
  const {
    devices,
    listen,
    snapshot,
    settings,
    permission,
    refreshing,
    busy,
    refresh,
    setPage,
    openMicrophoneSettings,
    stopListen,
    setVolume,
    toggleMute,
  } = useApp();

  const activeDevices = devices.filter((device) => device.state === "active");
  const defaultDevice =
    devices.find((device) => device.isDefault) ?? activeDevices[0] ?? null;
  const featured = [
    ...activeDevices,
    ...devices.filter((device) => device.state !== "active"),
  ].slice(0, 4);

  const guardActive = settings.feedbackGuard && listen.feedbackGuard;
  const monitorError = snapshot?.monitorStatus === "error";

  return (
    <>
      <header className="page-header">
        <div className="page-heading">
          <span className="eyebrow">Vue d'ensemble</span>
          <h1>Votre studio audio</h1>
          <p className="page-sub">
            Surveillance locale des microphones Windows, écoute temps réel et réglages
            indépendants.
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
          <button type="button" className="btn btn-primary" onClick={() => setPage("microphones")}>
            <Mic size={15} />
            Gérer les micros
          </button>
        </div>
      </header>

      <PermissionBanner />

      <section className="stat-grid" aria-label="Résumé audio">
        <StatCard
          icon={<Mic size={18} />}
          label="Micros détectés"
          value={devices.length}
          hint={`${activeDevices.length} connecté${activeDevices.length > 1 ? "s" : ""}`}
          tone="accent"
        />
        <StatCard
          icon={<Activity size={18} />}
          label="Connectés"
          value={activeDevices.length}
          hint="Surveillance automatique"
          tone="success"
        />
        <StatCard
          icon={<Headphones size={18} />}
          label="Micro par défaut"
          value={defaultDevice ? defaultDevice.displayName : "Aucun"}
          hint={defaultDevice?.formFactorLabel ?? "Aucun périphérique actif"}
          tone="neutral"
        />
        <StatCard
          icon={listen.active ? <Volume2 size={18} /> : <VolumeX size={18} />}
          label="Écoute"
          value={listen.active ? (listen.deviceName ?? "Active") : "Silencieux"}
          hint={
            listen.active
              ? `${Math.round(listen.volume)}% · ${listen.muted ? "Muet" : "Actif"}`
              : "Aucune session"
          }
          tone={listen.active ? "warning" : "neutral"}
        />
      </section>

      <div className="dashboard-grid">
        <section className="section-card">
          <header className="section-head">
            <div>
              <span className="section-kicker">Entrées</span>
              <h2>Microphones</h2>
            </div>
            <button
              type="button"
              className="btn btn-small btn-ghost"
              onClick={() => setPage("microphones")}
            >
              Tout voir
              <ChevronRight size={14} />
            </button>
          </header>

          {featured.length > 0 ? (
            <div className="device-grid compact-grid">
              {featured.map((device) => (
                <DeviceCard key={device.id} device={device} />
              ))}
            </div>
          ) : (
            <div className="empty-state">
              <span className="empty-orb">
                <Mic size={24} />
              </span>
              <strong>Aucun microphone détecté</strong>
              <p>
                Branchez un microphone puis actualisez la liste. Windows doit le voir dans
                Paramètres → Système → Son → Entrée.
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
          )}
        </section>

        <aside className="dashboard-side">
          <section className={`panel session-panel${listen.active ? " is-live" : ""}`}>
            <header className="panel-head">
              <div>
                <span className="section-kicker">Retour audio</span>
                <h2>Session d'écoute</h2>
              </div>
              {listen.active && (
                <span className="live-pill">
                  <span className="live-dot" />
                  Live
                </span>
              )}
            </header>

            {listen.active ? (
              <>
                <div className="session-hero">
                  <span className="session-icon">
                    <AudioLines size={20} />
                  </span>
                  <div>
                    <strong>{listen.deviceName}</strong>
                    <span>
                      {listen.outputName ?? "Sortie par défaut"} ·{" "}
                      {listen.muted ? "Muet" : "Volume actif"}
                    </span>
                  </div>
                </div>

                <LevelMeter level={listen.peak} muted={listen.muted} />

                <VolumeSlider value={listen.volume} onChange={setVolume} muted={listen.muted} />

                <div className="session-actions">
                  <button
                    type="button"
                    className={`btn${listen.muted ? " btn-warn" : ""}`}
                    onClick={toggleMute}
                  >
                    {listen.muted ? <VolumeX size={15} /> : <Volume2 size={15} />}
                    {listen.muted ? "Réactiver" : "Muet"}
                  </button>
                  <button
                    type="button"
                    className="btn btn-danger"
                    onClick={() => void stopListen()}
                    disabled={busy === "stop"}
                  >
                    <Square size={13} />
                    Arrêter
                  </button>
                </div>

                {guardActive && (
                  <p className="guard-note">
                    <ShieldAlert size={13} />
                    Protection anti-larsen active : limiteur doux et filtre passe-haut.
                  </p>
                )}
              </>
            ) : (
              <div className="session-empty">
                <span className="empty-orb">
                  <Mic size={24} />
                </span>
                <strong>Aucune écoute active</strong>
                <p>
                  Sélectionnez un microphone connecté et lancez l'écoute. Un seul micro peut
                  être écouté à la fois.
                </p>
                <button
                  type="button"
                  className="btn btn-primary"
                  onClick={() => setPage("microphones")}
                >
                  <Play size={14} />
                  Choisir un microphone
                </button>
              </div>
            )}
          </section>

          <section className="panel system-panel">
            <header className="panel-head">
              <div>
                <span className="section-kicker">Système</span>
                <h2>État local</h2>
              </div>
              <Activity size={16} className="panel-head-icon" />
            </header>

            <ul className="system-list">
              <li>
                <span>
                  <Speaker size={14} />
                  Sortie par défaut
                </span>
                <strong title={snapshot?.defaultOutput ?? undefined}>
                  {snapshot?.defaultOutput ?? "Non détectée"}
                </strong>
              </li>
              <li>
                <span>
                  <ShieldCheck size={14} />
                  Autorisation micro
                </span>
                <strong
                  className={
                    permission === "granted"
                      ? "is-ok"
                      : permission === "denied"
                        ? "is-bad"
                        : "is-warn"
                  }
                >
                  {permission === "granted"
                    ? "Autorisée"
                    : permission === "denied"
                      ? "Bloquée"
                      : "À vérifier"}
                </strong>
              </li>
              <li>
                <span>
                  <Activity size={14} />
                  Surveillance
                </span>
                <strong className={monitorError ? "is-bad" : "is-ok"}>
                  {monitorError ? "Erreur" : "Active (1 s)"}
                </strong>
              </li>
              <li>
                <span>
                  <Database size={14} />
                  Stockage
                </span>
                <strong className="is-ok">100 % local</strong>
              </li>
            </ul>

            <button
              type="button"
              className="btn btn-small btn-ghost full"
              onClick={() => void openMicrophoneSettings()}
            >
              <ExternalLink size={13} />
              Ouvrir les paramètres Windows
            </button>
          </section>

          <div className="local-note">
            <CheckCircle2 size={15} />
            <p>
              Aucune donnée audio ou personnelle n'est envoyée sur Internet. Les noms
              personnalisés restent sur cet ordinateur.
            </p>
          </div>
        </aside>
      </div>
    </>
  );
}
