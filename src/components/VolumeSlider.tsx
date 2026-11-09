import type { CSSProperties } from "react";
import { clamp } from "../lib/useMeter";

interface VolumeSliderProps {
  value: number;
  onChange: (value: number) => void;
  label?: string;
  compact?: boolean;
  disabled?: boolean;
  muted?: boolean;
}

export function VolumeSlider({
  value,
  onChange,
  label = "Volume d'écoute",
  compact = false,
  disabled = false,
  muted = false,
}: VolumeSliderProps) {
  const percent = clamp(value, 0, 100);
  const style = { "--pct": `${percent}%` } as CSSProperties;

  return (
    <div className={`volume-slider${compact ? " compact" : ""}${muted ? " is-muted" : ""}`}>
      <div className="volume-slider-head">
        <span>{label}</span>
        <strong>{Math.round(percent)}%</strong>
      </div>
      <input
        type="range"
        min={0}
        max={100}
        step={1}
        value={Math.round(percent)}
        onChange={(event) => onChange(Number(event.target.value))}
        style={style}
        aria-label={label}
        disabled={disabled}
      />
    </div>
  );
}
