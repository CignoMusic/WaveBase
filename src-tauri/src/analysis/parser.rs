use crate::db::models::ParsedMetadata;

pub fn parse_filename(filename: &str) -> ParsedMetadata {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename)
        .replace('_', " ")
        .replace('-', " ");

    let lower = stem.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    let mut bpm: Option<i32> = None;
    let mut key: Option<String> = None;
    let mut artist: Option<String> = None;
    let mut track_type: Option<String> = None;
    let mut used_indices: Vec<usize> = Vec::new();

    // Track type detection (must come before BPM to avoid ambiguity with "loop")
    for (i, w) in words.iter().enumerate() {
        let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
        match clean {
            "loop" | "loops" => { track_type = Some("Loop".to_string()); used_indices.push(i); }
            "beat" | "beats" => { track_type = Some("Beat".to_string()); used_indices.push(i); }
            "stem" | "stems" => { track_type = Some("Stem".to_string()); used_indices.push(i); }
            "oneshot" | "one_shot" | "one-shot" | "shot" => { track_type = Some("OneShot".to_string()); used_indices.push(i); }
            _ => {}
        }
    }

    // Artist detection: @username
    for (i, w) in words.iter().enumerate() {
        if w.starts_with('@') && w.len() > 1 {
            let name = w[1..].trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_');
            if !name.is_empty() {
                artist = Some(name.to_string());
                used_indices.push(i);
            }
        }
    }

    // BPM detection: "140bpm", "bpm140", "140 bpm", "bpm 140"
    for (i, w) in words.iter().enumerate() {
        let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
        if clean == "bpm" {
            // Check previous word (e.g., "120 bpm")
            if i > 0 {
                if let Ok(n) = words[i - 1].parse::<i32>() {
                    if (60..=300).contains(&n) {
                        bpm = Some(n);
                        used_indices.push(i - 1);
                        used_indices.push(i);
                        continue;
                    }
                }
            }
            // Check next word (e.g., "bpm 120")
            if i + 1 < words.len() {
                if let Ok(n) = words[i + 1].parse::<i32>() {
                    if (60..=300).contains(&n) {
                        bpm = Some(n);
                        used_indices.push(i);
                        used_indices.push(i + 1);
                        continue;
                    }
                }
            }
        } else if let Some(num) = clean.strip_suffix("bpm") {
            if !num.is_empty() {
                if let Ok(n) = num.parse::<i32>() {
                    if (60..=300).contains(&n) {
                        bpm = Some(n);
                        used_indices.push(i);
                    }
                }
            }
        } else if let Some(num) = clean.strip_prefix("bpm") {
            if !num.is_empty() {
                if let Ok(n) = num.parse::<i32>() {
                    if (60..=300).contains(&n) {
                        bpm = Some(n);
                        used_indices.push(i);
                    }
                }
            }
        }
    }

    // Key detection
    for (i, w) in words.iter().enumerate() {
        let clean = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '#');
        if let Some(normalized) = normalize_key(clean) {
            key = Some(normalized);
            used_indices.push(i);
        }
    }

    // Track name: remaining words not used by other patterns
    let track_name = if used_indices.is_empty() {
        Some(stem.trim().to_string())
    } else {
        let remaining: Vec<String> = words.iter().enumerate()
            .filter(|(i, _)| !used_indices.contains(i))
            .map(|(_, w)| (*w).to_string())
            .collect();
        if remaining.is_empty() {
            None
        } else {
            Some(remaining.join(" "))
        }
    };

    ParsedMetadata {
        track_name: track_name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        bpm,
        key,
        artist,
        track_type,
    }
}

fn normalize_key(s: &str) -> Option<String> {
    let s = s.trim();
    let upper = s.to_uppercase();
    let upper_chars: Vec<char> = upper.chars().collect();

    // Parse key pattern: root note + optional accidental + optional mode
    let (root_str, mode_str) = if upper_chars.len() >= 2 {
        let first = upper_chars[0];
        let second = upper_chars[1];
        if first >= 'A' && first <= 'G' {
            if second == '#' || second == 'b' {
                (format!("{}{}", first, second), &s[2..])
            } else {
                (first.to_string(), &s[1..])
            }
        } else {
            return None;
        }
    } else if upper_chars.len() == 1 {
        let c = upper_chars[0];
        if c >= 'A' && c <= 'G' {
            (c.to_string(), "")
        } else {
            return None;
        }
    } else {
        return None;
    };

    let mode_str = mode_str.trim().to_lowercase();

    // Determine mode
    let mode = match mode_str.as_str() {
        "" | "maj" | "major" | "dur" => "Major",
        "m" | "min" | "minor" | "moll" => "Minor",
        _ => return None,
    };

    Some(format!("{} {}", root_str, mode))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bpm_suffix() {
        let r = parse_filename("loop 128bpm test.wav");
        assert_eq!(r.bpm, Some(128));
        assert_eq!(r.track_type, Some("Loop".to_string()));
    }

    #[test]
    fn test_parse_bpm_prefix() {
        let r = parse_filename("bpm140 kick.wav");
        assert_eq!(r.bpm, Some(140));
    }

    #[test]
    fn test_parse_bpm_with_space() {
        let r = parse_filename("my track 120 BPM.mp3");
        assert_eq!(r.bpm, Some(120));
    }

    #[test]
    fn test_parse_key_minor() {
        let r = parse_filename("track Dm.wav");
        assert_eq!(r.key, Some("D Minor".to_string()));
    }

    #[test]
    fn test_parse_key_major() {
        let r = parse_filename("C# major loop.wav");
        assert_eq!(r.key, Some("C# Major".to_string()));
        assert_eq!(r.track_type, Some("Loop".to_string()));
    }

    #[test]
    fn test_parse_key_shorthand() {
        let r = parse_filename("Am bass.wav");
        assert_eq!(r.key, Some("A Minor".to_string()));
    }

    #[test]
    fn test_parse_artist() {
        let r = parse_filename("track @producer.wav");
        assert_eq!(r.artist, Some("producer".to_string()));
    }

    #[test]
    fn test_parse_full() {
        let r = parse_filename("Kick_Loop_140bpm_Dm_@user.wav");
        assert_eq!(r.bpm, Some(140));
        assert_eq!(r.key, Some("D Minor".to_string()));
        assert_eq!(r.track_type, Some("Loop".to_string()));
        assert_eq!(r.artist, Some("user".to_string()));
    }

    #[test]
    fn test_parse_no_match() {
        let r = parse_filename("unknown file.wav");
        assert_eq!(r.bpm, None);
        assert_eq!(r.key, None);
        assert_eq!(r.artist, None);
        assert_eq!(r.track_type, None);
    }

    #[test]
    fn test_parse_bpm_out_of_range() {
        let r = parse_filename("file 999bpm.wav");
        assert_eq!(r.bpm, None);
    }
}
