import {
  AudioLines,
  Check,
  Headphones,
  Loader2,
  Mic,
  Pencil,
  Play,
  ShieldAlert,
  Square,
  Volume2,
  VolumeX,
} from "lucide-react";
import { useApp } from "../context/AppContext";
import type { AudioDevice } from "../lib/types";
import { useMeterLevel } from "../lib/useMeter";
import { LevelMeter } from "./LevelMeter";
import { VolumeSlider } from "./VolumeSlider";

interface DeviceCardProps {
  device: AudioDevice;
}

const speakerFactors = ["speakers", "hdmi", "digital", "spdif"];

export function DeviceCard({ device }: DeviceCardProps) {
  const {
    listen,
    settings,
    snapshot,
    busy,
    startListen,
    stopListen,
    setVolume,
    toggleMute,
    setRenameTarget,
  } = useApp();

  const isActive = device.state === "active";
  const level = useMeterLevel(device.id, isActive);
  const engaged =
    listen.deviceId === device.id && (listen.active || listen.status === "starting");
  const starting = listen.deviceId === device.id && listen.status === "starting";
  const guardActive = settings.feedbackGuard && listen.feedbackGuard;
  const outputIsSpeakers = speakerFactors.includes(
    snapshot?.defaultOutputFormFactor ?? "",
  );

  const Icon =
    device.formFactor === "headset" || device.formFactor === "headphones"
      ? Headphones
      : device.formFactor === "microphone"
        ? Mic
        : AudioLines;

  return (
    <article
      className={[
        "device-card",
        engaged ? "is-listening" : "",
        !isActive ? "is-offline" : "",
        settings.compactCards ? "is-compact" : "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      <header className="device-card-head">
        <span className={`device-icon${engaged ? " is-live" : ""}`}>
          <Icon size={18} />
        </span>
        <div className="device-identity">
          <div className="device-title-row">
            <h3 title={device.displayName}>{device.displayName}</h3>
            {device.customName && <span className="custom-tag">Personnalisé</span>}
          </div>
          <div className="device-meta">
            <span className={`state-badge state-${device.state}`}>
              <i />
              {device.stateLabel}
            </span>
            {device.isDefault && (
              <span className="chip chip-accent">
                <Check size={11} />
                Par défaut
              </span>
            )}
            {device.isDefaultCommunication && <span className="chip">Communication</span>}
          </div>
        </div>
        <button
          type="button"
          className="btn btn-icon"
          onClick={() => setRenameTarget(device)}
          title="Renommer dans l'application"
          aria-label={`Renommer ${device.displayName}`}
        >
          <Pencil size={15} />
        </button>
      </header>

      <div className="device-signal">
        <div className="signal-head">
          <span>
            <AudioLines size={13} />
            Niveau d'entrée
          </span>
          <span className="signal-hint">
            {isActive ? (device.muted ? "Muet Windows" : "Temps réel") : "Indisponible"}
          </span>
        </div>
        <LevelMeter level={isActive && !device.muted ? level : 0} muted={device.muted} />
      </div>

      <div className="device-facts">
        <div className="fact">
          <span>Volume Windows</span>
          <strong className={device.muted ? "is-muted" : ""}>
            {Math.round(device.volumePercent)}%
            {device.muted && <VolumeX size={12} />}
          </strong>
        </div>
        <div className="fact">
          <span>Type</span>
          <strong>{device.formFactorLabel}</strong>
        </div>
        <div className="fact wide">
          <span>Interface</span>
          <strong title={device.interfaceName || device.deviceDesc}>
            {device.interfaceName || device.deviceDesc || "—"}
          </strong>
        </div>
      </div>

      {engaged ? (
        <div className="listen-panel">
          <div className="listen-panel-top">
            <span className="listen-live">
              <span className="live-dot" />
              {starting ? "Démarrage…" : "Écoute active"}
            </span>
            <span className="listen-output">
              <Headphones size={13} />
              {listen.outputName ?? "Sortie par défaut"}
            </span>
          </div>

          <VolumeSlider value={listen.volume} onChange={setVolume} muted={listen.muted} />

          <div className="listen-actions">
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
              Protection anti-larsen active
              {outputIsSpeakers
                ? " — sortie haut-parleurs limitée automatiquement."
                : " — limiteur et filtre passe-haut activés."}
            </p>
          )}
        </div>
      ) : (
        <div className="device-actions">
          <button
            type="button"
            className="btn btn-primary grow"
            disabled={!isActive || busy === device.id}
            onClick={() => void startListen(device.id)}
          >
            {busy === device.id ? (
              <Loader2 size={15} className="spin" />
            ) : (
              <Play size={15} />
            )}
            {isActive ? "Écouter" : "Indisponible"}
          </button>
          <button type="button" className="btn" onClick={() => setRenameTarget(device)}>
            <Pencil size={14} />
            Renommer
          </button>
        </div>
      )}
    </article>
  );
}
