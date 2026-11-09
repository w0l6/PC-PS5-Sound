import {
  CheckCircle2,
  Database,
  ExternalLink,
  FolderOpen,
  Info,
  Monitor,
  Moon,
  RotateCcw,
  ShieldCheck,
  SlidersHorizontal,
  Sun,
  Trash2,
  Volume2,
  Waves,
} from "lucide-react";
import { useEffect, useState } from "react";
import { Segmented } from "../components/Segmented";
import { Toggle } from "../components/Toggle";
import { VolumeSlider } from "../components/VolumeSlider";
import { useApp } from "../context/AppContext";
import type { Settings } from "../lib/types";

export function SettingsPage() {
  const {
    settings,
    updateSettings,
    permission,
    openMicrophoneSettings,
    resetCustomNames,
    appInfo,
    devices,
  } = useApp();
  const [confirmingReset, setConfirmingReset] = useState(false);

  const customCount = devices.filter((device) => device.customName).length;

  useEffect(() => {
    if (!confirmingReset) return;
    const timer = window.setTimeout(() => setConfirmingReset(false), 4500);
    return () => window.clearTimeout(timer);
  }, [confirmingReset]);

  const handleReset = async () => {
    if (!confirmingReset) {
      setConfirmingReset(true);
      return;
    }
    setConfirmingReset(false);
    await resetCustomNames();
  };

  return (
    <>
      <header className="page-header">
        <div className="page-heading">
          <span className="eyebrow">Configuration</span>
          <h1>Paramètres</h1>
          <p className="page-sub">
            Apparence, écoute, autorisations Windows et données locales de l'application.
          </p>
        </div>
      </header>

      <div className="settings-grid">
        <section className="settings-card">
          <header className="settings-head">
            <span className="settings-icon">
              <Waves size={17} />
            </span>
            <div>
              <h2>Apparence</h2>
              <p>Thème et confort visuel de l'interface.</p>
            </div>
          </header>

          <div className="setting-block">
            <span className="setting-label">Thème</span>
            <Segmented<Settings["theme"]>
              ariaLabel="Thème de l'application"
              value={settings.theme}
              onChange={(theme) => void updateSettings({ theme })}
              options={[
                { value: "dark", label: "Sombre", icon: <Moon size={14} /> },
                { value: "light", label: "Clair", icon: <Sun size={14} /> },
                { value: "system", label: "Système", icon: <Monitor size={14} /> },
              ]}
            />
            <p className="setting-help">
              Le mode Système suit le réglage clair/sombre de Windows en direct.
            </p>
          </div>

          <div className="setting-row">
            <div>
              <strong>Réduire les animations</strong>
              <p>Désactive les transitions et effets de mouvement.</p>
            </div>
            <Toggle
              label="Réduire les animations"
              checked={settings.reduceMotion}
              onChange={(reduceMotion) => void updateSettings({ reduceMotion })}
            />
          </div>

          <div className="setting-row">
            <div>
              <strong>Cartes compactes</strong>
              <p>Réduit la hauteur des cartes de microphone.</p>
            </div>
            <Toggle
              label="Cartes compactes"
              checked={settings.compactCards}
              onChange={(compactCards) => void updateSettings({ compactCards })}
            />
          </div>
        </section>

        <section className="settings-card">
          <header className="settings-head">
            <span className="settings-icon">
              <Volume2 size={17} />
            </span>
            <div>
              <h2>Écoute</h2>
              <p>Retour audio logiciel, sans toucher au volume Windows.</p>
            </div>
          </header>

          <div className="setting-block">
            <VolumeSlider
              label="Volume d'écoute par défaut"
              value={settings.listenVolume}
              onChange={(listenVolume) => void updateSettings({ listenVolume })}
            />
            <p className="setting-help">
              Ce volume s'applique au retour audio de l'application. Le volume de capture
              Windows du microphone n'est jamais modifié.
            </p>
          </div>

          <div className="setting-row">
            <div>
              <strong>Protection anti-larsen</strong>
              <p>
                Limiteur doux, filtre passe-haut 80 Hz et réduction automatique du gain en
                sortie haut-parleurs.
              </p>
            </div>
            <Toggle
              label="Protection anti-larsen"
              checked={settings.feedbackGuard}
              onChange={(feedbackGuard) => void updateSettings({ feedbackGuard })}
            />
          </div>

          <div className="setting-row">
            <div>
              <strong>Afficher les périphériques inactifs</strong>
              <p>Déconnectés, désactivés ou non présents dans Windows.</p>
            </div>
            <Toggle
              label="Afficher les périphériques inactifs"
              checked={settings.showInactive}
              onChange={(showInactive) => void updateSettings({ showInactive })}
            />
          </div>
        </section>

        <section className="settings-card">
          <header className="settings-head">
            <span className="settings-icon">
              <ShieldCheck size={17} />
            </span>
            <div>
              <h2>Autorisation Windows</h2>
              <p>Accès microphone accordé à l'application.</p>
            </div>
          </header>

          <div className={`permission-card${permission === "granted" ? " is-ok" : ""}`}>
            <span className="permission-card-icon">
              {permission === "granted" ? (
                <CheckCircle2 size={20} />
              ) : (
                <ShieldCheck size={20} />
              )}
            </span>
            <div>
              <strong>
                {permission === "granted"
                  ? "Microphone autorisé"
                  : permission === "denied"
                    ? "Microphone bloqué par Windows"
                    : "Autorisation à vérifier"}
              </strong>
              <p>
                {permission === "granted"
                  ? "L'application peut ouvrir un flux de capture sur vos microphones."
                  : "Ouvrez Confidentialité et sécurité → Microphone dans Windows et autorisez l'accès aux applications de bureau."}
              </p>
            </div>
          </div>

          <button
            type="button"
            className="btn btn-primary full"
            onClick={() => void openMicrophoneSettings()}
          >
            <ExternalLink size={14} />
            Ouvrir les paramètres Windows
          </button>

          <div className="setting-row">
            <div>
              <strong>Surveillance automatique</strong>
              <p>La liste et les niveaux sont actualisés en continu par le backend Rust.</p>
            </div>
            <span className="setting-state">
              <SlidersHorizontal size={13} />
              Active
            </span>
          </div>
        </section>

        <section className="settings-card">
          <header className="settings-head">
            <span className="settings-icon">
              <Database size={17} />
            </span>
            <div>
              <h2>Données locales</h2>
              <p>Noms personnalisés et préférences, stockés sur ce PC uniquement.</p>
            </div>
          </header>

          <div className="data-path">
            <FolderOpen size={15} />
            <code title={appInfo?.dataPath}>
              {appInfo?.dataPath ?? "Chargement du chemin…"}
            </code>
          </div>

          <div className="setting-row">
            <div>
              <strong>Noms personnalisés</strong>
              <p>
                {customCount > 0
                  ? `${customCount} microphone${customCount > 1 ? "s" : ""} renommé${customCount > 1 ? "s" : ""} localement.`
                  : "Aucun microphone renommé pour le moment."}
              </p>
            </div>
            <button
              type="button"
              className={`btn btn-small${confirmingReset ? " btn-danger" : ""}`}
              onClick={() => void handleReset()}
              disabled={customCount === 0}
            >
              {confirmingReset ? <Trash2 size={13} /> : <RotateCcw size={13} />}
              {confirmingReset ? "Confirmer" : "Réinitialiser"}
            </button>
          </div>

          <div className="privacy-note">
            <CheckCircle2 size={15} />
            <p>
              Aucune donnée n'est transmise sur Internet. Aucun compte, aucun serveur, aucune
              télémétrie.
            </p>
          </div>
        </section>

        <section className="settings-card about-card">
          <header className="settings-head">
            <span className="settings-icon">
              <Info size={17} />
            </span>
            <div>
              <h2>À propos</h2>
              <p>Audio Manager — console micro Windows.</p>
            </div>
          </header>

          <dl className="about-list">
            <div>
              <dt>Version</dt>
              <dd>{appInfo?.version ?? "0.1.0"}</dd>
            </div>
            <div>
              <dt>Identifiant</dt>
              <dd>{appInfo?.identifier ?? "com.audiomanager.desktop"}</dd>
            </div>
            <div>
              <dt>Moteur audio</dt>
              <dd>WASAPI partagé · faible latence</dd>
            </div>
          </dl>

          <div className="tech-tags">
            <span>Rust</span>
            <span>Tauri 2</span>
            <span>React 19</span>
            <span>TypeScript</span>
            <span>WASAPI</span>
          </div>
        </section>
      </div>
    </>
  );
}
