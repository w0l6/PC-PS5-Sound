import { meterDecibels, meterPercent } from "../lib/useMeter";

const SEGMENTS = 34;

interface LevelMeterProps {
  level: number;
  compact?: boolean;
  muted?: boolean;
}

export function LevelMeter({ level, compact = false, muted = false }: LevelMeterProps) {
  const percent = meterPercent(level);
  const lit = Math.round((percent / 100) * SEGMENTS);

  return (
    <div className={`level-meter${compact ? " compact" : ""}${muted ? " muted" : ""}`}>
      <div
        className="level-bars"
        role="meter"
        aria-label="Niveau sonore"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={Math.round(percent)}
      >
        {Array.from({ length: SEGMENTS }, (_, index) => {
          const position = (index + 1) / SEGMENTS;
          const tone = position > 0.87 ? "hot" : position > 0.68 ? "warm" : "cool";
          return (
            <span
              key={index}
              className={`level-bar tone-${tone}${index < lit ? " lit" : ""}`}
              style={{ height: `${48 + position * 52}%` }}
            />
          );
        })}
      </div>
      {!compact && <span className="level-value">{muted ? "Muet" : meterDecibels(level)}</span>}
    </div>
  );
}
