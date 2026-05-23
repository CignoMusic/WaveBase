use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rodio::{OutputStream, Sink, Decoder, Source};
use rodio::buffer::SamplesBuffer;
use std::fs::File;
use std::io::BufReader;
use serde::Serialize;
use lofty::file::AudioFile;
use lofty::read_from_path;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
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
    Seek(f64, mpsc::Sender<()>),
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

struct PcmBuffer {
    path: String,
    data: Vec<f32>,
    sample_rate: u32,
    channels: u16,
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

    pub fn seek(&self, position: f64) -> Result<PlaybackStatus, AppError> {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.cmd_tx.send(Command::Seek(position, ack_tx))
            .map_err(|_| AppError::Playback("Audio thread disconnected".into()))?;
        ack_rx.recv().ok();
        Ok(self.status())
    }
}

/// Reads the actual duration from audio file metadata headers (instant, no decode).
fn probe_duration(path: &str) -> f64 {
    match read_from_path(path) {
        Ok(file) => {
            let dur = file.properties().duration();
            dur.as_secs_f64()
        }
        Err(_) => 0.0,
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
    seek_offset: f64,
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

    s.position = if let Some(start) = started_at {
        let elapsed = start.elapsed().as_secs_f64();
        let mut pos = seek_offset + elapsed - total_paused;
        if let Some(ps) = pause_start {
            pos -= ps.elapsed().as_secs_f64();
        }
        pos.max(0.0).min(if duration > 0.0 { duration } else { f64::MAX })
    } else {
        0.0
    };
    s.duration = duration;
    s.file_path = file_path.to_string();
    s.volume = volume;
}

/// Decodes an entire audio file into raw PCM samples using Symphonia.
/// Runs in a background thread so playback starts immediately with Rodio.
fn decode_file_to_pcm(path: &str) -> Option<PcmBuffer> {
    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
    {
        hint.with_extension(ext);
    }

    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts = MetadataOptions::default();

    let mut probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .ok()?;

    let track = probed.format.default_track()?;
    let track_id = track.id;
    let codec_params = &track.codec_params;
    let sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);

    let mut decoder = symphonia::default::get_codecs()
        .make(codec_params, &DecoderOptions { verify: false, ..Default::default() })
        .ok()?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        match probed.format.next_packet() {
            Ok(packet) => {
                if packet.track_id() != track_id {
                    continue;
                }
                if let Ok(audio_buf) = decoder.decode(&packet) {
                    let num_frames = audio_buf.frames();
                    let spec = audio_buf.spec();
                    let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec.clone());
                    sample_buf.copy_interleaved_ref(audio_buf);
                    all_samples.extend_from_slice(sample_buf.samples());
                }
            }
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(_) => break,
        }
    }

    if all_samples.is_empty() {
        return None;
    }

    Some(PcmBuffer {
        path: path.to_string(),
        data: all_samples,
        sample_rate,
        channels,
    })
}

