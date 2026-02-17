#!/usr/bin/env python3
"""Check if the ACE-Step music model is present.

Expects --checkpoints-dir argument pointing to the checkpoints directory.
Prints "installed" or "missing" and exits 0 either way.
"""

import argparse
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description="Check ACE-Step model")
    parser.add_argument("--checkpoints-dir", required=True, help="Path to checkpoints directory")
    args = parser.parse_args()

    checkpoints = Path(args.checkpoints_dir)

    # ACE-Step stores the turbo DiT model under checkpoints/acestep-v15-turbo/
    turbo_dir = checkpoints / "acestep-v15-turbo"
    # Also needs the base model components
    has_turbo = turbo_dir.is_dir() and any(turbo_dir.iterdir())

    if has_turbo:
        print("installed")
    else:
        print("missing")


if __name__ == "__main__":
    main()
