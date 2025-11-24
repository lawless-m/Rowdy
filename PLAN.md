# Piper TTS Server — Implementation Plan

## Project Summary

A lightweight, self-hosted text-to-speech server using Piper (ONNX), with a custom expression DSL for controlling speech pacing and emphasis.

**Stack:**
- Rust (axum, ort, tokio)
- Piper ONNX models
- Vanilla HTML/CSS/JS frontend

**Target:**
- Linux server
- Cross-platform web client
- 8GB+ GPU friendly (CPU also works)

---

## Phases

| Phase | Focus | Key Deliverables |
|-------|-------|------------------|
| 1 | [Setup](./phase-1-setup.md) | Project structure, dependencies, health endpoint |
| 2 | [DSL](./phase-2-dsl.md) | Expression parser & transforms |
| 3 | [Piper](./phase-3-piper.md) | ONNX inference, phonemization, WAV output |
| 4 | [API](./phase-4-api.md) | HTTP endpoints, error handling |
| 5 | [Frontend](./phase-5-frontend.md) | Web UI with play/save |
| 6 | [Testing](./phase-6-testing.md) | Tests, CI, Docker, deployment |

---

## Quick Start for Claude Code

### Phase 1 — Get the skeleton running

```bash
# Read the plan
cat plans/phase-1-setup.md

# Create project
cargo init piper-tts-server
cd piper-tts-server

# Add dependencies to Cargo.toml
# Create module structure
# Implement basic axum server
# GET /api/health returns {"status": "ok"}

cargo run
curl http://localhost:3000/api/health
```

### Phase 2 — DSL first (no dependencies on Piper)

```bash
cat plans/phase-2-dsl.md

# Implement src/dsl/parser.rs
# Implement src/dsl/transforms.rs
# Write tests

cargo test
```

### Phase 3 — Piper integration

```bash
cat plans/phase-3-piper.md

# Download a test voice first
mkdir -p voices && cd voices
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx.json
cd ..

# Implement voice loading
# Implement ONNX inference
# Implement WAV encoding
# Test synthesis

cargo run
```

### Phase 4 — Wire up the API

```bash
cat plans/phase-4-api.md

# Implement POST /api/speak
# Implement GET /api/voices
# Add error handling

curl -X POST http://localhost:3000/api/speak \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world", "voice": "en_GB-alba-medium"}' \
  --output test.wav

aplay test.wav
```

### Phase 5 — Frontend

```bash
cat plans/phase-5-frontend.md

# Create static/index.html
# Create static/style.css
# Create static/app.js

# Open http://localhost:3000 in browser
```

### Phase 6 — Polish

```bash
cat plans/phase-6-testing.md

# Write tests
# Create Dockerfile
# Set up CI

cargo test
docker build -t piper-tts-server .
docker run -p 3000:3000 -v ./voices:/app/voices piper-tts-server
```

---

## Key Files Reference

```
piper-tts-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, config, server startup
│   ├── lib.rs               # Module exports
│   ├── error.rs             # AppError enum
│   ├── api/
│   │   ├── mod.rs           # Request/response types
│   │   ├── routes.rs        # Router setup
│   │   └── handlers.rs      # Endpoint handlers
│   ├── dsl/
│   │   ├── mod.rs           # Public interface
│   │   ├── parser.rs        # Tag parsing
│   │   └── transforms.rs    # Text transforms
│   └── tts/
│       ├── mod.rs           # TtsService
│       ├── piper.rs         # ONNX inference
│       └── voice.rs         # Voice config loading
├── static/
│   ├── index.html
│   ├── app.js
│   └── style.css
├── voices/                  # Piper .onnx models
├── plans/                   # These implementation plans
├── docs/
│   └── README.md            # User documentation
└── tests/
    ├── dsl_tests.rs
    └── api_tests.rs
```

---

## Dependencies Summary

**Cargo.toml:**
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["fs", "cors"] }
ort = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
hound = "3"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
regex = "1"
lazy_static = "1"
```

**System:**
```bash
# Ubuntu/Debian
sudo apt install espeak-ng libespeak-ng-dev
```

---

## Notes for Implementation

1. **Start with DSL** — It's self-contained, easy to test, builds confidence
2. **Stub TTS early** — Return a silent WAV so API work isn't blocked
3. **Test with real voices** — Download at least one voice before Phase 3
4. **Keep frontend simple** — No build step, no frameworks
5. **Error messages matter** — Users will see them in the browser

---

## Success Criteria

- [ ] Server starts, serves static files
- [ ] DSL parses and transforms correctly
- [ ] Piper generates audible speech
- [ ] API returns WAV audio
- [ ] Web UI plays and saves audio
- [ ] Docker image builds and runs
- [ ] Tests pass, CI green
