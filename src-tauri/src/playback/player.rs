use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rodio::{OutputStream, Sink, Decoder, Source};
use std::fs::File;
use std::io::BufReader;
use serde::Serialize;
use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
pub struct PlaybackStatus {
    pub playing: bool,
    pub paused: bool,
    pub stopped: bool,
    pub position: f64,
    pub duration: f64,
    pub file_path: String,
    pub volume: f32,
}

enum Command {
    Play(String, mpsc::Sender<()>),
    Toggle(String, mpsc::Sender<()>),
    Pause(mpsc::Sender<()>),
    Resume(mpsc::Sender<()>),
    Stop(mpsc::Sender<()>),
    SetVolume(f32),
    SetDuration(f64),
}

struct SharedState {
    playing: bool,
    paused: bool,
    stopped: bool,
    position: f64,
    duration: f64,
    file_path: String,
    volume: f32,
}

pub struct AudioPlayer {
    cmd_tx: mpsc::Sender<Command>,
    state: Arc<Mutex<SharedState>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();
        let state = Arc::new(Mutex::new(SharedState {
            playing: false,
            paused: false,
            stopped: true,
            position: 0.0,
            duration: 0.0,
            file_path: String::new(),
            volume: 0.8,
        }));

        let thread_state = state.clone();
        thread::spawn(move || {
            audio_thread(cmd_rx, thread_state);
        });

        Self { cmd_tx, state }
    }

    pub fn play(&self, path: &str) -> Result<PlaybackStatus, AppError> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.cmd_tx.send(Command::Play(path.to_string(), ack_tx))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        ack_rx.recv().ok();
        Ok(self.status())
    }

    pub fn toggle(&self, path: &str) -> Result<PlaybackStatus, AppError> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.cmd_tx.send(Command::Toggle(path.to_string(), ack_tx))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        ack_rx.recv().ok();
        Ok(self.status())
    }

    pub fn pause(&self) -> Result<PlaybackStatus, AppError> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.cmd_tx.send(Command::Pause(ack_tx))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        ack_rx.recv().ok();
        Ok(self.status())
    }

    pub fn resume(&self) -> Result<PlaybackStatus, AppError> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.cmd_tx.send(Command::Resume(ack_tx))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        ack_rx.recv().ok();
        Ok(self.status())
    }

    pub fn stop(&self) -> Result<PlaybackStatus, AppError> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.cmd_tx.send(Command::Stop(ack_tx))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        ack_rx.recv().ok();
        let mut s = self.state.lock().unwrap();
        s.playing = false;
        s.paused = false;
        s.stopped = true;
        s.position = 0.0;
        s.duration = 0.0;
        s.file_path.clear();
        Ok(shared_to_status(&s))
    }

    pub fn status(&self) -> PlaybackStatus {
        let s = self.state.lock().unwrap();
        shared_to_status(&s)
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), AppError> {
        self.cmd_tx.send(Command::SetVolume(volume))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        Ok(())
    }

    pub fn set_duration(&self, duration: f64) -> Result<(), AppError> {
        self.cmd_tx.send(Command::SetDuration(duration))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        Ok(())
    }
}

/// Fallback duration estimate when the decoder can't determine it (e.g. MP3 via minimp3).
/// Uses file size and a conservative bitrate so progress never jumps backwards
/// when the real duration arrives from waveform decoding.
fn estimate_duration(path: &str) -> f64 {
    if let Ok(meta) = std::fs::metadata(path) {
        let size = meta.len();
        if size == 0 {
            return 0.0;
        }
        // Conservative 128 kbps (16 KB/s). Real MP3s are typically 192–320 kbps,
        // so this gives a longer estimate → progress only moves forward when
        // replaced with the real (shorter) duration from waveform decode.
        size as f64 / 16_000.0
    } else {
        0.0
    }
}

fn shared_to_status(s: &SharedState) -> PlaybackStatus {
    PlaybackStatus {
        playing: s.playing,
        paused: s.paused,
        stopped: s.stopped,
        position: s.position,
        duration: s.duration,
        file_path: s.file_path.clone(),
        volume: s.volume,
    }
}

