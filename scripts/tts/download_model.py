#!/usr/bin/env python3
"""Download the Qwen3-TTS model into the HuggingFace cache.

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

        print("Downloading Qwen3-TTS model (this may take a few minutes)...")
        snapshot_download(
            repo_id="mlx-community/Qwen3-TTS-12Hz-0.6B-CustomVoice-bf16",
            cache_dir=os.path.join(hf_home, "hub"),
        )
        print("Download complete")
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
