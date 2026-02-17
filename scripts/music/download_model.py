#!/usr/bin/env python3
"""Download the ACE-Step music model.

Expects --checkpoints-dir argument pointing to the checkpoints directory.
Uses acestep.model_downloader so no GPU is required.
"""

import argparse
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Download ACE-Step model")
    parser.add_argument("--checkpoints-dir", required=True, help="Path to checkpoints directory")
    args = parser.parse_args()

    checkpoints = Path(args.checkpoints_dir)
    checkpoints.mkdir(parents=True, exist_ok=True)

    try:
        from acestep.model_downloader import download_main_model

        print("Downloading ACE-Step model (this may take several minutes)...")
        success, message = download_main_model(checkpoints_dir=checkpoints)
        print(message)
        if not success:
            sys.exit(1)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
