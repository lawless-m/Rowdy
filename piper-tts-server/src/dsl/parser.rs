use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Pause(Option<u32>),
    SlowStart,
    SlowEnd,
    FastStart,
    FastEnd,
    EmphasisStart,
    EmphasisEnd,
    SpellStart,
    SpellEnd,
    WhisperStart,
    WhisperEnd,
}

lazy_static! {
    static ref TAG_REGEX: Regex = Regex::new(
        r"(?x)
        \[pause:(\d+)\]|           # Timed pause [pause:500]
        \[pause\]|                  # Simple pause [pause]
        \[/?(slow|fast|emphasis|spell|whisper)\]  # Opening/closing tags
        "
    )
    .unwrap();
}

pub fn parse(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut last_end = 0;

    for cap in TAG_REGEX.captures_iter(input) {
        let m = cap.get(0).unwrap();

        // Add any text before this tag
        if m.start() > last_end {
            let text = &input[last_end..m.start()];
            if !text.is_empty() {
                tokens.push(Token::Text(text.to_string()));
            }
        }

        // Parse the tag
        let tag_str = m.as_str();
        let token = parse_tag(tag_str, &cap);
        tokens.push(token);

        last_end = m.end();
    }

    // Add any remaining text after the last tag
    if last_end < input.len() {
        let text = &input[last_end..];
        if !text.is_empty() {
            tokens.push(Token::Text(text.to_string()));
        }
    }

    // If no tags were found, return the whole input as text
    if tokens.is_empty() && !input.is_empty() {
        tokens.push(Token::Text(input.to_string()));
    }

    tokens
}

fn parse_tag(tag_str: &str, cap: &regex::Captures) -> Token {
    // Check for timed pause [pause:N]
    if let Some(ms_match) = cap.get(1) {
        let ms: u32 = ms_match.as_str().parse().unwrap_or(200);
        return Token::Pause(Some(ms));
    }

    // Check for simple pause
    if tag_str == "[pause]" {
        return Token::Pause(None);
    }

    // Check for paired tags
    match tag_str {
        "[slow]" => Token::SlowStart,
        "[/slow]" => Token::SlowEnd,
        "[fast]" => Token::FastStart,
        "[/fast]" => Token::FastEnd,
        "[emphasis]" => Token::EmphasisStart,
        "[/emphasis]" => Token::EmphasisEnd,
        "[spell]" => Token::SpellStart,
        "[/spell]" => Token::SpellEnd,
        "[whisper]" => Token::WhisperStart,
        "[/whisper]" => Token::WhisperEnd,
        _ => Token::Text(tag_str.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_text() {
        let tokens = parse("Hello world");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Text(s) if s == "Hello world"));
    }

    #[test]
    fn parses_simple_pause() {
        let tokens = parse("Hello [pause] world");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(&tokens[0], Token::Text(s) if s == "Hello "));
        assert!(matches!(tokens[1], Token::Pause(None)));
        assert!(matches!(&tokens[2], Token::Text(s) if s == " world"));
    }

    #[test]
    fn parses_timed_pause() {
        let tokens = parse("[pause:500]");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Pause(Some(500))));
    }

    #[test]
    fn parses_paired_tags() {
        let tokens = parse("[slow]text[/slow]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::SlowStart));
        assert!(matches!(&tokens[1], Token::Text(s) if s == "text"));
        assert!(matches!(tokens[2], Token::SlowEnd));
    }

    #[test]
    fn parses_emphasis() {
        let tokens = parse("[emphasis]important[/emphasis]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::EmphasisStart));
        assert!(matches!(tokens[2], Token::EmphasisEnd));
    }

    #[test]
    fn parses_spell() {
        let tokens = parse("[spell]ABC[/spell]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::SpellStart));
        assert!(matches!(tokens[2], Token::SpellEnd));
    }

    #[test]
    fn parses_whisper() {
        let tokens = parse("[whisper]secret[/whisper]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::WhisperStart));
        assert!(matches!(tokens[2], Token::WhisperEnd));
    }

    #[test]
    fn parses_fast() {
        let tokens = parse("[fast]quick[/fast]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::FastStart));
        assert!(matches!(tokens[2], Token::FastEnd));
    }
}
