import { useState, useCallback, useRef, useEffect } from "react";
import type { TimingSegment } from "@/lib/timing";

export function useAudioHighlight(segments: TimingSegment[] | null) {
  const [activeIndex, setActiveIndex] = useState(-1);
  const [isPlaying, setIsPlaying] = useState(false);
  const audioElRef = useRef<HTMLAudioElement | null>(null);
  const rafRef = useRef<number>(0);

  const tick = useCallback(() => {
    const audio = audioElRef.current;
    if (!audio || !segments || audio.paused) {
      return;
    }

    const t = audio.currentTime;
    let idx = -1;
    for (let i = 0; i < segments.length; i++) {
      if (t >= segments[i].start && t < segments[i].end) {
        idx = i;
        break;
      }
    }
    setActiveIndex(idx);
    rafRef.current = requestAnimationFrame(tick);
  }, [segments]);

  const bindAudio = useCallback(
    (el: HTMLAudioElement | null) => {
      // Clean up old
      if (audioElRef.current) {
        audioElRef.current.removeEventListener("play", onPlay);
        audioElRef.current.removeEventListener("pause", onPause);
        audioElRef.current.removeEventListener("ended", onEnded);
        cancelAnimationFrame(rafRef.current);
      }

      audioElRef.current = el;
      if (!el) return;

      el.addEventListener("play", onPlay);
      el.addEventListener("pause", onPause);
      el.addEventListener("ended", onEnded);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [tick],
  );

  function onPlay() {
    setIsPlaying(true);
    rafRef.current = requestAnimationFrame(tick);
  }

  function onPause() {
    setIsPlaying(false);
    cancelAnimationFrame(rafRef.current);
  }

  function onEnded() {
    setIsPlaying(false);
    setActiveIndex(-1);
    cancelAnimationFrame(rafRef.current);
  }

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      cancelAnimationFrame(rafRef.current);
    };
  }, []);

  return { activeIndex, isPlaying, bindAudio };
}
