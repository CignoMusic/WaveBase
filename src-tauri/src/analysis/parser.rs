use crate::db::models::ParsedMetadata;

pub fn parse_filename(filename: &str) -> ParsedMetadata {
    ParsedMetadata {
        track_name: None,
        bpm: None,
        key: None,
        artist: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bpm() {
        let result = parse_filename("loop 128bpm test.wav");
        assert_eq!(result.bpm, None);
    }
}
