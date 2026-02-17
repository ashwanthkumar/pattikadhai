#!/usr/bin/env python3
"""Check if the Qwen3-TTS model is present in the HuggingFace cache.

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
    # HF stores repos as models--<org>--<repo>
    model_dir = hub_dir / "models--mlx-community--Qwen3-TTS-12Hz-0.6B-CustomVoice-bf16"

    # A valid snapshot has at least one refs/ entry and a snapshots/ dir
    snapshots = model_dir / "snapshots"
    if snapshots.is_dir() and any(snapshots.iterdir()):
        print("installed")
    else:
        print("missing")


if __name__ == "__main__":
    main()
