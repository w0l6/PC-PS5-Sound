use crate::audio::com::ComGuard;
use crate::audio::enumerate::create_enumerator;
use crate::audio::format::{build_render_format, frames_to_stereo_f32, parse_format, SampleFormat};
use crate::audio::types::{now_ms, ListenState, Notice};
use crate::error::{AppError, AppResult};
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Media::Audio::{
    eConsole, eRender, IAudioCaptureClient, IAudioClient, IAudioRenderClient,
    AUDCLNT_BUFFERFLAGS_SILENT, AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM,
    AUDCLNT_STREAMFLAGS_EVENTCALLBACK, AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
};
use windows::Win32::System::Com::{CoTaskMemFree, CLSCTX_ALL};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

const BUFFER_DURATION_100NS: i64 = 400_000; // 40 ms : latence faible et stable
const METER_INTERVAL: Duration = Duration::from_millis(100);
const FEEDBACK_WINDOW_SECONDS: f32 = 3.0;

struct Session {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

/// Gestionnaire de la session d'ecoute unique.
/// Un seul microphone peut etre ecoute a la fois : demarrer une nouvelle
/// session arrete proprement la precedente.
pub struct ListenManager {
    session: Mutex<Option<Session>>,
    volume: AtomicU32,
    muted: AtomicBool,
    guard: AtomicBool,
    state: Mutex<ListenState>,
}

impl ListenManager {
    pub fn new(volume: f32, feedback_guard: bool) -> Self {
        let volume = volume.clamp(0.0, 100.0);
        let state = ListenState {
            volume,
            feedback_guard,
            ..ListenState::default()
        };
        Self {
            session: Mutex::new(None),
            volume: AtomicU32::new(volume.to_bits()),
            muted: AtomicBool::new(false),
            guard: AtomicBool::new(feedback_guard),
            state: Mutex::new(state),
        }
    }

