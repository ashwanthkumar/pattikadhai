#!/usr/bin/env python3
"""Download the KittenTTS model into the HuggingFace cache.

Expects HF_HOME env var to point at the custom cache directory.
"""

import os
import sys

from config import MODEL_ID


def main():
    hf_home = os.environ.get("HF_HOME")
    if not hf_home:
        print("Error: HF_HOME env var not set", file=sys.stderr)
        sys.exit(1)

    os.makedirs(hf_home, exist_ok=True)

    try:
        from kittentts import KittenTTS

        print("Downloading KittenTTS model...")
        KittenTTS(MODEL_ID)
        print("Download complete")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
