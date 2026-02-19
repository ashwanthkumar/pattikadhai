#!/usr/bin/env python3
"""Download the Kokoro TTS model into the HuggingFace cache.

Expects HF_HOME env var to point at the custom cache directory.
Uses huggingface_hub (bundled with mlx-audio) so no GPU is required.
"""

import os
import sys


def main():
    hf_home = os.environ.get("HF_HOME")
    if not hf_home:
        print("Error: HF_HOME env var not set", file=sys.stderr)
        sys.exit(1)

    os.makedirs(hf_home, exist_ok=True)

    try:
        from huggingface_hub import snapshot_download

        hub_cache = os.path.join(hf_home, "hub")

        print("Downloading Kokoro TTS model...")
        snapshot_download(
            repo_id="mlx-community/Kokoro-82M-bf16",
            cache_dir=hub_cache,
        )

        # Kokoro loads voice files from its original repo (prince-canuma/Kokoro-82M)
        print("Downloading Kokoro voice files...")
        snapshot_download(
            repo_id="prince-canuma/Kokoro-82M",
            allow_patterns=["voices/*"],
            cache_dir=hub_cache,
        )

        # Kokoro uses misaki which needs a spacy English model for phonemization
        import spacy
        if not spacy.util.is_package("en_core_web_sm"):
            print("Downloading spacy English model...")
            spacy.cli.download("en_core_web_sm")

        print("Download complete")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
