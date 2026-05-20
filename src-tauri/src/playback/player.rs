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
    Play(String),
    Toggle(String),
    Pause,
    Resume,
    Stop,
    SetVolume(f32),
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
        self.cmd_tx.send(Command::Play(path.to_string()))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        thread::sleep(Duration::from_millis(15));
        Ok(self.status())
    }

    pub fn toggle(&self, path: &str) -> Result<PlaybackStatus, AppError> {
        self.cmd_tx.send(Command::Toggle(path.to_string()))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        thread::sleep(Duration::from_millis(15));
        Ok(self.status())
    }

    pub fn pause(&self) -> Result<PlaybackStatus, AppError> {
        self.cmd_tx.send(Command::Pause)
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        thread::sleep(Duration::from_millis(10));
        Ok(self.status())
    }

    pub fn resume(&self) -> Result<PlaybackStatus, AppError> {
        self.cmd_tx.send(Command::Resume)
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        thread::sleep(Duration::from_millis(10));
        Ok(self.status())
    }

    pub fn stop(&self) -> Result<PlaybackStatus, AppError> {
        self.cmd_tx.send(Command::Stop)
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        thread::sleep(Duration::from_millis(10));
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
                Command::Play(path) => {
                    if let Some(old) = sink.take() {
                        old.stop();
                    }
                    if let Ok(file) = File::open(&path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            duration = source.total_duration()
                                .map(|d| d.as_secs_f64())
                                .unwrap_or(0.0);
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
                Command::Toggle(path) => {
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
                                    .unwrap_or(0.0);
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
                }
                Command::Pause => {
                    if let Some(ref s) = sink {
                        s.pause();
                    }
                    pause_start = Some(Instant::now());
                }
                Command::Resume => {
                    if let Some(ref s) = sink {
                        s.play();
                    }
                    if let Some(ps) = pause_start {
                        total_paused += ps.elapsed().as_secs_f64();
                    }
                    pause_start = None;
                }
                Command::Stop => {
                    if let Some(s) = sink.take() {
                        s.stop();
                    }
                    file_path.clear();
                    duration = 0.0;
                    started_at = None;
                    total_paused = 0.0;
                    pause_start = None;
                }
                Command::SetVolume(v) => {
                    volume = v;
                    if let Some(ref s) = sink {
                        s.set_volume(v);
                    }
                }
            }
        }

        let mut s = state.lock().unwrap();
        let sink_finished = sink.as_ref().map(|s| s.empty()).unwrap_or(false);
        let any_sink = sink.is_some();

        s.playing = any_sink && !sink_finished && pause_start.is_none();
        s.paused = any_sink && !sink_finished && pause_start.is_some();
        s.stopped = !any_sink || sink_finished;

        s.position = if let Some(start) = started_at {
            let elapsed = start.elapsed().as_secs_f64();
            let mut pos = elapsed - total_paused;
            if let Some(ps) = pause_start {
                pos -= ps.elapsed().as_secs_f64();
            }
            pos.max(0.0).min(duration)
        } else {
            0.0
        };
        s.duration = duration;
        s.file_path = file_path.clone();
        s.volume = volume;
        drop(s);

        thread::sleep(Duration::from_millis(50));
    }
}
