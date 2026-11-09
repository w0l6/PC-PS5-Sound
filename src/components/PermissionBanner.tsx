import { AlertTriangle, CheckCircle2, HelpCircle, ShieldX } from "lucide-react";
import { useApp } from "../context/AppContext";

export function PermissionBanner() {
  const { permission, openMicrophoneSettings } = useApp();

  if (permission === "granted") return null;

  const denied = permission === "denied";
  const Icon = denied ? ShieldX : permission === "unknown" ? HelpCircle : AlertTriangle;

  return (
    <div className={`permission-banner${denied ? " is-denied" : ""}`}>
      <span className="permission-icon">
        <Icon size={18} />
      </span>
      <div className="permission-copy">
        <strong>
          {denied
            ? "Windows bloque l'accès au microphone"
            : "Autorisation microphone à vérifier"}
        </strong>
        <span>
          {denied
            ? "Autorisez l'application dans Confidentialité et sécurité > Microphone pour pouvoir écouter."
            : "Si l'écoute échoue, vérifiez l'autorisation microphone de Windows."}
        </span>
      </div>
      <button type="button" className="btn btn-small" onClick={() => void openMicrophoneSettings()}>
        Ouvrir les paramètres
      </button>
    </div>
  );
}

export function PermissionPill() {
  const { permission } = useApp();
  const granted = permission === "granted";
  const denied = permission === "denied";
  const Icon = granted ? CheckCircle2 : denied ? ShieldX : HelpCircle;
  return (
    <span className={`permission-pill${granted ? " ok" : denied ? " bad" : " warn"}`}>
      <Icon size={12} />
      {granted ? "Micro autorisé" : denied ? "Micro bloqué" : "À vérifier"}
    </span>
  );
}
