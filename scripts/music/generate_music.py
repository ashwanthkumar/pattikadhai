#!/usr/bin/env python3
"""Generate background music using ACE-Step 1.5."""

import argparse
import sys


def main():
    parser = argparse.ArgumentParser(description="Generate background music")
    parser.add_argument("--genre", required=True, help="Music style caption")
    parser.add_argument("--duration", type=int, default=60, help="Duration in seconds")
    parser.add_argument("--output", required=True, help="Output WAV file path")
    parser.add_argument("--project-root", default=None, help="ACE-Step project root (contains checkpoints/)")
    args = parser.parse_args()

    try:
        import numpy as np
        import soundfile as sf
        from acestep.handler import AceStepHandler

        handler = AceStepHandler()

        # initialize_service needs project_root (where checkpoints live)
        # and config_path (model variant). Models auto-download on first run.
        status, success = handler.initialize_service(
            project_root=args.project_root,
            config_path="acestep-v15-turbo",
            device="mps",
        )
        if not success:
            print(f"Failed to initialize: {status}", file=sys.stderr)
            sys.exit(1)

        result = handler.generate_music(
            captions=args.genre,
            lyrics="[Instrumental]",
            audio_duration=float(args.duration),
            inference_steps=8,
            guidance_scale=7.0,
            use_random_seed=True,
            task_type="text2music",
        )

        if not result.get("success", False):
            print(f"Generation failed: {result.get('error', 'Unknown error')}", file=sys.stderr)
            sys.exit(1)

        audios = result.get("audios", [])
        if not audios:
            print("No audio generated", file=sys.stderr)
            sys.exit(1)

        # Result contains tensors, write first one to WAV
        audio_dict = audios[0]
        tensor = audio_dict["tensor"]
        sample_rate = audio_dict.get("sample_rate", 44100)

        # Convert torch tensor to numpy
        audio_np = tensor.cpu().numpy()
        if audio_np.ndim == 2:
            audio_np = audio_np.T  # soundfile expects (samples, channels)

        sf.write(args.output, audio_np, sample_rate)
        print(f"Music saved to {args.output}")

    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
