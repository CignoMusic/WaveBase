use std::fs::File;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AppError;

pub fn decode_audio_to_mono(path: &str) -> Result<(Vec<f32>, u32), AppError> {
    let file = File::open(path).map_err(|e| AppError::Io(format!("Failed to open {}: {}", path, e)))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    hint.with_extension(ext);

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
        .map_err(|e| AppError::Analysis(format!("Probe failed for {}: {}", path, e)))?;

    let mut format = probed.format;
    let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| AppError::Analysis("No audio track found".to_string()))?;

    let codec_params = track.codec_params.clone();
    let track_id = track.id;
    let sample_rate = codec_params.sample_rate.unwrap_or(44100);

    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|e| AppError::Analysis(format!("Decoder init failed: {}", e)))?;

    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(pkt) => pkt,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(_) => break,
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let spec = *decoded.spec();
        let num_frames = decoded.frames();
        let num_channels = spec.channels.count();

        let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64 * num_channels as u64, spec);
        sample_buf.copy_interleaved_ref(decoded);

        // Mix down to mono
        let samples = sample_buf.samples();
        if num_channels == 1 {
            all_samples.extend_from_slice(samples);
        } else {
            for frame in 0..num_frames {
                let mut sum = 0.0f32;
                for ch in 0..num_channels {
                    sum += samples[frame * num_channels + ch];
                }
                all_samples.push(sum / num_channels as f32);
            }
        }
    }

    if all_samples.is_empty() {
        return Err(AppError::Analysis("No audio data decoded".to_string()));
    }

    Ok((all_samples, sample_rate))
}
