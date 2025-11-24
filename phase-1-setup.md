# Phase 1: Project Setup & Core Structure

## Overview
Set up the Rust project structure, dependencies, and basic scaffolding.

## Tasks

### 1.1 Create Cargo.toml

```toml
[package]
name = "piper-tts-server"
version = "0.1.0"
edition = "2021"
description = "A lightweight TTS server using Piper with expression DSL"

[dependencies]
# Web framework
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors"] }

# ONNX runtime for Piper
ort = "2"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Audio processing
hound = "3"  # WAV encoding

# Utilities
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
regex = "1"
lazy_static = "1"

[profile.release]
lto = true
codegen-units = 1
```

### 1.2 Create directory structure

```
piper-tts-server/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   └── handlers.rs
│   ├── tts/
│   │   ├── mod.rs
│   │   ├── piper.rs
│   │   └── voice.rs
│   ├── dsl/
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── transforms.rs
│   └── error.rs
├── static/
│   ├── index.html
│   ├── app.js
│   └── style.css
├── voices/
│   └── .gitkeep
├── docs/
│   └── README.md
└── tests/
    ├── dsl_tests.rs
    └── api_tests.rs
```

### 1.3 Create src/main.rs

Basic server entry point:
- Load configuration from environment
- Initialise tracing/logging
- Set up axum router
- Start server

### 1.4 Create src/lib.rs

Re-export modules:
```rust
pub mod api;
pub mod dsl;
pub mod tts;
pub mod error;
```

### 1.5 Create src/error.rs

Define error types:
```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Voice not found: {0}")]
    VoiceNotFound(String),
    
    #[error("TTS generation failed: {0}")]
    TtsError(String),
    
    #[error("Invalid DSL syntax: {0}")]
    DslError(String),
    
    #[error("ONNX runtime error: {0}")]
    OnnxError(#[from] ort::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

Implement `IntoResponse` for axum integration.

## Acceptance Criteria

- [ ] `cargo build` succeeds
- [ ] `cargo run` starts server on port 3000
- [ ] Logs show startup message
- [ ] GET `/api/health` returns `{"status": "ok"}`
- [ ] Static files served from `/static`

## Notes

- Don't implement TTS yet — just stub it
- Focus on clean module boundaries
- Use `tracing` not `println!`
