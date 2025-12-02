# Build stage
FROM rust:1.83-bookworm as builder

WORKDIR /app

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    espeak-ng \
    libespeak-ng-dev \
    libasound2-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy src for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source
COPY src ./src

# Build for release with all features
RUN touch src/main.rs && cargo build --release

# Runtime stage - Using Debian 11 (bullseye) to match your deployment target
FROM debian:11-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    espeak-ng \
    libasound2 \
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
ENV RUST_LOG=info

EXPOSE 3000

CMD ["./piper-tts-server"]
