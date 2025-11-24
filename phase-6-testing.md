# Phase 6: Testing & Deployment

## Overview
Add comprehensive tests and prepare for deployment.

## Tasks

### 6.1 Unit tests

**DSL tests (tests/dsl_tests.rs):**
```rust
use piper_tts_server::dsl;

mod parse_tests {
    use super::*;
    
    #[test]
    fn parses_plain_text() {
        let tokens = dsl::parser::parse("Hello world");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], dsl::Token::Text(_)));
    }
    
    #[test]
    fn parses_simple_pause() {
        let tokens = dsl::parser::parse("Hello [pause] world");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[1], dsl::Token::Pause(None)));
    }
    
    #[test]
    fn parses_timed_pause() {
        let tokens = dsl::parser::parse("[pause:500]");
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], dsl::Token::Pause(Some(500))));
    }
    
    #[test]
    fn parses_paired_tags() {
        let tokens = dsl::parser::parse("[slow]text[/slow]");
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], dsl::Token::SlowStart));
        assert!(matches!(tokens[2], dsl::Token::SlowEnd));
    }
}

mod transform_tests {
    use super::*;
    
    #[test]
    fn transforms_emphasis_to_caps() {
        assert_eq!(
            dsl::process("[emphasis]hello[/emphasis]"),
            "HELLO"
        );
    }
    
    #[test]
    fn transforms_spell_to_letters() {
        assert_eq!(
            dsl::process("[spell]ABC[/spell]"),
            "A. B. C."
        );
    }
    
    #[test]
    fn handles_nested_tags() {
        let result = dsl::process("[slow][emphasis]test[/emphasis][/slow]");
        assert!(result.contains("TEST"));
        assert!(result.contains("..."));
    }
    
    #[test]
    fn preserves_unknown_tags() {
        assert_eq!(
            dsl::process("Hello [unknown] world"),
            "Hello [unknown] world"
        );
    }
    
    #[test]
    fn handles_empty_input() {
        assert_eq!(dsl::process(""), "");
    }
    
    #[test]
    fn handles_only_tags() {
        assert_eq!(dsl::process("[pause][pause][pause]"), ".........");
    }
}
```

**Voice tests (tests/voice_tests.rs):**
```rust
use piper_tts_server::tts::Voice;
use std::path::PathBuf;

#[test]
fn loads_valid_voice() {
    let voice = Voice::load(
        &PathBuf::from("./test_fixtures/voices"),
        "test-voice"
    );
    assert!(voice.is_ok());
}

#[test]
fn errors_on_missing_voice() {
    let voice = Voice::load(
        &PathBuf::from("./test_fixtures/voices"),
        "nonexistent"
    );
    assert!(voice.is_err());
}

#[test]
fn parses_voice_config() {
    let voice = Voice::load(
        &PathBuf::from("./test_fixtures/voices"),
        "test-voice"
    ).unwrap();
    
    assert!(voice.config.audio.sample_rate > 0);
    assert!(!voice.config.phoneme_id_map.is_empty());
}
```

### 6.2 Integration tests

**API tests (tests/api_tests.rs):**
```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use piper_tts_server::api::routes::{create_router, AppState};

async fn app() -> Router {
    let state = Arc::new(AppState {
        tts: TtsService::new("./test_fixtures/voices".into()),
    });
    create_router(state)
}

#[tokio::test]
async fn health_check() {
    let app = app().await;
    
    let response = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn list_voices() {
    let app = app().await;
    
    let response = app
        .oneshot(Request::get("/api/voices").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert!(json["voices"].is_array());
}

#[tokio::test]
async fn speak_empty_text_returns_400() {
    let app = app().await;
    
    let response = app
        .oneshot(
            Request::post("/api/speak")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "", "voice": "test"}"#))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn speak_unknown_voice_returns_404() {
    let app = app().await;
    
    let response = app
        .oneshot(
            Request::post("/api/speak")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "hello", "voice": "nonexistent"}"#))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn speak_returns_wav() {
    let app = app().await;
    
    let response = app
        .oneshot(
            Request::post("/api/speak")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "hello", "voice": "test-voice"}"#))
                .unwrap()
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "audio/wav"
    );
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[0..4], b"RIFF");
}

#[tokio::test]
async fn static_files_served() {
    let app = app().await;
    
    let response = app
        .oneshot(Request::get("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

### 6.3 Test fixtures

Create minimal test fixtures:

```
test_fixtures/
└── voices/
    ├── test-voice.onnx        # Minimal valid ONNX (or mock)
    └── test-voice.onnx.json   # Valid config
