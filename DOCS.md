# Piper TTS Server

A lightweight, model-neutral text-to-speech server written in Rust, with a custom DSL for expression control.

## Overview

This project provides a simple HTTP API for text-to-speech conversion using Piper (via ONNX runtime), served from a Linux machine and accessible from any browser.

### Architecture

```
┌─────────────────────────────────────────┐
│            Linux Server                 │
│  ┌─────────────────────────────────┐    │
│  │            axum                 │    │
│  │  ┌───────────┐ ┌─────────────┐  │    │
│  │  │  /api/*   │ │  /static/*  │  │    │
│  │  └─────┬─────┘ └─────────────┘  │    │
│  └────────┼────────────────────────┘    │
│           ▼                             │
│  ┌─────────────────────────────────┐    │
│  │         DSL Parser              │    │
│  │   [pause] → ... transforms      │    │
│  └─────────────┬───────────────────┘    │
│                ▼                        │
│  ┌─────────────────────────────────┐    │
│  │      Piper (ONNX Runtime)       │    │
│  │      Voice model loading        │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
           ▲
           │ HTTP (POST text, receive audio)
           ▼
┌─────────────────────────────────────────┐
│              Browser                    │
│   Text input → Listen → Save            │
└─────────────────────────────────────────┘
```

## Expression DSL

Piper doesn't natively support expressive markup, so we provide a simple DSL that transforms into Piper-friendly text using punctuation and formatting tricks.

### Syntax Reference

| Tag | Example | Transforms To | Purpose |
|-----|---------|---------------|---------|
| `[pause]` | `Hello [pause] world` | `Hello... world` | Short pause |
| `[pause:N]` | `Wait [pause:800] here` | `Wait...... here` | Pause in milliseconds (scaled to ellipses) |
| `[slow]...[/slow]` | `[slow]careful[/slow]` | `care...ful...` | Slower pacing |
| `[fast]...[/fast]` | `[fast]quickly now[/fast]` | `quickly now` | Faster pacing (strips pauses) |
| `[emphasis]...[/emphasis]` | `[emphasis]really[/emphasis]` | `REALLY` | Emphasise word |
| `[spell]...[/spell]` | `[spell]BBC[/spell]` | `B. B. C.` | Spell out letters |
| `[whisper]...[/whisper]` | `[whisper]secret[/whisper]` | `(secret)` | Quieter/softer hint |

### How It Works

The DSL parser runs before text is sent to Piper. It's purely text transformation—no special audio processing. The effectiveness depends on how well Piper interprets punctuation cues, which varies by voice model.

**Best results:**
- Punctuation (commas, ellipses, full stops) reliably affects pacing
- Question marks change intonation
- Some voices are more expressive than others

**Limitations:**
- No true volume control
- No pitch manipulation
- Expression is suggestive, not guaranteed

### Examples

**Input:**
```
Welcome to the system. [pause] Please [emphasis]listen carefully[/emphasis].

[slow]This next part is important.[/slow]

The code is [spell]PIN[/spell] [pause:500] one two three four.

[whisper]Don't tell anyone.[/whisper]
```

**Output (sent to Piper):**
```
Welcome to the system... Please LISTEN CAREFULLY.

This... next... part... is... important...

The code is P. I. N. ...... one two three four.

(don't tell anyone)
```

## API Reference

### `POST /api/speak`

Generate speech from text.

**Request:**
```json
{
  "text": "Hello [pause] world",
  "voice": "en_GB-alba-medium"
}
```

**Response:**
- Content-Type: `audio/wav`
- Body: WAV audio bytes

**Errors:**
- `400` — Invalid request (empty text, unknown voice)
- `500` — TTS generation failed

### `GET /api/voices`

List available voice models.

**Response:**
```json
{
  "voices": [
    {
      "id": "en_GB-alba-medium",
      "name": "Alba (British English)",
      "language": "en_GB",
      "quality": "medium"
    }
  ]
}
```

### `GET /api/health`

Health check endpoint.

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

## Web Interface

The bundled web UI provides:

- Text input area with DSL support
- Voice selection dropdown
- Play button — hear the result
- Save button — download as WAV (enabled after playback)

Access at `http://localhost:3000` when the server is running.

## Voice Models

Piper uses ONNX models. Each voice requires two files:
- `{voice}.onnx` — the model
- `{voice}.onnx.json` — configuration

Place these in the `voices/` directory.

### Downloading Voices

Voices are available from the Piper project:
https://github.com/rhasspy/piper/blob/master/VOICES.md

Recommended starting voices:
- `en_GB-alba-medium` — British English, female
- `en_GB-aru-medium` — British English, male
- `en_US-lessac-medium` — American English, female

### Voice Naming Convention

Piper voices follow the pattern: `{language}-{name}-{quality}`

- **language**: e.g., `en_GB`, `en_US`, `de_DE`
- **name**: voice name
- **quality**: `low`, `medium`, or `high` (affects size and quality)

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Bind address |
| `PORT` | `3000` | Server port |
| `VOICES_DIR` | `./voices` | Path to voice models |
| `LOG_LEVEL` | `info` | Logging verbosity |

## Performance Notes

- First request for a voice loads the model (may take a few seconds)
- Subsequent requests reuse loaded models
- Models stay in memory — approximately 50–100 MB per voice
- Generation is synchronous per request; consider a queue for high load

## Limitations

- No streaming audio (full generation before response)
- No real-time synthesis
- Expression DSL is best-effort, not guaranteed
- SSML support depends on voice model (generally limited)
