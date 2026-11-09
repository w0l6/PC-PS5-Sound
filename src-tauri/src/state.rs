use crate::audio::listen::ListenManager;
use crate::audio::types::AudioSnapshot;
use crate::store::AppStore;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Etat partage entre les commandes Tauri, le moniteur de peripheriques
/// et le moteur d'ecoute.
pub struct AppState {
    pub store: Mutex<AppStore>,
    pub snapshot: RwLock<AudioSnapshot>,
    pub permission: RwLock<String>,
    pub listen: Arc<ListenManager>,
    pub refresh: Sender<()>,
    /// Positionne a `true` a la fermeture de la fenetre pour arreter les threads.
    pub shutdown: AtomicBool,
}

impl AppState {
    pub fn new(store: AppStore, listen: Arc<ListenManager>, refresh: Sender<()>) -> Self {
        Self {
            store: Mutex::new(store),
            snapshot: RwLock::new(AudioSnapshot {
                permission: "unknown".to_string(),
                monitor_status: "starting".to_string(),
                ..AudioSnapshot::default()
            }),
            permission: RwLock::new("unknown".to_string()),
            listen,
            refresh,
            shutdown: AtomicBool::new(false),
        }
    }

    pub fn request_refresh(&self) {
        let _ = self.refresh.send(());
    }

    pub fn is_shutting_down(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    pub fn mark_shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }
}

/// Verrou tolerant aux empoisonnements : une erreur dans un thread
/// ne doit jamais bloquer l'application entiere.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn read<T>(lock: &RwLock<T>) -> RwLockReadGuard<'_, T> {
    lock.read().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn write<T>(lock: &RwLock<T>) -> RwLockWriteGuard<'_, T> {
    lock.write().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Applique les noms personnalises aux peripheriques d'un instantane.
/// Le nom Windows d'origine reste toujours disponible dans `name`.
pub fn apply_custom_names(snapshot: &mut AudioSnapshot, names: &HashMap<String, String>) {
    for device in &mut snapshot.devices {
        device.custom_name = names.get(&device.id).cloned();
        device.display_name = device
            .custom_name
            .clone()
            .unwrap_or_else(|| device.name.clone());
    }
}
