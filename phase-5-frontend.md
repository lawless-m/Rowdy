# Phase 5: Web Frontend

## Overview
Create a simple, functional web UI for the TTS service.

## Design Principles

- No frameworks — vanilla HTML/CSS/JS
- Works without JavaScript for basic form submission (progressive enhancement)
- Clean, minimal interface
- Mobile-friendly
- Accessible

## Tasks

### 5.1 HTML structure (static/index.html)

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Piper TTS</title>
    <link rel="stylesheet" href="/style.css">
</head>
<body>
    <main>
        <h1>Piper TTS</h1>
        
        <div class="input-group">
            <label for="text">Text</label>
            <textarea 
                id="text" 
                rows="6" 
                placeholder="Enter text to speak... Use [pause], [slow], [emphasis] etc."
            ></textarea>
            <div class="char-count"><span id="charCount">0</span> / 10000</div>
        </div>
        
        <div class="input-group">
            <label for="voice">Voice</label>
            <select id="voice">
                <option value="">Loading voices...</option>
            </select>
        </div>
        
        <div class="button-group">
            <button id="speakBtn" type="button">Speak</button>
            <button id="saveBtn" type="button" disabled>Save</button>
        </div>
        
        <div id="status" class="status"></div>
        
        <audio id="player" controls></audio>
        
        <details class="dsl-help">
            <summary>DSL Reference</summary>
            <table>
                <tr><td><code>[pause]</code></td><td>Short pause</td></tr>
                <tr><td><code>[pause:500]</code></td><td>Pause (milliseconds)</td></tr>
                <tr><td><code>[slow]...[/slow]</code></td><td>Slower speech</td></tr>
                <tr><td><code>[fast]...[/fast]</code></td><td>Faster speech</td></tr>
                <tr><td><code>[emphasis]...[/emphasis]</code></td><td>Emphasise</td></tr>
                <tr><td><code>[spell]...[/spell]</code></td><td>Spell out letters</td></tr>
                <tr><td><code>[whisper]...[/whisper]</code></td><td>Whisper</td></tr>
            </table>
        </details>
    </main>
    
    <script src="/app.js"></script>
</body>
</html>
```

### 5.2 Styles (static/style.css)

```css
:root {
    --bg: #1a1a2e;
    --surface: #16213e;
    --primary: #e94560;
    --text: #eee;
    --text-muted: #888;
    --border: #333;
    --radius: 6px;
}

* {
    box-sizing: border-box;
}

body {
    font-family: system-ui, -apple-system, sans-serif;
    background: var(--bg);
    color: var(--text);
    margin: 0;
    padding: 1rem;
    min-height: 100vh;
}

main {
    max-width: 600px;
    margin: 0 auto;
}

h1 {
    font-weight: 300;
    margin-bottom: 2rem;
}

.input-group {
    margin-bottom: 1.5rem;
}

label {
    display: block;
    margin-bottom: 0.5rem;
    font-size: 0.9rem;
    color: var(--text-muted);
}

textarea, select {
    width: 100%;
    padding: 0.75rem;
    font-size: 1rem;
    font-family: inherit;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    resize: vertical;
}

textarea:focus, select:focus {
    outline: none;
    border-color: var(--primary);
}

.char-count {
    font-size: 0.8rem;
    color: var(--text-muted);
    text-align: right;
    margin-top: 0.25rem;
}

.button-group {
    display: flex;
    gap: 1rem;
    margin-bottom: 1.5rem;
}

button {
    flex: 1;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 500;
    background: var(--primary);
    color: white;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
    transition: opacity 0.2s;
}

button:hover:not(:disabled) {
    opacity: 0.9;
}

button:disabled {
    background: var(--border);
    cursor: not-allowed;
}

button.secondary {
    background: transparent;
    border: 1px solid var(--primary);
    color: var(--primary);
}

.status {
    min-height: 1.5rem;
    margin-bottom: 1rem;
    font-size: 0.9rem;
    color: var(--text-muted);
}

.status.error {
    color: var(--primary);
}

audio {
    width: 100%;
    margin-bottom: 2rem;
}

/* Hide audio player until there's something to play */
audio:not([src]), audio[src=""] {
    display: none;
}

.dsl-help {
    font-size: 0.9rem;
    color: var(--text-muted);
}

.dsl-help summary {
    cursor: pointer;
    margin-bottom: 1rem;
}

.dsl-help table {
    width: 100%;
    border-collapse: collapse;
}

.dsl-help td {
    padding: 0.5rem;
    border-bottom: 1px solid var(--border);
}

.dsl-help code {
    background: var(--surface);
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-size: 0.85rem;
}

/* Loading state */
button.loading {
    position: relative;
    color: transparent;
}

button.loading::after {
    content: "";
    position: absolute;
    width: 1rem;
    height: 1rem;
    top: 50%;
    left: 50%;
    margin: -0.5rem 0 0 -0.5rem;
    border: 2px solid white;
    border-top-color: transparent;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
}

@keyframes spin {
    to { transform: rotate(360deg); }
}

/* Responsive */
@media (max-width: 480px) {
    .button-group {
        flex-direction: column;
    }
}
```

### 5.3 JavaScript (static/app.js)

```javascript
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
        
        voiceSelect.innerHTML = data.voices
            .map(v => `<option value="${v.id}">${v.name || v.id}</option>`)
            .join('');
        
        if (data.voices.length === 0) {
            voiceSelect.innerHTML = '<option value="">No voices available</option>';
        }
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
    a.click();
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
        charCount.style.color = 'var(--primary)';
    } else {
        charCount.style.color = '';
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
        speak();
    }
});

// Init
loadVoices();
updateCharCount();
```

### 5.4 Additional features (optional)

**History panel:**
```javascript
// Store recent generations
const history = [];

function addToHistory(text, voice, blob) {
    history.unshift({ text, voice, blob, time: new Date() });
    if (history.length > 10) history.pop();
    renderHistory();
}

function renderHistory() {
    // Render list of previous generations
    // Each with play/download buttons
}
```

**Voice preview:**
```javascript
async function previewVoice(voiceId) {
    const res = await fetch('/api/speak', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
            text: 'This is a sample of my voice.', 
            voice: voiceId 
        }),
    });
    // Play preview
}
```

**DSL syntax highlighting:**
```javascript
// Could use a simple regex-based highlighter
// to show DSL tags in a different colour in the textarea
// (requires contenteditable div instead of textarea)
```

## Acceptance Criteria

- [ ] Page loads without errors
- [ ] Voices populate on load
- [ ] Text input works
- [ ] Character count updates
- [ ] Speak button generates audio
- [ ] Audio plays in browser
- [ ] Save button downloads WAV
- [ ] Loading state shows during generation
- [ ] Error messages display
- [ ] Ctrl+Enter shortcut works
- [ ] Mobile layout works
- [ ] DSL reference visible

## Browser Testing

- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Mobile Chrome/Safari

## Notes

- No build step required
- Could add dark/light theme toggle
- Consider adding a "copy DSL" button for common patterns
- Audio element styling varies by browser
