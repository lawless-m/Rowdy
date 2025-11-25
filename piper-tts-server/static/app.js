const textInput = document.getElementById('text');
const voiceSelect = document.getElementById('voice');
const speakBtn = document.getElementById('speakBtn');
const saveBtn = document.getElementById('saveBtn');
const player = document.getElementById('player');
const status = document.getElementById('status');
const charCount = document.getElementById('charCount');

let audioBlob = null;
let lastText = '';

// Load voices on startup
async function loadVoices() {
    try {
        const res = await fetch('/api/voices');
        if (!res.ok) throw new Error('Failed to load voices');

        const data = await res.json();

        if (data.voices.length === 0) {
            voiceSelect.innerHTML = '<option value="">No voices available</option>';
            showStatus('No voice models found. Add .onnx files to the voices directory.', true);
            return;
        }

        voiceSelect.innerHTML = data.voices
            .map(v => `<option value="${v.id}">${v.name || v.id} (${v.language})</option>`)
            .join('');

    } catch (err) {
        voiceSelect.innerHTML = '<option value="">Error loading voices</option>';
        showStatus('Failed to load voices: ' + err.message, true);
    }
}

// Generate speech
async function speak() {
    const text = textInput.value.trim();
    const voice = voiceSelect.value;

    if (!text) {
        showStatus('Please enter some text', true);
        return;
    }

    if (!voice) {
        showStatus('Please select a voice', true);
        return;
    }

    if (text.length > 10000) {
        showStatus('Text too long (max 10000 characters)', true);
        return;
    }

    // Disable buttons, show loading
    speakBtn.disabled = true;
    speakBtn.classList.add('loading');
    saveBtn.disabled = true;
    showStatus('Generating...');

    try {
        const res = await fetch('/api/speak', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text, voice }),
        });

        if (!res.ok) {
            const err = await res.json();
            throw new Error(err.error || 'Generation failed');
        }

        audioBlob = await res.blob();
        lastText = text;

        // Revoke previous URL if exists
        if (player.src && player.src.startsWith('blob:')) {
            URL.revokeObjectURL(player.src);
        }

        const url = URL.createObjectURL(audioBlob);
        player.src = url;
        player.play();

        saveBtn.disabled = false;
        showStatus('');

    } catch (err) {
        showStatus(err.message, true);
    } finally {
        speakBtn.disabled = false;
        speakBtn.classList.remove('loading');
    }
}

// Save audio
function save() {
    if (!audioBlob) return;

    // Generate filename from text
    const filename = generateFilename(lastText);

    const a = document.createElement('a');
    a.href = URL.createObjectURL(audioBlob);
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
}

// Generate a sensible filename
function generateFilename(text) {
    const words = text
        .replace(/\[.*?\]/g, '')  // Remove DSL tags
        .trim()
        .split(/\s+/)
        .slice(0, 5)
        .join('_')
        .replace(/[^a-zA-Z0-9_]/g, '')
        .toLowerCase();

    const timestamp = new Date().toISOString().slice(0, 10);

    return `${words || 'speech'}_${timestamp}.wav`;
}

// Update character count
function updateCharCount() {
    const count = textInput.value.length;
    charCount.textContent = count;

    if (count > 10000) {
        charCount.parentElement.classList.add('over');
    } else {
        charCount.parentElement.classList.remove('over');
    }
}

// Show status message
function showStatus(message, isError = false) {
    status.textContent = message;
    status.className = 'status' + (isError ? ' error' : '');
}

// Event listeners
speakBtn.addEventListener('click', speak);
saveBtn.addEventListener('click', save);
textInput.addEventListener('input', updateCharCount);

// Keyboard shortcut: Ctrl+Enter to speak
textInput.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 'Enter') {
        e.preventDefault();
        speak();
    }
});

// Init
loadVoices();
updateCharCount();
