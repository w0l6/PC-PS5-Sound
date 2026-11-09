import {
  AudioLines,
  Headphones,
  ShieldCheck,
  Square,
  Volume2,
  VolumeX,
} from "lucide-react";
import { useApp } from "../context/AppContext";
import { LevelMeter } from "./LevelMeter";
import { VolumeSlider } from "./VolumeSlider";

export function ListenBar() {
  const { listen, settings, busy, stopListen, setVolume, toggleMute } = useApp();

  const visible =
    listen.active || listen.status === "starting" || listen.status === "stopping";
  if (!visible) return null;

  const guard = settings.feedbackGuard && listen.feedbackGuard;
  const label =
    listen.status === "starting"
      ? "Connexion au micro…"
      : listen.status === "stopping"
        ? "Arrêt en cours…"
        : listen.muted
          ? "Écoute muette"
          : "Écoute en cours";

  return (
    <div className={`listen-dock${listen.status === "starting" ? " is-starting" : ""}`}>
      <div className="dock-identity">
        <span className="dock-icon">
          <AudioLines size={19} />
        </span>
        <div className="dock-copy">
          <span className="dock-eyebrow">
            <span className="live-dot" />
            {label}
          </span>
          <strong>{listen.deviceName ?? "Microphone"}</strong>
        </div>
      </div>

      <div className="dock-meter">
        <LevelMeter level={listen.peak} compact muted={listen.muted} />
      </div>

      <div className="dock-volume">
        <VolumeSlider
          value={listen.volume}
          onChange={setVolume}
          compact
          muted={listen.muted}
          label="Volume d'écoute"
        />
      </div>

      <button
        type="button"
        className={`btn btn-icon${listen.muted ? " btn-warn" : ""}`}
        onClick={toggleMute}
        title={listen.muted ? "Réactiver le son" : "Couper le son"}
        aria-label={listen.muted ? "Réactiver le son" : "Couper le son"}
      >
        {listen.muted ? <VolumeX size={17} /> : <Volume2 size={17} />}
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

      <div className="dock-output" title={`Sortie : ${listen.outputName ?? "par défaut"}`}>
        <Headphones size={14} />
        <span>{listen.outputName ?? "Sortie par défaut"}</span>
        {guard && (
          <span className="dock-guard" title="Protection anti-larsen active">
            <ShieldCheck size={14} />
          </span>
        )}
      </div>
    </div>
  );
}