fn audio_thread(cmd_rx: mpsc::Receiver<Command>, state: Arc<Mutex<SharedState>>) {
    let (_stream, handle) = OutputStream::try_default()
        .expect("Failed to initialize audio output");

    let mut sink: Option<Sink> = None;
    let mut started_at: Option<Instant> = None;
    let mut seek_offset: f64 = 0.0;
    let mut total_paused: f64 = 0.0;
    let mut pause_start: Option<Instant> = None;
    let mut file_path = String::new();
    let mut duration = 0.0;
    let mut volume = 0.8;
    let mut pcm_buffer: Option<PcmBuffer> = None;
    let (bg_tx, bg_rx) = mpsc::channel::<PcmBuffer>();

    loop {
        // Collect any decoded PCM buffers from background threads
        while let Ok(buf) = bg_rx.try_recv() {
            pcm_buffer = Some(buf);
        }

        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                Command::Play(path, ack) => {
                    if let Some(old) = sink.take() {
                        old.stop();
                    }
                    pcm_buffer = None;
                    if let Ok(file) = File::open(&path) {
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            duration = source.total_duration()
                                .map(|d| d.as_secs_f64())
                                .unwrap_or_else(|| probe_duration(&path));
                            if let Ok(s) = Sink::try_new(&handle) {
                                s.append(source);
                                s.set_volume(volume);
                                sink = Some(s);
                                file_path = path.clone();
                                started_at = Some(Instant::now());
                                seek_offset = 0.0;
                                total_paused = 0.0;
                                pause_start = None;
                            }
                        }
                    }
                    // Background decode entire file for instant seeking
                    if !file_path.is_empty() {
                        let bg_path = file_path.clone();
                        let bg_tx = bg_tx.clone();
                        thread::spawn(move || {
                            if let Some(buf) = decode_file_to_pcm(&bg_path) {
                                let _ = bg_tx.send(buf);
                            }
                        });
                    }
                    flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
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
                        pcm_buffer = None;
                        if let Ok(file) = File::open(&path) {
                            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                                duration = source.total_duration()
                                    .map(|d| d.as_secs_f64())
                                    .unwrap_or_else(|| probe_duration(&path));
                                if let Ok(s) = Sink::try_new(&handle) {
                                    s.append(source);
                                    s.set_volume(volume);
                                    sink = Some(s);
                                    file_path = path.clone();
                                    started_at = Some(Instant::now());
                                    seek_offset = 0.0;
                                    total_paused = 0.0;
                                    pause_start = None;
                                }
                            }
                        }
                        if !file_path.is_empty() {
                            let bg_path = file_path.clone();
                            let bg_tx = bg_tx.clone();
                            thread::spawn(move || {
                                if let Some(buf) = decode_file_to_pcm(&bg_path) {
                                    let _ = bg_tx.send(buf);
                                }
                            });
                        }
                    }
                    flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::Pause(ack) => {
                    if let Some(ref s) = sink {
                        s.pause();
                    }
                    pause_start = Some(Instant::now());
                    flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
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
                    flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::Stop(ack) => {
                    if let Some(s) = sink.take() {
                        s.stop();
                    }
                    file_path.clear();
                    duration = 0.0;
                    started_at = None;
                    seek_offset = 0.0;
                    total_paused = 0.0;
                    pause_start = None;
                    pcm_buffer = None;
                    flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
                    let _ = ack.send(());
                }
                Command::Seek(position, ack) => {
                    let was_paused = pause_start.is_some();
                    if let Some(old) = sink.take() {
                        old.stop();
                    }
                    if !file_path.is_empty() {
                        let seek_pos = position.min(duration);

                        // Fast path: use pre-decoded PCM buffer
                        if let Some(ref buf) = pcm_buffer {
                            if buf.path == file_path && !buf.data.is_empty() {
                                let start_sample = (seek_pos * buf.sample_rate as f64 * buf.channels as f64) as usize;
                                let start_sample = start_sample.min(buf.data.len());
                                let slice = buf.data[start_sample..].to_vec();
                                if !slice.is_empty() {
                                    let samples_buf = SamplesBuffer::new(buf.channels, buf.sample_rate, slice);
                                    if let Ok(s) = Sink::try_new(&handle) {
                                        s.append(samples_buf);
                                        s.set_volume(volume);
                                        if was_paused {
                                            s.pause();
                                            pause_start = Some(Instant::now());
                                        } else {
                                            pause_start = None;
                                        }
                                        sink = Some(s);
                                        seek_offset = seek_pos;
                                        started_at = Some(Instant::now());
                                        total_paused = 0.0;
                                        flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
                                        let _ = ack.send(());
                                        continue;
                                    }
                                }
                            }
                        }

                        // Slow path: Rodio skip_duration (fallback if PCM buffer not ready yet)
                        if let Ok(file) = File::open(&file_path) {
                            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                                let skipped = source.skip_duration(Duration::from_secs_f64(seek_pos));
                                if let Ok(s) = Sink::try_new(&handle) {
                                    s.append(skipped);
                                    s.set_volume(volume);
                                    if was_paused {
                                        s.pause();
                                        pause_start = Some(Instant::now());
                                    } else {
                                        pause_start = None;
                                    }
                                    sink = Some(s);
                                    seek_offset = seek_pos;
                                    started_at = Some(Instant::now());
                                    total_paused = 0.0;
                                }
                            }
                        }
                    }
                    flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
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

        flush_state(&state, &sink, &started_at, seek_offset, total_paused, &pause_start, &file_path, duration, volume);
        thread::sleep(Duration::from_millis(50));
    }
}
