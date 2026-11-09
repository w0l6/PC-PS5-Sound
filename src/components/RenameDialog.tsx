import { Pencil, RotateCcw, X } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useApp } from "../context/AppContext";

const MAX_LENGTH = 64;

export function RenameDialog() {
  const { renameTarget, setRenameTarget, renameDevice } = useApp();
  const [name, setName] = useState("");
  const [saving, setSaving] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!renameTarget) return;
    setName(renameTarget.customName ?? "");
    setSaving(false);
    const timer = window.setTimeout(() => inputRef.current?.focus(), 40);
    return () => window.clearTimeout(timer);
  }, [renameTarget]);

  useEffect(() => {
    if (!renameTarget) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setRenameTarget(null);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [renameTarget, setRenameTarget]);

  if (!renameTarget) return null;

  const submit = async (value: string | null) => {
    setSaving(true);
    const ok = await renameDevice(renameTarget.id, value);
    if (!ok) setSaving(false);
  };

  return (
    <div
      className="modal-overlay"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) setRenameTarget(null);
      }}
    >
      <div className="modal" role="dialog" aria-modal="true" aria-labelledby="rename-title">
        <header className="modal-head">
          <span className="modal-icon">
            <Pencil size={17} />
          </span>
          <div>
            <h2 id="rename-title">Renommer le microphone</h2>
            <p>Uniquement dans Audio Manager. Windows conserve son nom d'origine.</p>
          </div>
          <button
            type="button"
            className="btn btn-icon"
            aria-label="Fermer"
            onClick={() => setRenameTarget(null)}
          >
            <X size={15} />
          </button>
        </header>

        <form
          className="modal-body"
          onSubmit={(event) => {
            event.preventDefault();
            if (!saving) void submit(name.trim() ? name : null);
          }}
        >
          <label className="field">
            <span className="field-label">Nom personnalisé</span>
            <input
              ref={inputRef}
              className="input"
              value={name}
              maxLength={MAX_LENGTH}
              placeholder={renameTarget.name}
              onChange={(event) => setName(event.target.value)}
              disabled={saving}
            />
            <span className="field-hint">
              Nom Windows actuel : <strong>{renameTarget.name}</strong>
              <span className="field-count">
                {name.length}/{MAX_LENGTH}
              </span>
            </span>
          </label>

          <div className="modal-note">
            Le nom personnalisé est stocké localement et réapparaît après redémarrage. Laisser
            vide pour restaurer le nom Windows.
          </div>

          <footer className="modal-actions">
            <button
              type="button"
              className="btn btn-ghost"
              onClick={() => void submit(null)}
              disabled={saving || !renameTarget.customName}
            >
              <RotateCcw size={14} />
              Nom d'origine
            </button>
            <div className="modal-actions-right">
              <button
                type="button"
                className="btn"
                onClick={() => setRenameTarget(null)}
                disabled={saving}
              >
                Annuler
              </button>
              <button type="submit" className="btn btn-primary" disabled={saving}>
                {saving ? "Enregistrement…" : "Enregistrer"}
              </button>
            </div>
          </footer>
        </form>
      </div>
    </div>
  );
}
