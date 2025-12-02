# Piper TTS Server - Deployment Guide

## Overview

This guide covers deploying the Piper TTS server to a remote server and setting up a local proxy for audio playback.

## Remote Server Deployment

### Prerequisites

- Linux server with CPU (no GPU required)
- Rust toolchain installed
- Nginx (for reverse proxy)
- Voice model files (`.onnx` and `.onnx.json`)

### 1. Clone and Build

```bash
git clone <repository-url>
cd piper-tts-server

# Build release version
cargo build --release
```

### 2. Install Voice Models

```bash
# Create voices directory
mkdir -p voices

# Copy your voice model files
# Example: en_GB-alba-medium.onnx and en_GB-alba-medium.onnx.json
cp /path/to/voices/*.onnx voices/
cp /path/to/voices/*.onnx.json voices/
```

### 3. Configure Server Port

Set the server to run on port 7734:

```bash
export PORT=7734
export HOST=127.0.0.1
export VOICES_DIR=./voices
```

Or create a systemd service file at `/etc/systemd/system/piper-tts.service`:

```ini
[Unit]
Description=Piper TTS Server
After=network.target

[Service]
Type=simple
User=www-data
WorkingDirectory=/path/to/piper-tts-server
Environment="PORT=7734"
Environment="HOST=127.0.0.1"
Environment="VOICES_DIR=/path/to/piper-tts-server/voices"
ExecStart=/path/to/piper-tts-server/target/release/piper-tts-server
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl enable piper-tts
sudo systemctl start piper-tts
sudo systemctl status piper-tts
```

### 4. Apache Reverse Proxy Configuration

Enable required Apache modules:

```bash
sudo a2enmod proxy
sudo a2enmod proxy_http
sudo a2enmod headers
```

Add to your Apache virtual host configuration (e.g., `/etc/apache2/sites-available/dw.ramsden-international.com.conf`):

```apache
<Location /tts/>
    ProxyPass http://127.0.0.1:7734/
    ProxyPassReverse http://127.0.0.1:7734/

    # Pass headers
    ProxyPreserveHost On
    RequestHeader set X-Forwarded-Proto "https"
    RequestHeader set X-Forwarded-For %{REMOTE_ADDR}s

    # Increase timeouts for audio generation
    ProxyTimeout 60
</Location>
```

Reload Apache:

```bash
sudo apachectl configtest
sudo systemctl reload apache2
```

### 5. Access the Service

The TTS server will be accessible at:
- **Web Interface**: https://dw.ramsden-international.com/tts/
- **API Endpoints**:
  - `POST https://dw.ramsden-international.com/tts/api/speak` - Generate and download WAV
  - `POST https://dw.ramsden-international.com/tts/api/speak-aloud` - Generate and play on server
  - `GET https://dw.ramsden-international.com/tts/api/voices` - List available voices
  - `GET https://dw.ramsden-international.com/tts/api/health` - Health check

## Local Proxy for Audio Playback

For local development, create a simple proxy that fetches audio from the remote TTS server and plays it locally.

### Local Proxy Script

Create `local-tts-proxy.sh`:

```bash
#!/bin/bash
# Local TTS Proxy - Fetches from remote server and plays locally

if [ $# -eq 0 ]; then
    echo "Usage: local-tts-proxy <text>"
    echo "Example: local-tts-proxy \"Hello, world\""
    exit 1
fi

TEXT="$*"
REMOTE_URL="https://dw.ramsden-international.com/tts/api/speak"
VOICE="en_GB-alba-medium"
TEMP_FILE="/tmp/tts-proxy-$$.wav"

# Fetch audio from remote server
curl -s -X POST "$REMOTE_URL" \
  -H "Content-Type: application/json" \
  -d "{\"text\": \"$TEXT\", \"voice\": \"$VOICE\"}" \
  --output "$TEMP_FILE"

# Check if curl succeeded
if [ $? -eq 0 ] && [ -f "$TEMP_FILE" ]; then
    # Calculate wait time based on text length
    WAIT_TIME=${#TEXT}

    # Play audio in background and clean up after
    (aplay -q "$TEMP_FILE" 2>/dev/null || \
     paplay "$TEMP_FILE" 2>/dev/null || \
     ffplay -nodisp -autoexit "$TEMP_FILE" 2>/dev/null || \
     echo "Error: No audio player found"; \
     sleep $WAIT_TIME; \
     rm -f "$TEMP_FILE") &
else
    echo "Error: Failed to fetch audio from remote server"
    exit 1
fi
```

Make it executable:

```bash
chmod +x local-tts-proxy.sh
cp local-tts-proxy.sh ~/bin/tts  # Or wherever you keep local scripts
```

## Testing

### Test Remote Server

```bash
# Health check
curl https://dw.ramsden-international.com/tts/api/health

# List voices
curl https://dw.ramsden-international.com/tts/api/voices

# Generate speech
curl -X POST https://dw.ramsden-international.com/tts/api/speak \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world", "voice": "en_GB-alba-medium"}' \
  --output test.wav
```

### Test Local Proxy

```bash
tts "This is a test of the local proxy"
```

## Monitoring

### Check Service Status

```bash
sudo systemctl status piper-tts
```

### View Logs

```bash
# Recent logs
sudo journalctl -u piper-tts -n 100

# Follow logs
sudo journalctl -u piper-tts -f
```

### Check Process

```bash
ps aux | grep piper-tts-server
netstat -tlnp | grep 7734
```

## Troubleshooting

### Server won't start
- Check logs: `sudo journalctl -u piper-tts -n 50`
- Verify port is available: `netstat -tlnp | grep 7734`
- Check voice files exist: `ls -la voices/`

### Apache 502/503 Errors
- Verify server is running: `systemctl status piper-tts`
- Check server is listening: `curl http://127.0.0.1:7734/api/health`
- Review Apache error log: `sudo tail -f /var/log/apache2/error.log`
- Check proxy modules are enabled: `apache2ctl -M | grep proxy`

### Audio not playing locally
- Verify audio player is installed: `which aplay` or `which paplay`
- Test audio player: `speaker-test -t wav -c 2`
- Check audio file is valid: `file /tmp/tts-*.wav`

## Security Considerations

1. **Firewall**: Ensure port 7734 is NOT exposed externally, only accessible via localhost
2. **Rate Limiting**: Consider adding nginx rate limiting for the TTS endpoints
3. **Input Validation**: The server validates text length (max 10000 chars)
4. **HTTPS**: Always use HTTPS for the public endpoint

## Performance

- **CPU Usage**: Piper is CPU-only, expect 1-2 seconds per sentence on modern CPUs
- **Memory**: ~200MB base + model size (~60MB per voice)
- **Concurrency**: Handles multiple requests, limited by CPU cores

## Updating

```bash
cd piper-tts-server
git pull
cargo build --release
sudo systemctl restart piper-tts
```
