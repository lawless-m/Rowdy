# Phase 2: DSL Parser & Transforms

## Overview
Implement the expression DSL that transforms tagged text into Piper-friendly output.

## Tasks

### 2.1 Define DSL tokens

In `src/dsl/mod.rs`:

```rust
pub mod parser;
pub mod transforms;

pub use parser::parse;
pub use transforms::transform;

/// Process DSL text into Piper-friendly plain text
pub fn process(input: &str) -> String {
    let tokens = parser::parse(input);
    transforms::transform(tokens)
}
```

### 2.2 Create parser (src/dsl/parser.rs)

Parse input text into a stream of tokens:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Text(String),
    Pause(Option<u32>),           // [pause] or [pause:500]
    SlowStart,                     // [slow]
    SlowEnd,                       // [/slow]
    FastStart,                     // [fast]
    FastEnd,                       // [/fast]
    EmphasisStart,                 // [emphasis]
    EmphasisEnd,                   // [/emphasis]
    SpellStart,                    // [spell]
    SpellEnd,                      // [/spell]
    WhisperStart,                  // [whisper]
    WhisperEnd,                    // [/whisper]
}

pub fn parse(input: &str) -> Vec<Token> {
    // Use regex to find tags
    // Everything between tags is Text
    // Return ordered Vec<Token>
}
```

Regex patterns:
- `\[pause\]` — simple pause
- `\[pause:(\d+)\]` — timed pause
- `\[(slow|fast|emphasis|spell|whisper)\]` — opening tags
- `\[/(slow|fast|emphasis|spell|whisper)\]` — closing tags

### 2.3 Create transforms (src/dsl/transforms.rs)

Convert tokens to plain text:

```rust
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
                let dots = (ms / 200).max(3) as usize;
                output.push_str(&".".repeat(dots));
                output.push(' ');
            }
            Token::SlowStart => state.slow = true,
            Token::SlowEnd => state.slow = false,
            // ... etc
        }
    }
    
    output
}

#[derive(Default)]
struct TransformState {
    slow: bool,
    fast: bool,
    emphasis: bool,
    spell: bool,
    whisper: bool,
}

fn apply_state(text: &str, state: &TransformState) -> String {
    let mut result = text.to_string();
    
    if state.emphasis {
        result = result.to_uppercase();
    }
    
    if state.slow {
        // Insert ellipses between words
        result = result.split_whitespace()
            .collect::<Vec<_>>()
            .join("... ");
        result.push_str("...");
    }
    
    if state.fast {
        // Remove excess punctuation
        result = result.replace("...", "").replace(",", "");
    }
    
    if state.spell {
        // Add dots between letters
        result = result.chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| format!("{}.", c.to_uppercase()))
            .collect::<Vec<_>>()
            .join(" ");
    }
    
    if state.whisper {
        result = format!("({})", result.to_lowercase());
    }
    
    result
}
```

### 2.4 Handle edge cases

- Nested tags: `[slow][emphasis]text[/emphasis][/slow]` — apply both
- Unclosed tags: treat as text, or close at end
- Empty tags: `[slow][/slow]` — produce nothing
- Escaped brackets: `\[not a tag\]` — output literal brackets
- Unknown tags: `[unknown]` — treat as plain text

### 2.5 Write tests (tests/dsl_tests.rs)

```rust
#[test]
fn test_simple_pause() {
    assert_eq!(
        dsl::process("Hello [pause] world"),
        "Hello... world"
    );
}

#[test]
fn test_timed_pause() {
    assert_eq!(
        dsl::process("Wait [pause:600] here"),
        "Wait...... here"
    );
}

#[test]
fn test_slow() {
    assert_eq!(
        dsl::process("[slow]one two[/slow]"),
        "one...... two..."
    );
}

#[test]
fn test_emphasis() {
    assert_eq!(
        dsl::process("This is [emphasis]important[/emphasis]"),
        "This is IMPORTANT"
    );
}

#[test]
fn test_spell() {
    assert_eq!(
        dsl::process("[spell]BBC[/spell]"),
        "B. B. C."
    );
}

#[test]
fn test_whisper() {
    assert_eq!(
        dsl::process("[whisper]Secret[/whisper]"),
        "(secret)"
    );
}

#[test]
fn test_nested() {
    assert_eq!(
        dsl::process("[slow][emphasis]wow[/emphasis][/slow]"),
        "WOW..."
    );
}

#[test]
fn test_no_tags() {
    assert_eq!(
        dsl::process("Plain text here."),
        "Plain text here."
    );
}

#[test]
fn test_unknown_tag_passthrough() {
    assert_eq!(
        dsl::process("Hello [unknown] world"),
        "Hello [unknown] world"
    );
}
```

## Acceptance Criteria

- [ ] All DSL tags parse correctly
- [ ] Transforms produce expected output
- [ ] Edge cases handled gracefully
- [ ] All tests pass
- [ ] `cargo test` runs clean

## Notes

- Keep transforms simple — Piper's interpretation varies
- Document behaviour for edge cases
- Consider adding `[break]` as alias for `[pause]`