    fn state_lock(&self) -> MutexGuard<'_, ListenState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn session_lock(&self) -> MutexGuard<'_, Option<Session>> {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn volume_value(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    pub fn muted(&self) -> bool {
        self.muted.load(Ordering::Relaxed)
    }

    pub fn guard_enabled(&self) -> bool {
        self.guard.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> ListenState {
        self.state_lock().clone()
    }

    fn publish(&self, app: &AppHandle) -> ListenState {
        let state = self.snapshot();
        let _ = app.emit("listen:state", &state);
        state
    }

    pub fn active_device(&self) -> Option<String> {
        let state = self.state_lock();
        if state.active {
            state.device_id.clone()
        } else {
            None
        }
    }

    /// Demarre l'ecoute d'un peripherique. Toute session existante est arretee.
    pub fn start(
        self: &Arc<Self>,
        app: &AppHandle,
        device_id: String,
        device_name: String,
        output_name: Option<String>,
        output_is_speakers: bool,
    ) -> AppResult<ListenState> {
        self.stop_session();

        {
            let mut state = self.state_lock();
            state.active = true;
            state.status = "starting".to_string();
            state.device_id = Some(device_id.clone());
            state.device_name = Some(device_name);
            state.output_name = output_name;
            state.volume = self.volume_value();
            state.muted = self.muted.load(Ordering::Relaxed);
            state.feedback_guard = self.guard.load(Ordering::Relaxed);
            state.peak = 0.0;
            state.error = None;
            state.started_at = None;
        }
        let starting = self.publish(app);

        let stop = Arc::new(AtomicBool::new(false));
        let manager = Arc::clone(self);
        let thread_app = app.clone();
        let thread_stop = Arc::clone(&stop);
        let thread_device = device_id.clone();

        let handle = std::thread::Builder::new()
            .name("audio-listen".to_string())
            .spawn(move || {
                let result = run_pipeline(
                    &manager,
                    &thread_app,
                    &thread_device,
                    output_is_speakers,
                    &thread_stop,
                );
                manager.on_exit(&thread_app, result);
            })
            .map_err(|error| {
                AppError::audio(format!(
                    "Impossible de demarrer le moteur audio : {error}"
                ))
            })?;

        *self.session_lock() = Some(Session {
            stop,
            handle: Some(handle),
        });

        Ok(starting)
    }

    /// Arrete la session courante et attend la fin du thread audio.
    pub fn stop(&self, app: &AppHandle) -> ListenState {
        {
            let mut state = self.state_lock();
            if state.active || state.status == "starting" {
                state.status = "stopping".to_string();
            }
        }
        let _ = app.emit("listen:state", &self.snapshot());

        self.stop_session();

        {
            let mut state = self.state_lock();
            state.active = false;
            state.status = "idle".to_string();
            state.peak = 0.0;
            state.started_at = None;
            state.error = None;
            state.device_id = None;
            state.device_name = None;
            state.output_name = None;
        }
        self.publish(app)
    }

    fn stop_session(&self) {
        let session = self.session_lock().take();
        if let Some(mut session) = session {
            session.stop.store(true, Ordering::SeqCst);
            if let Some(handle) = session.handle.take() {
                let _ = handle.join();
            }
        }
    }

    pub fn set_volume(&self, app: &AppHandle, value: f32) -> ListenState {
        let value = if value.is_finite() {
            value.clamp(0.0, 100.0)
        } else {
            self.volume_value()
        };
        self.volume.store(value.to_bits(), Ordering::Relaxed);
        {
            let mut state = self.state_lock();
            state.volume = value;
        }
        self.publish(app)
    }

    pub fn set_muted(&self, app: &AppHandle, muted: bool) -> ListenState {
        self.muted.store(muted, Ordering::Relaxed);
        {
            let mut state = self.state_lock();
            state.muted = muted;
        }
        self.publish(app)
    }

    pub fn set_guard(&self, app: &AppHandle, enabled: bool) -> ListenState {
        self.guard.store(enabled, Ordering::Relaxed);
        {
            let mut state = self.state_lock();
            state.feedback_guard = enabled;
        }
        self.publish(app)
    }

    fn mark_listening(&self, app: &AppHandle) {
        {
            let mut state = self.state_lock();
            if state.status != "starting" {
                return;
            }
            state.active = true;
            state.status = "listening".to_string();
            state.started_at = Some(now_ms());
            state.peak = 0.0;
        }
        self.publish(app);
    }

    fn update_peak(&self, app: &AppHandle, peak: f32) {
        {
            let mut state = self.state_lock();
            if !state.active {
                return;
            }
            state.peak = if peak.is_finite() {
                peak.clamp(0.0, 1.0)
            } else {
                0.0
            };
        }
        let state = self.snapshot();
        let _ = app.emit("listen:state", &state);
    }

    fn on_exit(&self, app: &AppHandle, result: AppResult<()>) {
        let mut state = self.state_lock();
        if matches!(state.status.as_str(), "stopping" | "idle") {
            return;
        }

        match result {
            Ok(()) => {
                state.active = false;
                state.status = "idle".to_string();
                state.peak = 0.0;
                state.started_at = None;
                let snapshot = state.clone();
                drop(state);
                let _ = app.emit("listen:state", &snapshot);
            }
            Err(error) => {
                state.active = false;
                state.status = "error".to_string();
                state.peak = 0.0;
                state.error = Some(error.message.clone());
                let snapshot = state.clone();
                drop(state);
                let _ = app.emit("listen:state", &snapshot);
                let _ = app.emit(
                    "audio:notice",
                    &Notice {
                        kind: "error".to_string(),
                        message: error.message,
                    },
                );
            }
        }
    }
}

/// Garde RAII autour du handle d'evenement WASAPI.
struct EventHandle(HANDLE);

impl Drop for EventHandle {
    fn drop(&mut self) {
        if !self.0.0.is_null() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

fn hpf_alpha(cutoff: f32, sample_rate: f32) -> f32 {
    let dt = 1.0 / sample_rate.max(1.0);
    let rc = 1.0 / (std::f32::consts::TAU * cutoff);
    rc / (rc + dt)
}

fn soft_limit(value: f32, threshold: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    let magnitude = value.abs();
    if magnitude <= threshold {
        return value;
    }
    let range = (1.0 - threshold).max(0.0001);
    let limited = threshold + range * ((magnitude - threshold) / range).tanh();
    value.signum() * limited.min(1.0)
}

/// Traitement bas niveau : gain logiciel, mute, filtre passe-haut 80 Hz
/// et limiteur doux. Ne touche jamais au volume de capture Windows.
fn process(
    samples: &mut [f32],
    gain: f32,
    muted: bool,
    alpha: f32,
    hpf: &mut [f32; 2],
    previous: &mut [f32; 2],
    threshold: f32,
) {
    for frame in samples.chunks_exact_mut(2) {
        for channel in 0..2 {
            let input = frame[channel];
            let filtered = alpha * (hpf[channel] + input - previous[channel]);
            previous[channel] = input;
            hpf[channel] = filtered;
            let value = if muted { 0.0 } else { filtered * gain };
            frame[channel] = soft_limit(value, threshold);
        }
    }
}

/// Ouvre les flux WASAPI de capture et de rendu, puis pompe l'audio
/// jusqu'a l'arret demande ou jusqu'a une erreur (peripherique retire).
fn run_pipeline(
    manager: &Arc<ListenManager>,
    app: &AppHandle,
    device_id: &str,
    output_is_speakers: bool,
    stop: &Arc<AtomicBool>,
) -> AppResult<()> {
    let _com = ComGuard::mta();
    let enumerator = create_enumerator()?;

    let device = unsafe { enumerator.GetDevice(&HSTRING::from(device_id))? };
    let capture_audio: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None)? };

    let mix = unsafe { capture_audio.GetMixFormat()? };
    if mix.is_null() {
        return Err(AppError::audio(
            "Le format audio du microphone est introuvable.",
        ));
    }
    let capture_format = unsafe { parse_format(mix) };
    if !capture_format.valid() {
        unsafe { CoTaskMemFree(Some(mix as *const c_void)) };
        return Err(AppError::audio(
            "Le format audio du microphone est invalide.",
        ));
    }

    let event = EventHandle(unsafe { CreateEventW(None, false, false, PCWSTR::null())? });

    // En mode partage evenementiel, la duree de tampon est choisie par le moteur.
    unsafe {
        capture_audio.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
            0,
            0,
            mix,
            None,
        )?;
    }
    unsafe { CoTaskMemFree(Some(mix as *const c_void)) };
    unsafe { capture_audio.SetEventHandle(event.0)? };
    let capture: IAudioCaptureClient = unsafe { capture_audio.GetService()? };

    let render_device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole)? };
    let render_audio: IAudioClient = unsafe { render_device.Activate(CLSCTX_ALL, None)? };
    let render_format = build_render_format(capture_format.sample_rate);

