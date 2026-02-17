# PattiKadhai

A desktop app that generates children's stories with AI narration and background music. Built with Tauri 2 (React + Rust).

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- [uv](https://docs.astral.sh/uv/) (Python package manager)
- [Ollama](https://ollama.com/) with a model pulled (e.g. `ollama pull llama3.2`)
- [ffmpeg](https://ffmpeg.org/) (for audio mixing)
- Apple Silicon Mac (required for mlx-audio TTS and ACE-Step music generation)

## Setup

```bash
# Install frontend dependencies
npm install

# Install Python dependencies for TTS and music scripts
cd scripts/tts && uv sync && cd ../..
cd scripts/music && uv sync && cd ../..
```

## Run

```bash
npx tauri dev
```

## Build

```bash
npx tauri build
```

## Tests

```bash
# Rust tests
cd src-tauri && cargo test

# Frontend tests
npm test
```

## License

MIT
