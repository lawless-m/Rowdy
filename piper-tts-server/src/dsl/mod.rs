pub mod parser;
pub mod transforms;

/// Process DSL text into Piper-friendly plain text
pub fn process(input: &str) -> String {
    let tokens = parser::parse(input);
    transforms::transform(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pause() {
        assert_eq!(process("Hello [pause] world"), "Hello ... world");
    }

    #[test]
    fn test_timed_pause() {
        let result = process("Wait [pause:600] here");
        assert!(result.contains("..."));
        assert!(result.contains("Wait"));
        assert!(result.contains("here"));
    }

    #[test]
    fn test_slow() {
        let result = process("[slow]one two[/slow]");
        assert!(result.contains("one"));
        assert!(result.contains("two"));
        assert!(result.contains("..."));
    }

    #[test]
    fn test_emphasis() {
        assert_eq!(
            process("This is [emphasis]important[/emphasis]"),
            "This is IMPORTANT"
        );
    }

    #[test]
    fn test_spell() {
        assert_eq!(process("[spell]BBC[/spell]"), "B. B. C.");
    }

    #[test]
    fn test_whisper() {
        assert_eq!(process("[whisper]Secret[/whisper]"), "(secret)");
    }

    #[test]
    fn test_no_tags() {
        assert_eq!(process("Plain text here."), "Plain text here.");
    }

    #[test]
    fn test_unknown_tag_passthrough() {
        assert_eq!(process("Hello [unknown] world"), "Hello [unknown] world");
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(process(""), "");
    }
}
