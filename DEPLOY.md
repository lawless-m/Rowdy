cd piper-tts-server

# Download a voice model
mkdir -p voices && cd voices
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/en_GB-alba-medium.onnx.json
cd ..

# Install espeak-ng (for phonemization)
sudo apt install espeak-ng

# Run the server
cargo run
# Open http://localhost:3000
