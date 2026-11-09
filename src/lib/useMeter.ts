import { useEffect, useRef, useState } from "react";
import { subscribeMeter } from "./meterBus";

/**
 * Niveau temps reel d'un peripherique. Le lissage et la retombee sont faits
 * cote interface pour rester fluides a 25 Hz.
 */
export function useMeterLevel(deviceId: string | null | undefined, enabled: boolean): number {
  const [level, setLevel] = useState(0);
  const decay = useRef(0);

  useEffect(() => {
    if (!deviceId || !enabled) {
      decay.current = 0;
      setLevel(0);
      return;
    }

    return subscribeMeter((levels) => {
      const raw = levels[deviceId] ?? 0;
      decay.current = Math.max(raw, decay.current * 0.78);
      setLevel(decay.current);
    });
  }, [deviceId, enabled]);

  return level;
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

export function meterPercent(level: number): number {
  return clamp(Math.pow(clamp(level, 0, 1), 0.55) * 100, 0, 100);
}

export function meterDecibels(level: number): string {
  if (level <= 0.0005) return "-∞ dB";
  const db = 20 * Math.log10(clamp(level, 0.0005, 1));
  return `${db > -0.5 ? "0.0" : db.toFixed(1)} dB`;
}

export function relativeTime(timestamp: number | null): string {
  if (!timestamp) return "—";
  const seconds = Math.max(0, Math.round((Date.now() - timestamp) / 1000));
  if (seconds < 5) return "à l'instant";
  if (seconds < 60) return `il y a ${seconds} s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `il y a ${minutes} min`;
  const hours = Math.floor(minutes / 60);
  return `il y a ${hours} h`;
}
