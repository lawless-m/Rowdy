use super::parser::Token;

#[derive(Default)]
struct TransformState {
    slow: bool,
    fast: bool,
    emphasis: bool,
    spell: bool,
    whisper: bool,
}

pub fn transform(tokens: Vec<Token>) -> String {
    let mut output = String::new();
    let mut state = TransformState::default();

    for token in tokens {
        match token {
            Token::Text(s) => {
                output.push_str(&apply_state(&s, &state));
            }
            Token::Pause(None) => {
                output.push_str("...");
            }
            Token::Pause(Some(ms)) => {
                // Scale: 200ms = 3 dots, 400ms = 6 dots, etc.
                let dots = ((ms / 200) as usize).max(3);
                output.push_str(&".".repeat(dots));
            }
            Token::SlowStart => state.slow = true,
            Token::SlowEnd => state.slow = false,
            Token::FastStart => state.fast = true,
            Token::FastEnd => state.fast = false,
            Token::EmphasisStart => state.emphasis = true,
            Token::EmphasisEnd => state.emphasis = false,
            Token::SpellStart => state.spell = true,
            Token::SpellEnd => state.spell = false,
            Token::WhisperStart => state.whisper = true,
            Token::WhisperEnd => state.whisper = false,
        }
    }

    output
}

fn apply_state(text: &str, state: &TransformState) -> String {
    let mut result = text.to_string();

    // Apply spell first (changes the structure of the text)
    if state.spell {
        result = result
            .chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| format!("{}.", c.to_uppercase()))
            .collect::<Vec<_>>()
            .join(" ");
        return result; // Spell mode doesn't combine with others
    }

    // Apply emphasis (uppercase)
    if state.emphasis {
        result = result.to_uppercase();
    }

    // Apply whisper (lowercase with parentheses)
    if state.whisper {
        result = format!("({})", result.to_lowercase());
        return result; // Whisper doesn't combine with slow/fast
    }

    // Apply slow (insert ellipses between words)
    if state.slow {
        result = result
            .split_whitespace()
            .collect::<Vec<_>>()
            .join("... ");
        if !result.is_empty() {
            result.push_str("...");
        }
    }

    // Apply fast (remove excess punctuation)
    if state.fast {
        result = result.replace("...", "").replace(',', "");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::parser::parse;

    fn process(input: &str) -> String {
        let tokens = parse(input);
        transform(tokens)
    }

    #[test]
    fn transforms_emphasis_to_caps() {
        assert_eq!(process("[emphasis]hello[/emphasis]"), "HELLO");
    }

    #[test]
    fn transforms_spell_to_letters() {
        assert_eq!(process("[spell]ABC[/spell]"), "A. B. C.");
    }

    #[test]
    fn transforms_whisper() {
        assert_eq!(process("[whisper]Secret[/whisper]"), "(secret)");
    }

    #[test]
    fn transforms_slow() {
        let result = process("[slow]one two three[/slow]");
        assert!(result.contains("one..."));
        assert!(result.contains("two..."));
    }

    #[test]
    fn transforms_fast_removes_pauses() {
        let result = process("[fast]hello, world...[/fast]");
        assert!(!result.contains("..."));
        assert!(!result.contains(','));
    }

    #[test]
    fn transforms_pause() {
        assert_eq!(process("Hello [pause] world"), "Hello ... world");
    }

    #[test]
    fn transforms_timed_pause() {
        let result = process("Wait [pause:600] here");
        // 600ms / 200 = 3 dots minimum
        assert!(result.contains("..."));
    }

    #[test]
    fn handles_nested_emphasis_and_slow() {
        let result = process("[slow][emphasis]wow[/emphasis][/slow]");
        assert!(result.contains("WOW"));
        assert!(result.contains("..."));
    }

    #[test]
    fn preserves_plain_text() {
        assert_eq!(process("Hello world"), "Hello world");
    }

    #[test]
    fn handles_empty_input() {
        assert_eq!(process(""), "");
    }

    #[test]
    fn handles_multiple_pauses() {
        let result = process("[pause][pause][pause]");
        assert_eq!(result, ".........");
    }
}
