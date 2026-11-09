import { getCurrentWindow } from "@tauri-apps/api/window";
import { AudioLines, Maximize2, Minimize2, Minus, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useApp } from "../context/AppContext";

export function TitleBar() {
  const { minimize, toggleMaximize, closeWindow } = useApp();
  const [maximized, setMaximized] = useState(false);

  // Le glissement et le double-clic sont gérés nativement par Tauri
  // (`data-tauri-drag-region`). On synchronise simplement l'icône.
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;

    const window = getCurrentWindow();
    void window.isMaximized().then((value) => {
      if (!disposed) setMaximized(value);
    });
    void window.onResized(() => {
      void window.isMaximized().then((value) => {
        if (!disposed) setMaximized(value);
      });
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  const handleMaximize = async () => {
    const next = await toggleMaximize();
    if (next !== null) setMaximized(next);
  };

  return (
    <header className="titlebar">
      <div className="titlebar-drag" data-tauri-drag-region="deep">
        <span className="titlebar-mark">
          <AudioLines size={15} />
        </span>
        <span className="titlebar-title">Audio Manager</span>
        <span className="titlebar-divider" />
        <span className="titlebar-sub">Console micro Windows</span>
      </div>
      <div className="window-controls">
        <button type="button" aria-label="Réduire" onClick={() => void minimize()}>
          <Minus size={15} />
        </button>
        <button
          type="button"
          aria-label={maximized ? "Restaurer" : "Agrandir"}
          onClick={() => void handleMaximize()}
        >
          {maximized ? <Minimize2 size={14} /> : <Maximize2 size={14} />}
        </button>
        <button
          type="button"
          className="window-close"
          aria-label="Fermer"
          onClick={() => void closeWindow()}
        >
          <X size={15} />
        </button>
      </div>
    </header>
  );
}