fn flush_state(
    state: &Arc<Mutex<SharedState>>,
    sink: &Option<Sink>,
    started_at: &Option<Instant>,
    total_paused: f64,
    pause_start: &Option<Instant>,
    file_path: &str,
    duration: f64,
    volume: f32,
) {
    let mut s = state.lock().unwrap();
    let sink_finished = sink.as_ref().map(|s| s.empty()).unwrap_or(false);
    let any_sink = sink.is_some();

    s.playing = any_sink && !sink_finished && pause_start.is_none();
    s.paused = any_sink && !sink_finished && pause_start.is_some();
    s.stopped = !any_sink || sink_finished;

    s.position = if let Some(ref sk) = sink {
        let sink_pos = sk.get_pos().as_secs_f64();
        // Use Rodio's decoded position when available — it accounts for the
        // sink's internal buffer, unlike started_at.elapsed() which counts
        // wall-clock time from when the source was appended (before audio starts).
        if sink_pos > 0.0 {
            sink_pos.min(if duration > 0.0 { duration } else { f64::MAX })
        } else if let Some(start) = started_at {
            let elapsed = start.elapsed().as_secs_f64();
            let mut pos = elapsed - total_paused;
            if let Some(ps) = pause_start {
                pos -= ps.elapsed().as_secs_f64();
            }
            pos.max(0.0).min(if duration > 0.0 { duration } else { f64::MAX })
        } else {
            0.0
        }
    } else {
        0.0
    };
    s.duration = duration;
    s.file_path = file_path.to_string();
    s.volume = volume;
}

fn audio_thread(cmd_rx: mpsc::Receiver<Command>, state: Arc<Mutex<SharedState>>) {
    let (_stream, handle) = OutputStream::try_default()
        .expect("Failed to initialize audio output");

    let mut sink: Option<Sink> = None;
    let mut started_at: Option<Instant> = None;
    let mut total_paused: f64 = 0.0;
    let mut pause_start: Option<Instant> = None;
    let mut file_path = String::new();
    let mut duration = 0.0;
    let mut volume = 0.8;

    loop {
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Play(path, ack) => {
                    if let Some(old) = sink.take() {
                        old.stop();
                    }
                    if let Ok(file) = File::open(&path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            duration = source.total_duration()
                                .map(|d| d.as_secs_f64())
                                .unwrap_or_else(|| estimate_duration(&path));
                            if let Ok(s) = Sink::try_new(&handle) {
                                s.append(source);
                                s.set_volume(volume);
                                sink = Some(s);
                                file_path = path;
                                started_at = Some(Instant::now());
                                total_paused = 0.0;
                                pause_start = None;
                            }
                        }
                    }
                    flush_state(&state, &sink, &started_at, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::Toggle(path, ack) => {
                    if file_path == path && !file_path.is_empty() && pause_start.is_some() {
                        if let Some(ref s) = sink {
                            s.play();
                        }
                        if let Some(ps) = pause_start {
                            total_paused += ps.elapsed().as_secs_f64();
                        }
                        pause_start = None;
                    } else if file_path == path && !file_path.is_empty() {
                        if let Some(ref s) = sink {
                            s.pause();
                        }
                        pause_start = Some(Instant::now());
                    } else {
                        if let Some(old) = sink.take() {
                            old.stop();
                        }
                        if let Ok(file) = File::open(&path) {
                            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                                duration = source.total_duration()
                                    .map(|d| d.as_secs_f64())
                                    .unwrap_or_else(|| estimate_duration(&path));
                                if let Ok(s) = Sink::try_new(&handle) {
                                    s.append(source);
                                    s.set_volume(volume);
                                    sink = Some(s);
                                    file_path = path;
                                    started_at = Some(Instant::now());
                                    total_paused = 0.0;
                                    pause_start = None;
                                }
                            }
                        }
                    }
                    flush_state(&state, &sink, &started_at, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::Pause(ack) => {
                    if let Some(ref s) = sink {
                        s.pause();
                    }
                    pause_start = Some(Instant::now());
                    flush_state(&state, &sink, &started_at, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::Resume(ack) => {
                    if let Some(ref s) = sink {
                        s.play();
                    }
                    if let Some(ps) = pause_start {
                        total_paused += ps.elapsed().as_secs_f64();
                    }
                    pause_start = None;
                    flush_state(&state, &sink, &started_at, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::Stop(ack) => {
                    if let Some(s) = sink.take() {
                        s.stop();
                    }
                    file_path.clear();
                    duration = 0.0;
                    started_at = None;
                    total_paused = 0.0;
                    pause_start = None;
                    flush_state(&state, &sink, &started_at, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::SetVolume(v) => {
                    volume = v;
                    if let Some(ref s) = sink {
                        s.set_volume(v);
                    }
                }
                Command::SetDuration(d) => {
                    duration = d;
                }
            }
        }

        flush_state(&state, &sink, &started_at, total_paused, &pause_start, &file_path, duration, volume);
        thread::sleep(Duration::from_millis(50));
    }
}
