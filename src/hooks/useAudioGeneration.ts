import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { startAudioGeneration } from "@/lib/api";
import { createAudioJob } from "@/lib/database";
import type { PipelineProgress } from "@/types";

type AudioStage =
  | "idle"
  | "voice_generating"
  | "music_generating"
  | "mixing"
  | "complete"
  | "failed";

export function useAudioGeneration() {
  const [stage, setStage] = useState<AudioStage>("idle");
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [jobId, setJobId] = useState<string | null>(null);
  const [finalPath, setFinalPath] = useState<string | null>(null);

  useEffect(() => {
    const unlisten = listen<PipelineProgress>("audio-progress", (event) => {
      const data = event.payload;
      if (jobId && data.job_id === jobId) {
        setStage(data.stage as AudioStage);
        setProgress(data.progress);
        if (data.error) {
          setError(data.error);
        }
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [jobId]);

  const startGeneration = useCallback(
    async (partId: string, text: string, genre: string) => {
      const newJobId = crypto.randomUUID();
      setJobId(newJobId);
      setStage("voice_generating");
      setProgress(0);
      setError(null);
      setFinalPath(null);

      try {
        await createAudioJob(newJobId, partId);
        await startAudioGeneration(newJobId, partId, text, genre);
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        setStage("failed");
      }
    },
    [],
  );

  const reset = useCallback(() => {
    setStage("idle");
    setProgress(0);
    setError(null);
    setJobId(null);
    setFinalPath(null);
  }, []);

  return { stage, progress, error, jobId, finalPath, startGeneration, reset };
}
