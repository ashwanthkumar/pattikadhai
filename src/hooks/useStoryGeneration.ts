import { useState, useCallback, useRef } from "react";
import { generateStoryText, continueStory } from "@/lib/api";

type GenerationState = "idle" | "generating" | "complete" | "error";

export function useStoryGeneration() {
  const [state, setState] = useState<GenerationState>("idle");
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const textRef = useRef("");

  const generate = useCallback(
    async (
      genreName: string,
      genreDescription: string,
      titleHint: string | null,
    ) => {
      setState("generating");
      setText("");
      setError(null);
      textRef.current = "";

      try {
        await generateStoryText(
          genreName,
          genreDescription,
          titleHint,
          (token) => {
            if (token.done) {
              setState("complete");
            } else {
              textRef.current += token.token;
              setText(textRef.current);
            }
          },
        );
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        setState("error");
      }
    },
    [],
  );

  const generateContinuation = useCallback(
    async (
      genreName: string,
      genreDescription: string,
      previousText: string,
      partNumber: number,
    ) => {
      setState("generating");
      setText("");
      setError(null);
      textRef.current = "";

      try {
        await continueStory(
          genreName,
          genreDescription,
          previousText,
          partNumber,
          (token) => {
            if (token.done) {
              setState("complete");
            } else {
              textRef.current += token.token;
              setText(textRef.current);
            }
          },
        );
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        setState("error");
      }
    },
    [],
  );

  const reset = useCallback(() => {
    setState("idle");
    setText("");
    setError(null);
    textRef.current = "";
  }, []);

  return { state, text, error, generate, generateContinuation, reset, setText };
}
