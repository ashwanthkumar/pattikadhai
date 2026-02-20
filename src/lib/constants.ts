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
  { name: "gemma3:4b", label: "Gemma 3 4B", description: "Story generation model" },
  { name: "ffmpeg", label: "ffmpeg", description: "Audio processing tool" },
  {
    name: "espeak_ng",
    label: "espeak-ng",
    description: "Phonemization engine",
  },
  {
    name: "tts_model",
    label: "TTS Model",
    description: "Kokoro voice model (~115 MB)",
  },
] as const;
