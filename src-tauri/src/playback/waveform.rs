use std::fs::File;

use serde::Serialize;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
pub struct WaveformData {
    pub peaks: Vec<f64>,
    pub duration: f64,
}

pub fn compute_waveform_peaks(path: &str, num_bars: usize) -> Result<WaveformData, AppError> {
    let file = File::open(path).map_err(|e| AppError::Io(e.to_string()))?;
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
        .map_err(|e| AppError::Analysis(format!("Failed to probe audio: {}", e)))?;

    let track = probed
        .format
        .default_track()
        .ok_or_else(|| AppError::Analysis("No audio track found".to_string()))?;

    let codec_params = &track.codec_params;

    let duration = codec_params
        .time_base
        .zip(codec_params.n_frames)
        .map(|(tb, nf)| {
            let t = tb.calc_time(nf);
            t.seconds as f64 + t.frac
        })
        .unwrap_or(0.0);

    let total_frames = codec_params.n_frames.unwrap_or(0);

    let mut decoder = symphonia::default::get_codecs()
        .make(codec_params, &DecoderOptions { verify: false, ..Default::default() })
        .map_err(|e| AppError::Analysis(format!("Failed to create decoder: {}", e)))?;

    let frames_per_window = if total_frames > 0 {
        std::cmp::max(1, (total_frames as f64 / num_bars as f64).ceil() as usize)
    } else {
        44100
    };

    let mut peaks: Vec<f64> = Vec::with_capacity(num_bars);
    let mut window_peak: f64 = 0.0;
    let mut frames_in_window: usize = 0;
    let mut any_audio = false;

    loop {
        match probed.format.next_packet() {
            Ok(packet) => {
                if let Ok(audio_buf) = decoder.decode(&packet) {
                    any_audio = true;
                    let num_frames = audio_buf.frames();
                    let spec = audio_buf.spec();
                    let channels = spec.channels.count() as usize;

                    let mut sample_buf =
                        SampleBuffer::<f32>::new(num_frames as u64, spec.clone());
                    sample_buf.copy_interleaved_ref(audio_buf);
                    let samples = sample_buf.samples();

                    for frame in 0..num_frames {
                        let mut frame_peak = 0.0f64;
                        for ch in 0..channels {
                            if let Some(s) = samples.get(frame * channels + ch) {
                                let abs_val = s.abs() as f64;
                                if abs_val > frame_peak {
                                    frame_peak = abs_val;
                                }
                            }
                        }
                        if frame_peak > window_peak {
                            window_peak = frame_peak;
                        }
                        frames_in_window += 1;

                        if frames_in_window >= frames_per_window {
                            peaks.push(window_peak.min(1.0));
                            window_peak = 0.0;
                            frames_in_window = 0;
                        }
                    }
                }
            }
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_) => break,
        }
    }

    if !any_audio {
        return Ok(WaveformData {
            peaks: vec![0.0; num_bars],
            duration,
        });
    }

    if frames_in_window > 0 {
        peaks.push(window_peak.min(1.0));
    }

    if peaks.len() != num_bars {
        if peaks.is_empty() {
            return Ok(WaveformData {
                peaks: vec![0.0; num_bars],
                duration,
            });
        }
        let original = std::mem::replace(&mut peaks, Vec::with_capacity(num_bars));
        for i in 0..num_bars {
            let pos = (i as f64 / num_bars as f64) * original.len() as f64;
            let idx = (pos.floor() as usize).min(original.len() - 1);
            peaks.push(original[idx]);
        }
    }

    Ok(WaveformData { peaks, duration })
}
