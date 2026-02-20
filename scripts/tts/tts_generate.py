#!/usr/bin/env python3
"""Generate speech audio from text using KittenTTS."""

import argparse
import sys

from config import MODEL_ID, DEFAULT_VOICE


def main():
    parser = argparse.ArgumentParser(description="Generate TTS audio")
    parser.add_argument("--text", required=True, help="Text to speak")
    parser.add_argument("--output", required=True, help="Output WAV file path")
    parser.add_argument("--voice", default=DEFAULT_VOICE, help=f"Voice preset (default: {DEFAULT_VOICE})")
    args = parser.parse_args()

    try:
        from kittentts import KittenTTS
        import soundfile as sf

        tts = KittenTTS(MODEL_ID)
        audio = tts.generate(text=args.text, voice=args.voice)

        sf.write(args.output, audio, 24000)
        print(f"Audio saved to {args.output}")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
