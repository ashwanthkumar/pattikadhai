#!/usr/bin/env python3
"""Generate speech audio from text using Kokoro TTS via mlx-audio."""

import argparse
import os
import sys

# Suppress "PyTorch was not found" warning from transformers —
# mlx-audio uses MLX as its backend, not PyTorch.
os.environ["TRANSFORMERS_NO_ADVISORY_WARNINGS"] = "1"


def main():
    parser = argparse.ArgumentParser(description="Generate TTS audio")
    parser.add_argument("--text", required=True, help="Text to speak")
    parser.add_argument("--output", required=True, help="Output WAV file path")
    parser.add_argument("--voice", default="af_nova", help="Voice preset (default: af_nova)")
    parser.add_argument("--speed", type=float, default=1.0, help="Speech speed multiplier (default: 1.0)")
    args = parser.parse_args()

    try:
        from mlx_audio.tts.utils import load_model
        import soundfile as sf
        import numpy as np

        model = load_model("mlx-community/Kokoro-82M-bf16")

        audio_chunks = []
        sample_rate = 24000
        for result in model.generate(
            text=args.text,
            voice=args.voice,
            speed=args.speed,
        ):
            print("Generated audio chunk.", file=sys.stderr, flush=True)
            chunk = result.audio
            if hasattr(chunk, 'tolist'):
                chunk = np.array(chunk.tolist(), dtype=np.float32)
            audio_chunks.append(chunk)
            sample_rate = getattr(result, 'sr', 24000)

        if not audio_chunks:
            print("Error: No audio generated", file=sys.stderr)
            sys.exit(1)

        audio_data = np.concatenate(audio_chunks) if len(audio_chunks) > 1 else audio_chunks[0]
        sf.write(args.output, audio_data, sample_rate)
        print(f"Audio saved to {args.output}")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
