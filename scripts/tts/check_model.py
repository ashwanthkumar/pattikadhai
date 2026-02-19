#!/usr/bin/env python3
"""Check if the Kokoro TTS model is present in the HuggingFace cache.

Expects HF_HOME env var to point at the custom cache directory.
Prints "installed" or "missing" and exits 0 either way.
"""

import os
import sys
from pathlib import Path


def main():
    hf_home = os.environ.get("HF_HOME")
    if not hf_home:
        print("missing")
        sys.exit(0)

    hub_dir = Path(hf_home) / "hub"

    # Check both the MLX model weights and the voice files repo
    for repo_name in ["models--mlx-community--Kokoro-82M-bf16", "models--prince-canuma--Kokoro-82M"]:
        snapshots = hub_dir / repo_name / "snapshots"
        if not (snapshots.is_dir() and any(snapshots.iterdir())):
            print("missing")
            return

    # Also check that the spacy English model is installed (needed by misaki phonemizer)
    try:
        import spacy
        if not spacy.util.is_package("en_core_web_sm"):
            print("missing")
            return
    except ImportError:
        print("missing")
        return

    print("installed")


if __name__ == "__main__":
    main()
