import { useState, useEffect, useCallback } from "react";
import type { Genre, Story, StoryPart } from "@/types";
import * as db from "@/lib/database";

export function useGenres() {
  const [genres, setGenres] = useState<Genre[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    db.getGenres()
      .then(setGenres)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  return { genres, loading };
}

export function useStories() {
  const [stories, setStories] = useState<Story[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(() => {
    setLoading(true);
    db.getStories()
      .then(setStories)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { stories, loading, refresh };
}

export function useStoryParts(storyId: string | null) {
  const [parts, setParts] = useState<StoryPart[]>([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(() => {
    if (!storyId) return;
    setLoading(true);
    db.getStoryParts(storyId)
      .then(setParts)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [storyId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { parts, loading, refresh };
}

export function useStory(storyId: string | null) {
  const [story, setStory] = useState<Story | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(() => {
    if (!storyId) return;
    setLoading(true);
    db.getStory(storyId)
      .then(setStory)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [storyId]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { story, loading, refresh };
}