    // AUTOCONVERTPCM + SRC_DEFAULT_QUALITY laissent Windows convertir le
    // float32 stereo vers le format materiel de sortie.
    unsafe {
        render_audio.Initialize(
            AUDCLNT_SHAREMODE_SHARED,
            AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
            BUFFER_DURATION_100NS,
            0,
            std::ptr::addr_of!(render_format.Format),
            None,
        )?;
    }
    let render: IAudioRenderClient = unsafe { render_audio.GetService()? };
    let render_frames = unsafe { render_audio.GetBufferSize()? };
    if render_frames == 0 {
        return Err(AppError::audio(
            "Le peripherique de sortie audio n'a pas de tampon disponible.",
        ));
    }

    if stop.load(Ordering::SeqCst) {
        return Ok(());
    }

    unsafe {
        capture_audio.Start()?;
        render_audio.Start()?;
    }
    manager.mark_listening(app);

    let result = pump(
        manager,
        app,
        &capture,
        &render_audio,
        &render,
        render_frames,
        &capture_format,
        &event,
        output_is_speakers,
        stop,
    );

    unsafe {
        let _ = render_audio.Stop();
        let _ = capture_audio.Stop();
    }

    result
}

#[allow(clippy::too_many_arguments)]
fn pump(
    manager: &Arc<ListenManager>,
    app: &AppHandle,
    capture: &IAudioCaptureClient,
    render_audio: &IAudioClient,
    render: &IAudioRenderClient,
    render_frames: u32,
    format: &SampleFormat,
    event: &EventHandle,
    output_is_speakers: bool,
    stop: &Arc<AtomicBool>,
) -> AppResult<()> {
    let sample_rate = format.sample_rate.max(1) as f32;
    let alpha = hpf_alpha(80.0, sample_rate);
    let mut hpf = [0.0f32; 2];
    let mut previous = [0.0f32; 2];
    let mut scratch: Vec<f32> = Vec::new();
    let mut pending: Vec<f32> = Vec::new();
    let max_pending = (format.sample_rate as usize * 2 * 2).max(8192);
    let mut last_peak_emit = Instant::now() - METER_INTERVAL;
    let mut feedback_frames: u64 = 0;

    while !stop.load(Ordering::Relaxed) {
        let wait = unsafe { WaitForSingleObject(event.0, 20) };
        if wait == WAIT_OBJECT_0 {
            loop {
                let packet = unsafe { capture.GetNextPacketSize()? };
                if packet == 0 {
                    break;
                }

                let mut data: *mut u8 = std::ptr::null_mut();
                let mut frames: u32 = 0;
                let mut flags: u32 = 0;
                unsafe {
                    capture.GetBuffer(&mut data, &mut frames, &mut flags, None, None)?;
                }

                let silent = flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0;
                if silent || data.is_null() || frames == 0 {
                    scratch.clear();
                    scratch.resize(frames as usize * 2, 0.0);
                } else {
                    let bytes = frames as usize * format.block_align as usize;
                    let slice = unsafe { std::slice::from_raw_parts(data, bytes) };
                    frames_to_stereo_f32(format, slice, &mut scratch);
                }
                unsafe { capture.ReleaseBuffer(frames)? };

                let input_peak = scratch.iter().fold(0.0f32, |acc, value| {
                    let value = if value.is_finite() { value.abs() } else { 0.0 };
                    acc.max(value)
                });

                let guard_on = manager.guard_enabled();
                let gain = manager.volume_value() / 100.0;
                let muted = manager.muted();
                let effective_gain = if output_is_speakers && guard_on {
                    gain * 0.6
                } else {
                    gain
                };
                let threshold = if output_is_speakers && guard_on {
                    0.86
                } else {
                    0.98
                };

                process(
                    &mut scratch,
                    effective_gain,
                    muted,
                    alpha,
                    &mut hpf,
                    &mut previous,
                    threshold,
                );

                // Protection anti-larsen : une saturation continue en sortie
                // haut-parleurs indique une boucle probable.
                if output_is_speakers
                    && guard_on
                    && !muted
                    && effective_gain > 0.2
                    && input_peak > 0.995
                {
                    feedback_frames += frames as u64;
                    if feedback_frames as f32 / sample_rate > FEEDBACK_WINDOW_SECONDS {
                        return Err(AppError::audio(
                            "Larsen probable detecte : l'ecoute a ete arretee automatiquement. Utilisez un casque ou baissez le volume d'ecoute.",
                        ));
                    }
                } else {
                    feedback_frames = 0;
                }

                pending.extend_from_slice(&scratch);
                if pending.len() > max_pending {
                    let excess = pending.len() - max_pending;
                    pending.drain(..excess);
                }

                if last_peak_emit.elapsed() >= METER_INTERVAL {
                    manager.update_peak(app, input_peak);
                    last_peak_emit = Instant::now();
                }
            }
        }

        // Alimente le tampon de rendu avec les donnees disponibles. L'espace
        // restant est comble par du silence : cela evite les coupures lorsque
        // l'horloge de capture prend un peu de retard sur la sortie.
        let padding = unsafe { render_audio.GetCurrentPadding()? };
        let available = render_frames.saturating_sub(padding) as usize;
        if available > 0 {
            let buffer = unsafe { render.GetBuffer(available as u32)? };
            if buffer.is_null() {
                return Err(AppError::audio(
                    "Le tampon de sortie audio est indisponible.",
                ));
            }

            let output =
                unsafe { std::slice::from_raw_parts_mut(buffer as *mut f32, available * 2) };
            let frames_ready = (pending.len() / 2).min(available);
            if frames_ready > 0 {
                output[..frames_ready * 2]
                    .copy_from_slice(&pending[..frames_ready * 2]);
                pending.drain(..frames_ready * 2);
            }
            output[frames_ready * 2..].fill(0.0);

            unsafe { render.ReleaseBuffer(available as u32, 0)? };
        }
    }

    manager.update_peak(app, 0.0);
    Ok(())
}