```

For unit tests, consider mocking the ONNX inference to avoid needing real models.

### 6.4 CI configuration

**.github/workflows/ci.yml:**
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y espeak-ng libespeak-ng-dev
    
    - name: Install Rust
      uses: dtolnay/rust-action@stable
    
    - name: Cache cargo
      uses: Swatinem/rust-cache@v2
    
    - name: Check formatting
      run: cargo fmt --check
    
    - name: Clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      run: cargo test
    
    - name: Build release
      run: cargo build --release

  docker:
    runs-on: ubuntu-latest
    needs: test
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Build Docker image
      run: docker build -t piper-tts-server .
    
    - name: Test Docker image
      run: |
        docker run -d -p 3000:3000 --name test piper-tts-server
        sleep 5
        curl -f http://localhost:3000/api/health
        docker stop test
```

### 6.5 Dockerfile

```dockerfile
# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    espeak-ng \
    libespeak-ng-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source
COPY src ./src
COPY static ./static

# Build for release
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    espeak-ng \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/piper-tts-server .

# Copy static files
COPY static ./static

# Create voices directory
RUN mkdir voices

# Environment
ENV HOST=0.0.0.0
ENV PORT=3000
ENV VOICES_DIR=/app/voices
ENV LOG_LEVEL=info

EXPOSE 3000

CMD ["./piper-tts-server"]
```

### 6.6 Docker Compose (for easy local dev)

**docker-compose.yml:**
```yaml
version: '3.8'

services:
  tts:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - ./voices:/app/voices:ro
    environment:
      - LOG_LEVEL=debug
    restart: unless-stopped
```

### 6.7 Deployment options

**Option A: Systemd service**

**/etc/systemd/system/piper-tts.service:**
```ini
[Unit]
Description=Piper TTS Server
After=network.target

[Service]
Type=simple
User=tts
WorkingDirectory=/opt/piper-tts
ExecStart=/opt/piper-tts/piper-tts-server
Environment=HOST=127.0.0.1
Environment=PORT=3000
Environment=VOICES_DIR=/opt/piper-tts/voices
Environment=LOG_LEVEL=info
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

**Option B: Docker**
```bash
docker run -d \
  --name piper-tts \
  -p 3000:3000 \
  -v /path/to/voices:/app/voices:ro \
  --restart unless-stopped \
  piper-tts-server
```

### 6.8 Reverse proxy (optional)

**Nginx config:**
```nginx
server {
    listen 80;
    server_name tts.example.com;
    
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        
        # For large audio responses
        proxy_buffering off;
        proxy_read_timeout 300s;
    }
}
```

## Acceptance Criteria

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy` has no warnings
- [ ] Docker image builds
- [ ] Docker container starts and responds
- [ ] CI pipeline green

## Release Checklist

1. [ ] All tests passing
2. [ ] Version bumped in Cargo.toml
3. [ ] CHANGELOG updated
4. [ ] Docker image tagged
5. [ ] Documentation current
6. [ ] Example voices documented

## Notes

- Consider adding benchmarks for TTS performance
- Monitor memory usage with multiple voices loaded
- Log audio generation times for performance tracking
- Consider health check that verifies TTS works, not just HTTP
