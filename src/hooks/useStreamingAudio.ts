import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { getAudioUrl } from "@/lib/audio";

interface SentenceAudioEvent {
  job_id: string;
  index: number;
  total: number;
  text: string;
  wav_path: string;
  duration_secs: number;
}

interface SentenceInfo {
  index: number;
  text: string;
  wavPath: string;
  durationSecs: number;
  blobUrl?: string;
}

type StreamState = "idle" | "buffering" | "playing" | "paused" | "complete";

const BUFFER_THRESHOLD = 5;

export function useStreamingAudio(jobId: string | null) {
  const [sentences, setSentences] = useState<SentenceInfo[]>([]);
  const [activeIndex, setActiveIndex] = useState(-1);
  const [state, setState] = useState<StreamState>("idle");
  const [totalExpected, setTotalExpected] = useState(0);

  const audioRef = useRef<HTMLAudioElement | null>(null);
  const allDoneRef = useRef(false);
  const playStartedRef = useRef(false);
  const sentencesRef = useRef<SentenceInfo[]>([]);

  // Keep ref in sync
  useEffect(() => {
    sentencesRef.current = sentences;
  }, [sentences]);

  // Listen for audio-sentence events
  useEffect(() => {
    if (!jobId) return;

    setState("buffering");
    setSentences([]);
    setActiveIndex(-1);
    allDoneRef.current = false;
    playStartedRef.current = false;

    const unlisten = listen<SentenceAudioEvent>(
      "audio-sentence",
      async (event) => {
        const data = event.payload;
        if (data.job_id !== jobId) return;

        setTotalExpected(data.total);

        // Load blob URL for playback
        const blobUrl = await getAudioUrl(data.wav_path);

        const newSentence: SentenceInfo = {
          index: data.index,
          text: data.text,
          wavPath: data.wav_path,
          durationSecs: data.duration_secs,
          blobUrl,
        };

        setSentences((prev) => {
          const next = [...prev, newSentence];
          return next;
        });
      },
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [jobId]);

  // Auto-start playback once buffer threshold reached
  useEffect(() => {
    if (playStartedRef.current) return;
    if (state !== "buffering") return;

    const readyCount = sentences.filter((s) => s.blobUrl).length;
    const allReceived = totalExpected > 0 && sentences.length >= totalExpected;
    const bufferReady = readyCount >= BUFFER_THRESHOLD || allReceived;

    if (bufferReady && readyCount > 0) {
      playStartedRef.current = true;
      setActiveIndex(0);
      setState("playing");
    }
  }, [sentences, state, totalExpected]);

  // Play the active sentence
  useEffect(() => {
    if (state !== "playing" || activeIndex < 0) return;

    const sentence = sentencesRef.current[activeIndex];
    if (!sentence?.blobUrl) return;

    const audio = new Audio(sentence.blobUrl);
    audioRef.current = audio;

    audio.onended = () => {
      const nextIndex = activeIndex + 1;
      const currentSentences = sentencesRef.current;

      if (nextIndex < currentSentences.length && currentSentences[nextIndex]?.blobUrl) {
        setActiveIndex(nextIndex);
      } else if (totalExpected > 0 && nextIndex >= totalExpected) {
        // All sentences played
        setState("complete");
        setActiveIndex(-1);
      } else {
        // Wait for more sentences — go back to buffering
        setState("buffering");
        setActiveIndex(nextIndex);
      }
    };

    audio.play().catch(() => {
      // Browser may block autoplay; that's OK, user can trigger via pause/play
    });

    return () => {
      audio.pause();
      audio.onended = null;
    };
  }, [activeIndex, state, totalExpected]);

  // If we were waiting for more sentences and they arrived, resume playing
  useEffect(() => {
    if (state !== "buffering" || activeIndex < 0) return;
    const sentence = sentences[activeIndex];
    if (sentence?.blobUrl) {
      setState("playing");
    }
  }, [sentences, state, activeIndex]);

  const pause = useCallback(() => {
    if (audioRef.current) {
      audioRef.current.pause();
    }
    setState("paused");
  }, []);

  const play = useCallback(() => {
    if (state === "paused" && audioRef.current) {
      audioRef.current.play();
      setState("playing");
    } else if (state === "complete") {
      // Restart from beginning
      setActiveIndex(0);
      setState("playing");
    }
  }, [state]);

  const isPlaying = state === "playing";
  const isBuffering = state === "buffering";

  return {
    sentences,
    activeIndex,
    isPlaying,
    isBuffering,
    state,
    play,
    pause,
  };
}
