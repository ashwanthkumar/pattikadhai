#!/usr/bin/env python3
"""Generate speech audio from text using Qwen3-TTS via mlx-audio."""

import argparse
import sys


def main():
    parser = argparse.ArgumentParser(description="Generate TTS audio")
    parser.add_argument("--text", required=True, help="Text to speak")
    parser.add_argument("--output", required=True, help="Output WAV file path")
    parser.add_argument("--voice", default="Vivian", help="Speaker name (default: Vivian)")
    parser.add_argument("--seed", type=int, default=42, help="Random seed for reproducible voice (default: 42)")
    parser.add_argument("--temperature", type=float, default=0.3, help="Sampling temperature (default: 0.3)")
    args = parser.parse_args()

    try:
        from mlx_audio.tts.utils import load_model
        import mlx.core as mx
        import soundfile as sf
        import numpy as np

        mx.random.seed(args.seed)

        model = load_model("mlx-community/Qwen3-TTS-12Hz-0.6B-CustomVoice-bf16")

        results = list(model.generate_custom_voice(
            text=args.text,
            speaker=args.voice,
            language="English",
            instruct="Warm and gentle storytelling voice, like a grandmother telling a bedtime story.",
            temperature=args.temperature,
        ))

        if not results:
            print("Error: No audio generated", file=sys.stderr)
            sys.exit(1)

        audio_data = results[0].audio
        # Convert mlx array to numpy if needed
        if hasattr(audio_data, 'tolist'):
            audio_data = np.array(audio_data.tolist(), dtype=np.float32)

        sample_rate = getattr(results[0], 'sr', 24000)
        sf.write(args.output, audio_data, sample_rate)
        print(f"Audio saved to {args.output}")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
