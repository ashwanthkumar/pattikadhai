import { readTextFile } from "@tauri-apps/plugin-fs";

export interface TimingSegment {
  text: string;
  start: number;
  end: number;
}

/**
 * Load timing data for a given audio path.
 * Derives the timing JSON path from the audio path: `_final.wav` -> `_timing.json`.
 * Returns null if no timing file exists (legacy audio).
 */
export async function loadTimingData(
  audioPath: string,
): Promise<TimingSegment[] | null> {
  const timingPath = audioPath.replace(/_final\.wav$/, "_timing.json");
  if (timingPath === audioPath) return null; // path didn't match pattern

  try {
    const json = await readTextFile(timingPath);
    return JSON.parse(json) as TimingSegment[];
  } catch {
    return null; // file doesn't exist (legacy audio)
  }
}
