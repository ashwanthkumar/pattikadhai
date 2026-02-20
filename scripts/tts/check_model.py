#!/usr/bin/env python3
"""Check if the KittenTTS model is present in the HuggingFace cache.

Expects HF_HOME env var to point at the custom cache directory.
Prints "installed" or "missing" and exits 0 either way.
"""

import sys

from config import MODEL_ID


def main():
    try:
        from kittentts import KittenTTS

        KittenTTS(MODEL_ID)
        print("installed")
    except Exception:
        print("missing")


if __name__ == "__main__":
    main()
