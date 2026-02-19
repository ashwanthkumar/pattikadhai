export const GENRE_ICONS: Record<string, string> = {
  adventure: "compass",
  fantasy: "sparkles",
  moral: "heart",
  bedtime: "moon",
  animal: "paw-print",
  science: "flask",
};

export const AUDIO_STAGES = [
  { key: "voice_generating", label: "Generating Voice" },
  { key: "complete", label: "Done" },
] as const;

export const DEPENDENCY_STEPS = [
  { name: "ollama", label: "Ollama", description: "Local AI model server" },
  { name: "gemma3", label: "Gemma 3", description: "Story generation model" },
  { name: "uv", label: "uv", description: "Python package manager" },
  { name: "ffmpeg", label: "ffmpeg", description: "Audio processing tool" },
  {
    name: "python_deps",
    label: "Python ML deps",
    description: "mlx-audio TTS library",
  },
  {
    name: "tts_model",
    label: "TTS Model",
    description: "Kokoro voice model (~355 MB)",
  },
] as const;
