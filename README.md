# Audio Manager

> Console de bureau Windows pour gérer et écouter vos microphones.

**Audio Manager** est une application de bureau native qui détecte les périphériques
d'enregistrement de Windows, affiche leur état et leur niveau en temps réel, permet de
leur donner un nom local et d'en écouter un à la fois sur la sortie audio par défaut.

L'application ne modifie **jamais** les réglages système : ni le nom des périphériques,
ni le volume de capture Windows. Toutes les personnalisations restent locales à
l'application.

---

## Table des matières

1. [Présentation](#1-présentation)
2. [Aperçu de l'interface](#2-aperçu-de-linterface)
3. [Fonctionnalités](#3-fonctionnalités)
4. [Stack technique](#4-stack-technique)
5. [Architecture](#5-architecture)
6. [Référence IPC](#6-référence-ipc)
7. [Modèles de données](#7-modèles-de-données)
8. [Design system](#8-design-system)
9. [Prérequis](#9-prérequis)
10. [Installation](#10-installation)
11. [Commandes](#11-commandes)
12. [Compilation et distribution](#12-compilation-et-distribution)
13. [Persistance et vie privée](#13-persistance-et-vie-privée)
14. [Explication technique](#14-explication-technique)
15. [Performances](#15-performances)
16. [Sécurité](#16-sécurité)
17. [Dépannage et FAQ](#17-dépannage-et-faq)
18. [Développement et contribution](#18-développement-et-contribution)
19. [Feuille de route](#19-feuille-de-route)
20. [Licence](#20-licence)

---

## 1. Présentation

### Le problème

Windows expose de nombreux périphériques d'entrée : micros USB, casques, casques-micro,
cartes son, webcams, entrées virtuelles, etc. Le panneau
*Paramètres → Système → Son → Entrée* reste limité :

- peu d'informations sur chaque périphérique ;
- aucun moyen simple de surveiller le niveau en direct ;
- pas d'écoute rapide sans passer par un logiciel tiers ;
- impossible de renommer un micro localement sans toucher au système ;
- les périphériques désactivés ou débranchés disparaissent ou sont difficiles à
  identifier.

### La solution

Audio Manager fournit une console unique, rapide et lisible pour :

- **voir** tous les microphones, même inactifs ;
- **comprendre** leur état et leur niveau en temps réel ;
- **écouter** un microphone sur la sortie par défaut, avec un volume logiciel ;
- **nommer** chaque micro localement, sans modifier Windows ;
- **protéger** l'utilisateur contre le larsen lors d'une écoute sur haut-parleurs.

### Philosophie

Le projet repose sur quatre principes :

1. **Fonctionnel avant décoratif.** Chaque bouton, chaque indicateur correspond à une
   action ou à une donnée réelle du système.
2. **Non intrusif.** L'application lit les réglages Windows, elle n'y écrit jamais.
3. **Local par défaut.** Aucune connexion réseau, aucune télémétrie, aucune donnée
   envoyée.
4. **Séparation stricte.** Le backend Rust parle à Windows ; le frontend React ne
   connaît que des commandes et des événements typés.

### Public visé

- Utilisateurs qui jonglent entre plusieurs micros (streaming, visioconférence,
  enregistrement).
- Personnes qui veulent tester rapidement un micro sans ouvrir un logiciel lourd.
- Développeurs qui cherchent un exemple concret d'intégration WASAPI dans Tauri 2.

---

## 2. Aperçu de l'interface

L'interface est organisée autour d'une barre de titre personnalisée, d'une barre
latérale et d'un contenu central. Trois pages sont disponibles.

### Tableau de bord

- Cartes de statistiques : nombre de microphones, périphérique actif, session d'écoute,
  état de l'autorisation.
- Aperçu des microphones actifs avec leur vumètre.
- Panneau de session : micro écouté, volume, mute, arrêt.
- Panneau système : sortie par défaut, autorisation, statut du moteur audio.

### Microphones

- Liste complète des périphériques d'enregistrement.
- Recherche par nom.
- Filtres : tous, actifs, inactifs.
- Cartes détaillées avec état, volume Windows, mute, type de périphérique et vumètre.
- Actions : renommer localement, écouter, arrêter.
- Bascule entre cartes détaillées et cartes compactes.

### Paramètres

- Thème : sombre, clair ou système.
- Volume d'écoute par défaut.
- Protection anti-larsen.
- Affichage des périphériques inactifs.
- Animations réduites.
- Cartes compactes.
- Réinitialisation des noms personnalisés.
- Autorisation microphone et accès direct aux paramètres Windows.
- Chemin du fichier de données.
- Informations sur l'application et les technologies.

### Dock d'écoute

Lorsqu'une écoute est active, un dock apparaît en bas de la fenêtre :

- identité du micro écouté ;
- vumètre temps réel ;
- volume logiciel ;
- bouton mute ;
- bouton d'arrêt ;
- nom de la sortie audio ;
- indicateur de protection anti-larsen.

### Notifications

Les erreurs et événements importants apparaissent sous forme de notifications
non bloquantes, empilées en haut à droite. Elles disparaissent automatiquement après
environ 5 secondes et peuvent être fermées manuellement.

---

## 3. Fonctionnalités

### 3.1 Détection des microphones

- Énumération de **tous** les endpoints d'enregistrement WASAPI, quel que soit leur
  état.
- Prise en charge des états :
  - `Connecté` (actif) ;
  - `Déconnecté` (unplugged) ;
  - `Désactivé` (disabled) ;
  - `Non présent` (notpresent) ;
  - `Inconnu`.
- Pour chaque périphérique, l'application affiche :
  - le nom Windows d'origine ;
  - le nom personnalisé éventuel ;
  - la description matérielle ;
  - l'interface hôte (contrôleur audio) ;
  - le type de périphérique (microphone, casque, casque-micro, ligne, HDMI, etc.) ;
  - le volume de capture Windows en pourcentage (lecture seule) ;
  - l'état de mute système (lecture seule) ;
  - le statut « périphérique par défaut » ;
  - le statut « périphérique de communication par défaut ».
- **Rafraîchissement automatique** : réénumération environ une fois par seconde.
- **Rafraîchissement manuel** disponible depuis l'interface.
- Tri intelligent : actifs d'abord, puis déconnectés, désactivés, non présents, et par
  ordre alphabétique à l'intérieur de chaque groupe.

### 3.2 Niveaux temps réel

- Lecture du pic via `IAudioMeterInformation::GetPeakValue`.
- Fréquence de **25 Hz** (une mesure toutes les 40 ms).
- Valeurs bornées entre 0 et 1.
- Vumètre segmenté avec code couleur :
  - tons froids pour les niveaux faibles ;
  - tons chauds pour les niveaux moyens ;
  - tons rouges avec halo pour la saturation.
- Les niveaux proviennent du pilote audio, pas d'une simulation.
- Diffusion à haut débit via un bus dédié, hors du cycle de rendu React.

### 3.3 Écoute d'un microphone

- Écoute **d'un seul microphone à la fois**.
- Démarrer une nouvelle écoute arrête proprement la précédente.
- États exposés : `idle`, `starting`, `listening`, `stopping`, `error`.
- Volume d'écoute **logiciel** de 0 à 100 %, indépendant du volume Windows.
- Mute logiciel instantané.
- Traitement audio :
  - conversion du format natif du micro vers `float32` stéréo ;
  - gestion des entiers 8/16/24/32 bits et des flottants 32 bits ;
  - duplication du signal mono vers les deux canaux ;
  - filtre passe-haut à 80 Hz ;
  - limiteur doux ;
  - conversion et rééchantillonnage délégués à Windows.
- Latence faible :
  - capture pilotée par événements ;
  - tampon de rendu de 40 ms ;
  - attente d'événement avec délai de garde de 20 ms.
- Arrêt automatique et notification si le microphone disparaît.
- Aucune modification du volume de capture Windows.

### 3.4 Protection anti-larsen

Lorsque la sortie par défaut est un haut-parleur (et non un casque), la protection peut
être activée :

- gain effectif plafonné à **60 %** ;
- seuil de limiteur abaissé (0,86 au lieu de 0,98) ;
- **arrêt automatique** après environ **3 secondes** de signal saturé quasi permanent
  (pic > 0,995) avec un gain non négligeable.

L'objectif est d'éviter qu'une boucle acoustique ne devienne douloureuse ou destructrice
pour les haut-parleurs. La protection est un filet de sécurité, pas un substitut à
l'usage d'un casque.

### 3.5 Renommage local

- Chaque micro peut recevoir un nom personnalisé.
- Le nom est stocké localement et restauré au redémarrage.
- Le nom Windows d'origine reste intact.
- L'identifiant endpoint Windows reste la clé stable en interne.
- Limite : 64 caractères.
- Les caractères de contrôle sont refusés.
- Un nom vide restaure le nom Windows.
- Réinitialisation globale possible depuis les paramètres.

### 3.6 Personnalisation

- Thème sombre par défaut.
- Thème clair.
- Thème système (suit la préférence Windows).
- Animations réduites, en plus du respect de `prefers-reduced-motion`.
- Cartes compactes.
- Affichage ou masquage des périphériques inactifs.
- Volume d'écoute par défaut.

### 3.7 Confidentialité

- **Aucune connexion réseau.**
- Aucune télémétrie.
- Aucun compte.
- Aucune donnée envoyée à un serveur.
- Persistance locale uniquement.
- Aucune écriture dans le registre ou dans les propriétés des périphériques.

### 3.8 Tableau récapitulatif

| Fonction | État | Détail |
| --- | --- | --- |
| Détection de tous les micros | ✅ | WASAPI, tous états |
| Niveau temps réel | ✅ | 25 Hz |
| Volume Windows affiché | ✅ | Lecture seule |
| Mute Windows affiché | ✅ | Lecture seule |
| Renommage local | ✅ | Persistant |
| Écoute d'un micro | ✅ | Session unique |
| Volume d'écoute logiciel | ✅ | 0–100 % |
| Mute logiciel | ✅ | Instantané |
| Anti-larsen | ✅ | Limiteur + arrêt auto |
| Détection de disparition | ✅ | Arrêt + notification |
| Autorisation micro | ✅ | Registre + lien Paramètres |
| Thème sombre/clair/système | ✅ | Persistant |
| Animations réduites | ✅ | Préférence + système |
| Données locales | ✅ | JSON atomique |
| Connexion réseau | ❌ | Aucune |

---

## 4. Stack technique

### 4.1 Vue d'ensemble

| Couche | Technologie | Version |
| --- | --- | --- |
| Application de bureau | Tauri | 2 |
| Backend | Rust | édition 2021, MSVC |
| APIs audio | WASAPI via crate `windows` | 0.62 |
| Frontend | React | 19 |
| Langage frontend | TypeScript | 5.7 |
| Bundler | Vite | 6 |
| Icônes | `lucide-react` | 0.468 |
| Styles | CSS écrit à la main | — |
| Persistance | JSON local | — |
| Installateur | NSIS | — |

### 4.2 Dépendances backend

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
windows = { version = "0.62", features = [
  "Win32_Foundation",
  "Win32_Devices_FunctionDiscovery",
  "Win32_Media_Audio",
  "Win32_Media_Audio_Endpoints",
  "Win32_Media_KernelStreaming",
  "Win32_Media_Multimedia",
  "Win32_Security",
  "Win32_System_Com",
  "Win32_System_Com_StructuredStorage",
  "Win32_System_Variant",
  "Win32_System_Registry",
  "Win32_System_Threading",
  "Win32_UI_Shell_PropertiesSystem",
] }
```

Le crate `windows` est utilisé avec des fonctionnalités ciblées afin de limiter le temps
de compilation et la taille du binaire.

### 4.3 Dépendances frontend

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "lucide-react": "^0.468.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@vitejs/plugin-react": "^4.3.4",
    "typescript": "^5.7.2",
    "vite": "^6.0.0"
  }
}
```

Aucun framework UI, aucun Tailwind : les styles sont écrits à la main pour garder un
contrôle total sur l'apparence et éviter une couche d'abstraction supplémentaire.

### 4.4 Choix techniques

- **Tauri plutôt qu'Electron** : binaire plus léger, backend Rust natif, accès direct
  aux APIs Windows via le crate `windows`, meilleure maîtrise de la mémoire.
- **WASAPI plutôt qu'un crate audio tiers** : contrôle précis des interfaces
  d'énumération, de volume et de mètre, sans dépendance supplémentaire.
- **Polling plutôt qu'`IMMNotificationClient`** : la réénumération à 1 s est simple,
  robuste et suffisante pour un usage de bureau. Elle évite la gestion des callbacks
  COM et des cycles de vie associés. Ce choix est assumé et documenté.
- **Un thread audio dédié** : la boucle de capture/rendu ne doit jamais être bloquée par
  l'interface.
- **Atomiques pour le volume et le mute** : aucun verrou dans la boucle temps réel.
- **Bus de niveaux hors React** : les mesures à 25 Hz n'entraînent pas de rendu global
  du composant racine.

---

## 5. Architecture

### 5.1 Vue d'ensemble

```text
┌──────────────────────────────────────────────────────────────┐
│                        Fenêtre Tauri                          │
│                                                              │
│  ┌────────────────────────┐      ┌─────────────────────────┐ │
│  │   Frontend React/TS    │      │      Backend Rust       │ │
│  │                        │      │                         │ │
│  │  AppContext            │ IPC  │  commands.rs            │ │
│  │  Pages / Composants    │◄────►│  state.rs / store.rs    │ │
│  │  meterBus              │events│  permissions.rs         │ │
│  │  ipc.ts                │      │  audio/*                │ │
│  └────────────────────────┘      └───────────┬─────────────┘ │
└──────────────────────────────────────────────┼───────────────┘
                                               │
                                               ▼
                                   ┌─────────────────────┐
                                   │   Windows / WASAPI   │
                                   │  énumération, volume │
                                   │  mètres, capture,    │
                                   │  rendu audio         │
                                   └─────────────────────┘
```

### 5.2 Arborescence

```text
PC-PS5-Sound/
├─ index.html
├─ package.json
├─ package-lock.json
├─ tsconfig.json
├─ vite.config.ts
├─ README.md
├─ LICENSE
├─ .gitignore
├─ src/                          # Frontend React
│  ├─ App.tsx
│  ├─ main.tsx
│  ├─ components/
│  │  ├─ DeviceCard.tsx
│  │  ├─ LevelMeter.tsx
│  │  ├─ ListenBar.tsx
│  │  ├─ PermissionBanner.tsx
│  │  ├─ RenameDialog.tsx
│  │  ├─ Segmented.tsx
│  │  ├─ Sidebar.tsx
│  │  ├─ StatCard.tsx
│  │  ├─ TitleBar.tsx
│  │  ├─ Toasts.tsx
│  │  ├─ Toggle.tsx
│  │  └─ VolumeSlider.tsx
│  ├─ context/
│  │  └─ AppContext.tsx
│  ├─ lib/
│  │  ├─ ipc.ts
│  │  ├─ meterBus.ts
│  │  ├─ types.ts
│  │  └─ useMeter.ts
│  ├─ pages/
│  │  ├─ DashboardPage.tsx
│  │  ├─ MicrophonesPage.tsx
│  │  └─ SettingsPage.tsx
│  └─ styles/
│     ├─ base.css
│     ├─ layout.css
│     ├─ components.css
│     └─ pages.css
└─ src-tauri/                     # Backend Rust
   ├─ Cargo.toml
   ├─ build.rs
   ├─ tauri.conf.json
   ├─ capabilities/
   │  └─ default.json
   ├─ icons/
   └─ src/
      ├─ main.rs
      ├─ lib.rs
      ├─ commands.rs
      ├─ error.rs
      ├─ permissions.rs
      ├─ state.rs
      ├─ store.rs
      └─ audio/
         ├─ mod.rs
         ├─ com.rs
         ├─ types.rs
         ├─ format.rs
         ├─ enumerate.rs
         ├─ monitor.rs
         └─ listen.rs
```

### 5.3 Backend : rôle de chaque module

| Module | Responsabilité |
| --- | --- |
| `main.rs` | Point d'entrée binaire ; appelle `audio_manager_lib::run()`. |
| `lib.rs` | Construit l'application Tauri, initialise l'état, démarre le moniteur, enregistre les commandes et gère la fermeture. |
| `commands.rs` | Surface IPC : validation des entrées, orchestration, renvoi d'erreurs typées. |
| `error.rs` | `AppError { code, message }`, sérialisé vers le frontend. |
| `permissions.rs` | Lecture des clés de consentement microphone et ouverture des paramètres Windows. |
| `state.rs` | État partagé : store, instantané, permission, gestionnaire d'écoute, canal de rafraîchissement, drapeau d'arrêt. |
| `store.rs` | Chargement, validation et sauvegarde atomique du JSON local. |
| `audio/mod.rs` | Regroupe les sous-modules audio. |
| `audio/com.rs` | Garde RAII pour COM, conversions de chaînes, propriétés. |
| `audio/types.rs` | Structures sérialisées : `AudioDevice`, `AudioSnapshot`, `ListenState`, `MeterPayload`, `Notice`. |
| `audio/format.rs` | Analyse de `WAVEFORMATEX(TENSIBLE)`, conversion d'échantillons, format de rendu, tests. |
| `audio/enumerate.rs` | Énumération WASAPI, propriétés, volumes, mètres, états, tri. |
| `audio/monitor.rs` | Thread de surveillance : énumération 1 s, mètres 25 Hz, permission 5 s, détection de disparition. |
| `audio/listen.rs` | Gestionnaire de session unique, thread audio, capture/rendu WASAPI, DSP, anti-larsen. |

### 5.4 Frontend : rôle de chaque module

| Fichier | Responsabilité |
| --- | --- |
| `main.tsx` | Monte React dans `#root`, importe les styles, enveloppe l'app dans `AppProvider`. |
| `App.tsx` | Structure générale, écran de démarrage, navigation entre pages, dock et notifications. |
| `context/AppContext.tsx` | État global, chargement initial, abonnements aux événements, thème, toasts, actions IPC. |
| `lib/ipc.ts` | Passerelle typée : commandes `invoke` et abonnements `listen`. |
| `lib/types.ts` | Types TypeScript partagés avec le backend. |
| `lib/meterBus.ts` | Bus de niveaux à haut débit, abonnement unique à l'événement `audio:meter`. |
| `lib/useMeter.ts` | Hook d'abonnement au bus et utilitaire `clamp`. |
| `components/*` | Composants réutilisables (cartes, vumètres, dock, modale, contrôles). |
| `pages/*` | Trois pages : tableau de bord, microphones, paramètres. |
| `styles/base.css` | Variables de thème, typographie, animations, réduction de mouvement. |
| `styles/layout.css` | Structure : barre de titre, barre latérale, contenu, dock, toasts. |
| `styles/components.css` | Boutons, cartes, vumètres, modales, champs, états vides. |
| `styles/pages.css` | Mise en page spécifique aux pages. |

### 5.5 Modèle de threads

| Thread | Rôle | Cadence |
| --- | --- | --- |
| Thread principal Tauri | Fenêtre, WebView, rendu de l'interface | Événementiel |
| Threads de commandes | Exécution des commandes IPC | À la demande |
| `audio-monitor` | Énumération, mètres, permission | 1 s / 40 ms / 5 s |
| `audio-listen` | Capture et rendu audio | Piloté par événement |

Chaque thread qui utilise COM appelle `CoInitializeEx` en mode multithread via
`ComGuard`, et libère COM à sa fin. Une seule session d'écoute peut exister à la fois.

### 5.6 Flux de données

```text
Windows (WASAPI)
   │
   ├─ énumération + propriétés + volume ─┐
   ├─ IAudioMeterInformation (25 Hz) ────┤
   │                                     ▼
   │                         audio/monitor.rs
   │                                     │
   │                     événements Tauri │
   │                                     ▼
   │                      audio:devices / audio:meter
   │                                     │
   │                                     ▼
   │                         React (AppContext + meterBus)
   │
   └─ capture + rendu ── audio/listen.rs ── listen:state / audio:notice
```

### 5.7 Cycle de vie d'une session d'écoute

1. L'utilisateur clique sur « Écouter » pour un micro.
2. Le frontend appelle `start_listen` avec l'identifiant endpoint.
3. Le backend vérifie l'autorisation microphone et l'état du périphérique.
4. `ListenManager` arrête toute session existante.
5. L'état passe à `starting` et est publié.
6. Un thread `audio-listen` ouvre la capture et le rendu.
7. Une fois les flux démarrés, l'état passe à `listening`.
8. Le thread pompe l'audio jusqu'à l'arrêt, une erreur ou une disparition du micro.
9. À la sortie, l'état passe à `idle` ou `error`, et une notification est émise si
   nécessaire.

---

## 6. Référence IPC

Le frontend ne communique avec le backend que par des commandes `invoke` et des
événements `listen`, tous typés dans `src/lib/ipc.ts`.

### 6.1 Commandes

| Commande | Paramètres | Retour | Description |
| --- | --- | --- | --- |
| `get_devices` | — | `AudioSnapshot` | Renvoie l'instantané courant. |
| `refresh_devices` | — | `AudioSnapshot` | Force une réénumération synchrone. |
| `rename_device` | `deviceId`, `customName` | `AudioSnapshot` | Renomme localement (ou restaure). |
| `reset_custom_names` | — | `AudioSnapshot` | Efface tous les noms personnalisés. |
| `get_listen_state` | — | `ListenState` | État courant de la session d'écoute. |
| `start_listen` | `deviceId` | `ListenState` | Démarre l'écoute d'un micro. |
| `stop_listen` | — | `ListenState` | Arrête la session d'écoute. |
| `set_listen_volume` | `value` | `ListenState` | Volume logiciel 0–100. |
| `set_listen_mute` | `muted` | `ListenState` | Active/désactive le mute logiciel. |
| `get_settings` | — | `Settings` | Préférences locales. |
| `update_settings` | `settings` | `Settings` | Valide et enregistre les préférences. |
| `get_permission` | — | `string` | `granted`, `denied` ou `unknown`. |
| `open_microphone_settings` | — | `void` | Ouvre les paramètres Windows du micro. |
| `get_app_info` | — | `AppInfo` | Version, identifiant, chemin des données. |
| `window_minimize` | — | `void` | Réduit la fenêtre. |
| `window_toggle_maximize` | — | `boolean` | Bascule plein écran/restauré. |
| `window_close` | — | `void` | Ferme la fenêtre. |

### 6.2 Événements

| Événement | Charge utile | Émis par | Fréquence |
| --- | --- | --- | --- |
| `audio:devices` | `AudioSnapshot` | `monitor.rs`, `commands.rs` | À chaque changement |
| `audio:meter` | `MeterPayload` | `monitor.rs` | 25 Hz |
| `listen:state` | `ListenState` | `listen.rs` | À chaque transition / pic |
| `audio:notice` | `Notice` | `monitor.rs`, `listen.rs` | À la demande |

### 6.3 Erreurs

Toute commande peut renvoyer une erreur structurée :

```json
{
  "code": "device_error",
  "message": "Ce microphone n'est plus disponible. Actualisez la liste."
}
```

Codes utilisés :

| Code | Signification |
| --- | --- |
| `invalid_input` | Paramètre invalide (volume non fini, identifiant vide, nom interdit). |
| `device_error` | Périphérique inconnu, indisponible ou non connecté. |
| `audio_error` | Échec WASAPI, format invalide, tampon indisponible, larsen. |
| `permission_denied` | Accès microphone refusé par Windows. |
| `store_write` | Impossible d'écrire les préférences. |
| `settings_open` | Impossible d'ouvrir les paramètres Windows. |
| `internal` | Erreur interne (fenêtre, etc.). |

Côté frontend, `errorMessage()` normalise les erreurs inconnues en message lisible.

---

## 7. Modèles de données

### 7.1 `AudioDevice`

```ts
interface AudioDevice {
  id: string;                     // identifiant endpoint Windows stable
  name: string;                   // nom Windows d'origine
  displayName: string;            // nom affiché (customName ?? name)
  customName: string | null;      // nom local éventuel
  deviceDesc: string;             // description matérielle
  interfaceName: string;          // interface hôte
  state: DeviceState | string;    // active | unplugged | disabled | notpresent | unknown
  stateLabel: string;             // libellé français
  isDefault: boolean;
  isDefaultCommunication: boolean;
  volumePercent: number;          // volume de capture Windows (lecture seule)
  muted: boolean;                 // mute Windows (lecture seule)
  formFactor: string;
  formFactorLabel: string;
}
```

### 7.2 `AudioSnapshot`

```ts
interface AudioSnapshot {
  devices: AudioDevice[];
  defaultOutput: string | null;
  defaultOutputFormFactor: string;
  permission: string;             // granted | denied | unknown
  monitorStatus: string;          // starting | running | error
  capturedAt: number;             // horodatage Unix en millisecondes
}
```

### 7.3 `ListenState`

```ts
interface ListenState {
  active: boolean;
  deviceId: string | null;
  deviceName: string | null;
  volume: number;
  muted: boolean;
  outputName: string | null;
  peak: number;
  feedbackGuard: boolean;
  status: "idle" | "starting" | "listening" | "stopping" | "error" | string;
  error: string | null;
  startedAt: number | null;
}
```

### 7.4 `Settings`

```ts
interface Settings {
  theme: "dark" | "light" | "system";
  listenVolume: number;           // 0–100
  feedbackGuard: boolean;
  showInactive: boolean;
  reduceMotion: boolean;
  compactCards: boolean;
}
```

### 7.5 Format du fichier de persistance

```json
{
  "customNames": {
    "\\?\\SWD#MMDEVAPI#{0.0.1.00000000}.{...}": "Micro studio"
  },
  "settings": {
    "theme": "dark",
    "listenVolume": 80,
    "feedbackGuard": true,
    "showInactive": true,
    "reduceMotion": false,
    "compactCards": false
  }
}
```

Le nom des champs est en camelCase (`serde(rename_all = "camelCase")`) afin de rester
cohérent entre Rust et TypeScript.

---

## 8. Design system

### 8.1 Thèmes

Le thème est appliqué via l'attribut `data-theme` sur `<html>` :

- `dark` (par défaut) ;
- `light` ;
- `system` (résolu côté frontend selon `prefers-color-scheme`).

Le mode `system` suit les changements du système en direct.

### 8.2 Variables CSS

Le design repose sur des variables CSS centralisées dans `base.css` :

| Variable | Usage |
| --- | --- |
| `--bg`, `--bg-soft` | Fonds principaux |
| `--surface`, `--surface-solid`, `--surface-2`, `--surface-3` | Surfaces et cartes |
| `--border`, `--border-strong` | Bordures |
| `--text`, `--text-soft`, `--text-dim` | Hiérarchie de texte |
| `--accent`, `--accent-2`, `--accent-3` | Couleurs d'accent |
| `--accent-soft`, `--accent-border` | Accent atténué |
| `--success`, `--warning`, `--danger` | États |
| `--glass`, `--glass-strong` | Effets de transparence |
| `--meter-track`, `--range-track` | Vumètres et curseurs |
| `--shadow-md`, `--shadow-lg`, `--glow` | Ombres et halos |

Les deux thèmes redéfinissent ces variables, ce qui permet de changer d'apparence sans
dupliquer les règles de composants.

### 8.3 Composants

- `TitleBar` : barre de titre personnalisée, glissement natif, double-clic maximiser.
- `Sidebar` : navigation, session en cours, bascule de thème, version.
- `DeviceCard` : carte de micro, vumètre, état, actions, panneau d'écoute.
- `LevelMeter` : vumètre segmenté animé.
- `VolumeSlider` : curseur de volume avec remplissage dynamique.
- `Toggle` : interrupteur animé.
- `Segmented` : sélecteur segmenté (thème, filtres).
- `StatCard` : carte de statistique.
- `PermissionBanner` : bandeau d'autorisation microphone.
- `RenameDialog` : modale de renommage local.
- `ListenBar` : dock d'écoute.
- `Toasts` : notifications.

### 8.4 Animations

Animations définies : `fade-up`, `pop`, `slide-in`, `dock-in`, `spin`, `pulse-dot`,
`live-glow`, `soft-breathe`, `boot-slide`.

Deux mécanismes de réduction :

1. `@media (prefers-reduced-motion: reduce)` ;
2. `data-motion="reduced"` lorsque l'utilisateur active « Animations réduites ».

Dans les deux cas, les durées d'animation et de transition sont ramenées à une valeur
quasi nulle.

### 8.5 Accessibilité

- Contraste soigné dans les deux thèmes.
- Focus visible (`:focus-visible`) avec contour d'accent.
- Boutons avec `aria-label` pour les icônes seules.
- Navigation clavier prise en charge par les éléments natifs (`button`, `input`).
- Textes alternatifs et libellés explicites.
- Respect de la réduction de mouvement.

---

## 9. Prérequis

### 9.1 Système

- **Windows 10 ou Windows 11** (x64).
- Un périphérique d'entrée audio (micro, casque, webcam…).
- Un périphérique de sortie actif pour l'écoute.

### 9.2 Outils de développement

| Outil | Version conseillée | Vérification |
| --- | --- | --- |
| Rust (rustup) | stable MSVC | `rustc --version` |
| Cargo | fourni avec Rust | `cargo --version` |
| Microsoft C++ Build Tools | récent | installation Visual Studio Build Tools |
| Node.js | 20+ (testé 24) | `node --version` |
| npm | 10+ (testé 11) | `npm --version` |
| WebView2 Runtime | à jour | généralement préinstallé |

### 9.3 Vérification de l'environnement

```powershell
rustc --version
cargo --version
node --version
npm --version
rustup show
```

La toolchain doit être `stable-x86_64-pc-windows-msvc`.

### 9.4 NSIS

NSIS n'est pas à installer manuellement : Tauri le télécharge automatiquement lors du
premier build d'installateur.

---

## 10. Installation

### 10.1 Récupérer le projet

```powershell
git clone <url-du-depot> PC-PS5-Sound
cd PC-PS5-Sound
```

### 10.2 Construire l'application depuis les sources

```powershell
npm install
npm run build
cd src-tauri && cargo build --release
```

> `npm run build` compile le frontend React dans le dossier `dist`, nécessaire à la
> compilation Rust. Sans cette étape, `cargo build` échoue sur un clone neuf avec
> l'erreur `frontendDist ... doesn't exist`.

Une fois la compilation terminée, l'exécutable se trouve dans :

```text
src-tauri/target/release/audio-manager.exe
```

Lancez-le directement : c'est l'application complète, sans installation.

### 10.3 Ou installer l'installateur Windows

Si vous préférez une installation classique, générez puis lancez l'installateur NSIS :

```powershell
npm run tauri build
```

L'installateur est créé ici :

```text
src-tauri/target/release/bundle/nsis/Audio Manager_0.1.0_x64-setup.exe
```

Double-cliquez dessus pour installer Audio Manager (installation par utilisateur, en
français).

### 10.4 Lancer en développement

```powershell
npm run tauri dev
```

Cette commande démarre Vite, compile le backend et lance la fenêtre Tauri. La première
compilation peut prendre plusieurs minutes.

### 10.5 Premier lancement

1. Au démarrage, l'application charge les préférences locales.
2. Le moniteur énumère les microphones et publie un premier instantané.
3. Si Windows n'autorise pas l'accès au microphone, un bandeau apparaît.
4. Le bouton du bandeau ouvre directement
   *Paramètres → Confidentialité et sécurité → Microphone*.
5. Les microphones apparaissent avec leur état et leur niveau.

---

## 11. Commandes

| Commande | Description |
| --- | --- |
| `npm install` | Installe les dépendances frontend. |
| `npm run dev` | Démarre Vite seul (frontend dans le navigateur, sans backend). |
| `npm run tauri dev` | Lance l'application Tauri complète en développement. |
| `npm run build` | Vérifie TypeScript (`tsc --noEmit`) et construit le frontend. |
| `npm run preview` | Prévisualise le build Vite. |
| `npm run tauri build` | Construit l'exécutable et l'installateur Windows. |
| `npm run tauri build -- --no-bundle` | Construit uniquement l'exécutable. |
| `cargo check` | Vérifie la compilation Rust. |
| `cargo build --release` | Compile le backend en mode optimisé. |
| `cargo clippy --all-targets -- -D warnings` | Analyse statique stricte. |
| `cargo test` | Lance les tests unitaires Rust. |

---

## 12. Compilation et distribution

### 12.1 Debug vs release

| Profil | Commande | Sortie |
| --- | --- | --- |
| Debug | `cargo build` | `src-tauri/target/debug/audio-manager.exe` |
| Release | `cargo build --release` | `src-tauri/target/release/audio-manager.exe` |
| Tauri | `npm run tauri build` | exécutable + installateur |

Le mode release active les optimisations et le profil défini dans `Cargo.toml`.

### 12.2 Exécutable Windows

Après `cd src-tauri && cargo build --release`, l'exécutable autonome se trouve à :

```text
src-tauri/target/release/audio-manager.exe
```

Il s'agit d'une application native x64. Elle nécessite le runtime WebView2, présent par
défaut sur Windows 10 récent et Windows 11.

Pour générer l'exécutable via Tauri (frontend + backend), vous pouvez aussi utiliser :

```powershell
npm run tauri build -- --no-bundle
```

### 12.3 Installateur NSIS

L'installateur Windows est généré à :

```text
src-tauri/target/release/bundle/nsis/Audio Manager_0.1.0_x64-setup.exe
```

Caractéristiques du bundle :

- cible **NSIS** ;
- mode d'installation **par utilisateur** (`currentUser`) ;
- langue **français** ;
- icônes multi-résolutions fournies dans `src-tauri/icons`.

### 12.4 Profil de release

```toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

Objectifs : binaire compact, démarrage rapide, pas de symboles inutiles.

### 12.5 Distribution portable

L'exécutable `audio-manager.exe` peut être copié et lancé tel quel sur une machine
Windows disposant de WebView2. Les préférences restent stockées dans le profil de
l'utilisateur (`%APPDATA%`).

### 12.6 Signature de code

Le projet ne signe pas automatiquement l'exécutable. Pour une distribution publique,
il est recommandé de signer le binaire et l'installateur avec un certificat de signature
de code afin d'éviter les avertissements SmartScreen.

---

## 13. Persistance et vie privée

### 13.1 Emplacement

```text
%APPDATA%\com.audiomanager.desktop\audio-manager.json
```

### 13.2 Contenu

- `customNames` : association `identifiant endpoint Windows → nom local` ;
- `settings` : thème, volume d'écoute, protection anti-larsen, affichage des
  périphériques inactifs, animations réduites, cartes compactes.

### 13.3 Écriture atomique

La sauvegarde suit ce schéma :

1. sérialisation JSON ;
2. écriture dans `audio-manager.json.tmp` ;
3. `sync_all()` pour forcer l'écriture disque ;
4. suppression de l'ancien fichier si présent ;
5. renommage du fichier temporaire vers le fichier final.

Sur Windows, `rename` ne remplace pas un fichier existant : la suppression préalable est
donc nécessaire.

### 13.4 Corruption

Si le fichier JSON est illisible :

- il est renommé en `audio-manager.json.bak` ;
- une configuration par défaut est utilisée ;
- l'application continue de fonctionner.

### 13.5 Validation

Avant sauvegarde :

- le thème est normalisé (`dark`, `light`, `system`, sinon `dark`) ;
- le volume est borné entre 0 et 100 ;
- les valeurs non finies (`NaN`, `Infinity`) sont remplacées par la valeur par défaut ;
- les noms vides sont ignorés ;
- les identifiants vides sont rejetés ;
- les noms personnalisés sont limités à 64 caractères ;
- les caractères de contrôle sont refusés.

### 13.6 Garanties de confidentialité

- **Aucune donnée ne quitte la machine.**
- Aucun appel réseau n'est effectué par l'application.
- Les identifiants d'endpoint Windows sont conservés tels quels en interne ; seuls les
  noms affichés changent.
- Les noms personnalisés ne sont **jamais** écrits dans le registre ni dans les
  propriétés du périphérique.
- Aucun accès en écriture aux réglages Windows.

---

## 14. Explication technique

### 14.1 COM et threads

WASAPI repose sur COM. Chaque thread qui utilise les interfaces audio initialise COM en
mode multithread (`COINIT_MULTITHREADED`) via un garde RAII `ComGuard`.

Le garde ne libère COM que si l'initialisation a réussi (`HRESULT >= 0`), afin d'éviter
un déséquilibre en cas de mode déjà initialisé différemment.

### 14.2 Énumération des périphériques

Le backend crée un `IMMDeviceEnumerator` puis appelle :

```text
EnumAudioEndpoints(eCapture, DEVICE_STATEMASK_ALL)
```

Tous les états sont demandés afin de pouvoir afficher les périphériques désactivés ou
absents.

Pour chaque endpoint :

- `GetId()` fournit l'identifiant stable ;
- `OpenPropertyStore(STGM_READ)` lit :
  - `PKEY_Device_FriendlyName` ;
  - `PKEY_Device_DeviceDesc` ;
  - `PKEY_DeviceInterface_FriendlyName` ;
  - `PKEY_AudioEndpoint_FormFactor` ;
- `GetState()` détermine l'état ;
- `IAudioEndpointVolume` lit le volume et le mute (jamais modifiés) ;
- `IAudioMeterInformation` fournit le pic temps réel.

Les périphériques sont triés : actifs d'abord, puis déconnectés, désactivés, et enfin
non présents, par ordre alphabétique.

### 14.3 États des périphériques

| État WASAPI | Libellé | Signification |
| --- | --- | --- |
| `DEVICE_STATE_ACTIVE` | Connecté | Disponible |
| `DEVICE_STATE_UNPLUGGED` | Déconnecté | Physiquement retiré |
| `DEVICE_STATE_DISABLED` | Désactivé | Désactivé dans Windows |
| `DEVICE_STATE_NOTPRESENT` | Non présent | Endpoint fantôme |
| autre | Inconnu | État non reconnu |

### 14.4 Niveaux temps réel

Un thread dédié conserve les interfaces `IAudioMeterInformation` des périphériques
actifs et lit `GetPeakValue()` toutes les 40 ms, soit 25 Hz.

Les valeurs sont bornées entre 0 et 1 et diffusées via l'événement `audio:meter`.

Côté frontend, `meterBus` :

- s'abonne une seule fois à l'événement ;
- conserve un ensemble d'écouteurs ;
- notifie chaque écouteur à chaque paquet.

Les vumètres peuvent ainsi s'animer à haute fréquence sans provoquer de rendu React
global.

### 14.5 Pipeline d'écoute WASAPI

1. Ouverture du périphérique de capture en mode **partagé événementiel**.
2. Lecture du format de mixage natif via `GetMixFormat()`.
3. Création d'un événement Win32 pour la capture.
4. Initialisation de la capture avec `AUDCLNT_STREAMFLAGS_EVENTCALLBACK`.
5. Ouverture du périphérique de rendu par défaut (`eRender`, `eConsole`).
6. Construction d'un format de rendu `float32` stéréo à la fréquence du micro.
7. Initialisation du rendu avec :
   - `AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM` ;
   - `AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY` ;
   - un tampon de 40 ms (`400 000` unités de 100 ns).
8. Démarrage des deux flux.
9. Boucle pilotée par événement :
   - attente du paquet de capture (timeout 20 ms) ;
   - lecture des paquets disponibles ;
   - gestion du flag `AUDCLNT_BUFFERFLAGS_SILENT` ;
   - conversion vers `float32` stéréo ;
   - calcul du pic d'entrée ;
   - gain logiciel et mute ;
   - filtre passe-haut à 80 Hz ;
   - limiteur doux ;
   - détection de larsen ;
   - écriture dans le tampon de rendu ;
   - complément par du silence si nécessaire ;
   - publication du pic à 10 Hz.
10. Arrêt propre des flux, libération des ressources et publication de l'état final.

### 14.6 Conversion des formats

La fonction `frames_to_stereo_f32` gère :

- PCM 8 bits non signé ;
- PCM 16 bits signé ;
- PCM 24 bits empaqueté ;
- PCM 32 bits signé ;
- flottants 32 bits.

Règles :

- le pas de trame est `block_align` (avec un minimum de `channels × bytes_per_sample`) ;
- un signal mono est dupliqué sur les deux canaux ;
- un signal multicanal utilise les deux premiers canaux ;
- les échantillons sont normalisés entre -1 et 1.

### 14.7 Traitement du signal

- **Gain** : `volume / 100`, appliqué après le filtre.
- **Mute** : sortie à zéro, sans couper le flux.
- **Passe-haut 80 Hz** : filtre one-pole, coefficients calculés à partir de la fréquence
  d'échantillonnage.
- **Limiteur doux** : au-delà du seuil, la sortie est comprimée via `tanh` puis bornée
  à 1.
- **Anti-larsen** : voir section 14.8.

### 14.8 Protection anti-larsen

Lorsque la sortie par défaut est un haut-parleur et que la protection est active :

- le gain effectif est plafonné à 60 % ;
- le seuil du limiteur passe de 0,98 à 0,86 ;
- si le signal d'entrée reste saturé (pic > 0,995) pendant plus de 3 secondes avec un
  gain non négligeable, la session est arrêtée et une notification explique la cause.

Le filtre passe-haut à 80 Hz atténue par ailleurs les basses fréquences responsables du
sifflement.

### 14.9 Autorisations microphone

Windows stocke le consentement dans :

```text
HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone
```

Deux valeurs sont lues :

- la valeur globale `Value` ;
- la valeur `NonPackaged\Value`, qui concerne les applications de bureau non empaquetées.

Règles appliquées :

- si l'une des deux vaut `Deny` → accès **refusé** ;
- si l'une des deux vaut `Allow` → accès **accordé** ;
- sinon → état **inconnu** (Windows demandera l'autorisation à l'usage).

L'interface affiche un bandeau explicite et propose un bouton qui ouvre directement
`ms-settings:privacy-microphone`.

### 14.10 Renommage local

Le renommage ne modifie que le store JSON :

- l'identifiant endpoint Windows reste la clé stable ;
- le nom Windows d'origine reste disponible dans le champ `name` ;
- le nom affiché est calculé à la volée (`displayName`) ;
- les noms sont limités à 64 caractères et les caractères de contrôle sont refusés ;
- un nom vide supprime le nom personnalisé.

### 14.11 Gestion des erreurs

Toutes les commandes renvoient soit une valeur typée, soit un objet
`{ code, message }`. Le frontend les présente sous forme de notifications lisibles.

Les verrous partagés sont tolérants aux empoisonnements : une erreur dans un thread ne
bloque pas l'application entière.

### 14.12 Sûreté Rust

- Interfaces COM encapsulées et libérées via RAII.
- Handles Win32 (`HANDLE`) encapsulés dans `EventHandle` avec `Drop`.
- Accès aux structures `packed(1)` via `read_unaligned`.
- Aucun `unwrap()` non maîtrisé sur les chemins critiques.
- Atomiques pour les paramètres temps réel.
- Threads nommés pour faciliter le diagnostic.

---

## 15. Performances

| Opération | Fréquence / coût | Commentaire |
| --- | --- | --- |
| Énumération complète | 1 s | Réénumération WASAPI + propriétés |
| Lecture des mètres | 25 Hz | `GetPeakValue` par périphérique actif |
| Vérification permission | 5 s | Lecture du registre |
| Publication du pic d'écoute | 10 Hz | Événement `listen:state` |
| Tampon de rendu | 40 ms | Latence faible |
| Timeout d'événement capture | 20 ms | Évite un blocage prolongé |
| Buffer de niveaux frontend | 25 Hz | Hors rendu React |

Le frontend évite les re-rendus inutiles :

- `meterBus` notifie directement les vumètres ;
- les niveaux ne passent pas par l'état global React ;
- les pages sont rendues à la demande.

Le binaire de release est compilé avec LTO, `opt-level = "s"` et `strip = true`.

---

## 16. Sécurité

- **Aucune connexion réseau.**
- **Aucune télémétrie.**
- **Aucun compte.**
- **Aucune élévation de privilèges.**
- **Aucune écriture dans les réglages Windows.**
- Lecture seule du registre de consentement microphone.
- Validation stricte des entrées IPC.
- Erreurs typées, sans fuite de détails internes sensibles.
- Capabilities Tauri minimales :
  - `core:default` ;
  - `core:window:allow-start-dragging`.
- CSP désactivée dans `tauri.conf.json` pour le développement local ; à durcir si
  l'application est distribuée publiquement avec des contenus distants (ce qui n'est
  pas le cas ici).

---

## 17. Dépannage et FAQ

### Aucun microphone n'apparaît

- Vérifier *Paramètres → Système → Son → Entrée*.
- Autoriser l'accès au microphone dans
  *Paramètres → Confidentialité et sécurité → Microphone*.
- Vérifier que les périphériques ne sont pas désactivés dans le Gestionnaire de
  périphériques.
- Utiliser le bouton de rafraîchissement de l'application.
- Vérifier que le service audio Windows est démarré.

### L'écoute ne démarre pas

- Vérifier qu'un périphérique de sortie est disponible et actif.
- Contrôler l'autorisation microphone (bandeau dans l'application).
- Fermer les applications qui capturent déjà le micro en mode exclusif.
- Vérifier que le micro est à l'état `Connecté`.

### Le vumètre reste à zéro

- Vérifier le volume de capture Windows (affiché en lecture seule).
- Vérifier que le micro n'est pas muet dans Windows.
- Parler ou produire un son pendant la mesure.
- Certains pilotes exposent un mètre qui ne fonctionne qu'en cours de capture.

### Larsen ou sifflement

- Utiliser un casque plutôt que des haut-parleurs.
- Laisser la protection anti-larsen activée.
- Baisser le volume d'écoute logiciel.
- Éloigner le micro des haut-parleurs.

### Le microphone disparaît pendant l'écoute

- L'application arrête automatiquement la session et affiche une notification.
- Rebrancher le périphérique puis relancer l'écoute.

### Erreur de compilation Rust

- Vérifier l'installation des C++ Build Tools.
- Vérifier que la toolchain MSVC est active :

```powershell
rustup show
```

### `cargo build` échoue : `frontendDist` introuvable

Le frontend doit être compilé avant le backend :

```powershell
npm run build
cd src-tauri && cargo build --release
```

### L'installateur ne se génère pas

- Vérifier la connexion réseau : Tauri télécharge NSIS lors du premier build.
- Relancer `npm run tauri build`.
- Vérifier que `npm run build` réussit avant le bundling.

### Où sont stockées mes préférences ?

```text
%APPDATA%\com.audiomanager.desktop\audio-manager.json
```

Supprimer ce fichier réinitialise l'application.

### Les noms personnalisés modifient-ils Windows ?

Non. Ils sont stockés uniquement dans le fichier JSON de l'application. Le nom Windows
d'origine et l'identifiant endpoint restent intacts.

### L'application envoie-t-elle des données ?

Non. Aucune connexion réseau n'est effectuée.

---

## 18. Développement et contribution

### 18.1 Conventions

- Rust : `rustfmt`, `clippy -D warnings`, pas de `unsafe` non justifié.
- TypeScript : mode strict, types explicites aux frontières IPC.
- CSS : variables de thème, pas de valeurs codées en dur dans les composants.
- Nommage : français pour les messages utilisateur, anglais pour les identifiants
  techniques.

### 18.2 Vérifications avant commit

```powershell
npm run build
cd src-tauri
cargo clippy --all-targets -- -D warnings
cargo test
```

### 18.3 Tests

Les tests unitaires couvrent :

- la conversion des formats audio (i16, mono, 24 bits, `block_align`, float32) ;
- la construction du format de rendu ;
- la validation et la normalisation des préférences ;
- la gestion des noms personnalisés ;
- la validation des identifiants de périphérique.

```powershell
cd src-tauri
cargo test
```

### 18.4 Ajouter une fonctionnalité

1. Définir le modèle côté Rust (`audio/types.rs` ou `store.rs`).
2. Ajouter la commande dans `commands.rs` et l'enregistrer dans `lib.rs`.
3. Ajouter la méthode typée dans `src/lib/ipc.ts`.
4. Exposer l'action dans `AppContext.tsx`.
5. Construire l'interface dans `components/` ou `pages/`.
6. Ajouter les styles dans le fichier CSS approprié.
7. Vérifier avec `npm run build`, `cargo clippy` et `cargo test`.

### 18.5 Organisation du dépôt

- `src/` : frontend uniquement.
- `src-tauri/src/` : backend uniquement.
- Aucun accès direct au système depuis le frontend.
- Aucune logique d'interface dans le backend.

---

## 19. Feuille de route

Pistes envisagées, non implémentées à ce jour :

- notifications natives de changement de périphérique (`IMMNotificationClient`) en
  complément du polling ;
- sélection explicite du périphérique de sortie pour l'écoute ;
- égaliseur graphique ;
- enregistrement du signal traité dans un fichier WAV ;
- profils de volume par microphone ;
- raccourcis clavier globaux ;
- démarrage automatique avec Windows ;
- traduction anglaise de l'interface.

Ces éléments sont des idées et ne doivent pas être considérés comme des fonctionnalités
présentes.

---

## 20. Licence

Ce projet est distribué sous la licence présente dans le fichier [`LICENSE`](./LICENSE).
