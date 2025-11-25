# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    espeak-ng \
    libespeak-ng-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy src for dependency caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source
COPY src ./src

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
ENV RUST_LOG=info

EXPOSE 3000

CMD ["./piper-tts-server"]
