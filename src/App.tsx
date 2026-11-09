import { AudioLines } from "lucide-react";
import { useEffect, useRef } from "react";
import { ListenBar } from "./components/ListenBar";
import { RenameDialog } from "./components/RenameDialog";
import { Sidebar } from "./components/Sidebar";
import { TitleBar } from "./components/TitleBar";
import { Toasts } from "./components/Toasts";
import { useApp } from "./context/AppContext";
import { DashboardPage } from "./pages/DashboardPage";
import { MicrophonesPage } from "./pages/MicrophonesPage";
import { SettingsPage } from "./pages/SettingsPage";

export default function App() {
  const { page, loading, listen } = useApp();
  const contentRef = useRef<HTMLElement>(null);

  useEffect(() => {
    contentRef.current?.scrollTo({ top: 0, behavior: "smooth" });
  }, [page]);

  if (loading) {
    return (
      <div className="boot-screen">
        <span className="boot-mark">
          <AudioLines size={26} />
        </span>
        <strong>Audio Manager</strong>
        <span className="boot-caption">Connexion au moteur audio Windows…</span>
        <span className="boot-bar" aria-hidden="true">
          <i />
        </span>
      </div>
    );
  }

  return (
    <div className="app">
      <TitleBar />
      <div className="app-shell">
        <Sidebar />
        <main className="content" ref={contentRef}>
          <div className={`content-inner${listen.active ? " has-dock" : ""}`}>
            {page === "dashboard" && <DashboardPage />}
            {page === "microphones" && <MicrophonesPage />}
            {page === "settings" && <SettingsPage />}
          </div>
        </main>
      </div>
      <ListenBar />
      <Toasts />
      <RenameDialog />
    </div>
  );
}
