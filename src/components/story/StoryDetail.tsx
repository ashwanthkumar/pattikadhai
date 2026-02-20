import { useState, useEffect } from "react";
import {
  ArrowLeft,
  ChevronDown,
  ChevronRight,
  Pencil,
  Volume2,
  Plus,
  Check,
  X,
  Loader2,
} from "lucide-react";
import { getAudioUrl } from "@/lib/audio";
import { cn } from "@/lib/utils";
import { useStory, useStoryParts, useGenres } from "@/hooks/useDatabase";
import { useStoryGeneration } from "@/hooks/useStoryGeneration";
import { useAudioGeneration } from "@/hooks/useAudioGeneration";
import {
  updateStoryPartContent,
  createStoryPart,
} from "@/lib/database";
import { AUDIO_STAGES } from "@/lib/constants";
import type { StoryPart } from "@/types";

interface StoryDetailProps {
  storyId: string;
  onBack: () => void;
}

export function StoryDetail({ storyId, onBack }: StoryDetailProps) {
  const { story, loading: storyLoading } = useStory(storyId);
  const { parts, loading: partsLoading, refresh: refreshParts } = useStoryParts(storyId);
  const { genres } = useGenres();

  const [expandedParts, setExpandedParts] = useState<Set<string>>(new Set());

  // Auto-expand all parts once loaded
  useEffect(() => {
    if (parts.length > 0) {
      setExpandedParts(new Set(parts.map((p) => p.id)));
    }
  }, [parts]);

  // Re-fetch parts when audio generation completes so audio_path is available
  useEffect(() => {
    if (audioStage === "complete") {
      refreshParts();
    }
  }, [audioStage, refreshParts]);
  const [editingPartId, setEditingPartId] = useState<string | null>(null);
  const [editText, setEditText] = useState("");
  const [savingEdit, setSavingEdit] = useState(false);

  // Audio generation state
  const {
    stage: audioStage,
    progress: audioProgress,
    error: audioError,
    startGeneration,
    reset: _resetAudio,
  } = useAudioGeneration();
  const [audioPartId, setAudioPartId] = useState<string | null>(null);

  // Continuation generation state
  const {
    state: contState,
    text: contText,
    error: contError,
    generateContinuation,
    reset: resetCont,
  } = useStoryGeneration();
  const [showContinuation, setShowContinuation] = useState(false);

  const genreMap = new Map(genres.map((g) => [g.id, g]));
  const genre = story ? genreMap.get(story.genre_id) : null;

  const toggleExpand = (partId: string) => {
    setExpandedParts((prev) => {
      const next = new Set(prev);
      if (next.has(partId)) {
        next.delete(partId);
      } else {
        next.add(partId);
      }
      return next;
    });
  };

  const startEditing = (part: StoryPart) => {
    setEditingPartId(part.id);
    setEditText(part.content);
    // Ensure the part is expanded
    setExpandedParts((prev) => new Set(prev).add(part.id));
  };

  const cancelEditing = () => {
    setEditingPartId(null);
    setEditText("");
  };

  const saveEdit = async () => {
    if (!editingPartId) return;
    setSavingEdit(true);
    try {
      await updateStoryPartContent(editingPartId, editText);
      await refreshParts();
      setEditingPartId(null);
      setEditText("");
    } catch (e) {
      console.error("Failed to save edit:", e);
    } finally {
      setSavingEdit(false);
    }
  };

  const handleGenerateAudio = async (part: StoryPart) => {
    if (!genre) return;
    setAudioPartId(part.id);
    await startGeneration(part.id, part.content);
  };

  const handleAddContinuation = async () => {
    if (!genre || parts.length === 0) return;
    setShowContinuation(true);
    const lastPart = parts[parts.length - 1];
    const nextPartNumber = lastPart.part_number + 1;
    await generateContinuation(
      genre.name,
      genre.description,
      lastPart.content,
      nextPartNumber,
    );
  };

  const handleSaveContinuation = async () => {
    if (!contText.trim()) return;
    const lastPart = parts[parts.length - 1];
    const nextPartNumber = lastPart ? lastPart.part_number + 1 : 1;
    const partId = crypto.randomUUID();

    try {
      await createStoryPart(partId, storyId, nextPartNumber, contText);
      await refreshParts();
      resetCont();
      setShowContinuation(false);
    } catch (e) {
      console.error("Failed to save continuation:", e);
    }
  };

  if (storyLoading || partsLoading) {
    return (
      <div className="flex items-center justify-center p-16">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">Loading story...</p>
        </div>
      </div>
    );
  }

  if (!story) {
    return (
      <div className="flex items-center justify-center p-16">
        <p className="text-sm text-muted-foreground">Story not found.</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-3xl p-8">
      {/* Header */}
      <div className="mb-8 flex flex-col gap-3">
        <div className="flex items-center gap-3">
          <button
            onClick={onBack}
            className={cn(
              "flex h-8 w-8 items-center justify-center rounded-lg",
              "text-muted-foreground transition-colors",
              "hover:bg-secondary hover:text-foreground",
            )}
          >
            <ArrowLeft className="h-4 w-4" />
          </button>
          <h1 className="text-2xl font-bold text-foreground">{story.title}</h1>
        </div>

        <div className="flex items-center gap-2 pl-11">
          {genre && (
            <span
              className={cn(
                "inline-flex items-center rounded-full px-2.5 py-0.5",
                "bg-secondary text-xs font-medium text-secondary-foreground",
              )}
            >
              {genre.name}
            </span>
          )}
          <span
            className={cn(
              "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium",
              story.status === "complete"
                ? "bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300"
                : "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300",
            )}
          >
            {story.status === "complete" ? "Complete" : "Draft"}
          </span>
        </div>
      </div>

      {/* Story parts */}
      <div className="flex flex-col gap-4">
        {parts.map((part) => {
          const isExpanded = expandedParts.has(part.id);
          const isEditing = editingPartId === part.id;
          const isAudioGenerating =
            audioPartId === part.id && audioStage !== "idle" && audioStage !== "complete" && audioStage !== "failed";

          return (
            <div
              key={part.id}
              className="rounded-xl border border-border bg-card overflow-hidden"
            >
              {/* Part header */}
              <button
                onClick={() => toggleExpand(part.id)}
                className={cn(
                  "flex w-full items-center gap-3 px-5 py-4 text-left",
                  "transition-colors hover:bg-secondary/50",
                )}
              >
                {isExpanded ? (
                  <ChevronDown className="h-4 w-4 text-muted-foreground" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-muted-foreground" />
                )}
                <span className="text-sm font-semibold text-card-foreground">
                  Part {part.part_number}
                </span>

                {/* Status badge */}
                <span
                  className={cn(
                    "ml-auto inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium",
                    part.status === "audio_ready"
                      ? "bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300"
                      : part.status === "text_ready"
                        ? "bg-blue-100 text-blue-800 dark:bg-blue-900/40 dark:text-blue-300"
                        : part.status === "audio_processing"
                          ? "bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300"
                          : part.status === "audio_failed"
                            ? "bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300"
                            : "bg-muted text-muted-foreground",
                  )}
                >
                  {part.status.replace("_", " ")}
                </span>

                {/* Truncated preview when collapsed */}
                {!isExpanded && (
                  <span className="ml-3 max-w-[200px] truncate text-xs text-muted-foreground">
                    {part.content.slice(0, 80)}...
                  </span>
                )}
              </button>

              {/* Expanded content */}
              {isExpanded && (
                <div className="border-t border-border px-5 py-4">
                  {isEditing ? (
                    <div className="flex flex-col gap-3">
                      <textarea
                        value={editText}
                        onChange={(e) => setEditText(e.target.value)}
                        rows={12}
                        className={cn(
                          "w-full rounded-lg border border-input bg-background px-3 py-2 text-sm",
                          "leading-relaxed text-foreground resize-y",
                          "focus:border-primary focus:outline-none focus:ring-2 focus:ring-ring/20",
                        )}
                      />
                      <div className="flex items-center gap-2">
                        <button
                          onClick={saveEdit}
                          disabled={savingEdit}
                          className={cn(
                            "flex items-center gap-1.5 rounded-lg px-4 py-2",
                            "bg-primary text-primary-foreground text-xs font-medium",
                            "transition-colors hover:bg-primary/90",
                            "disabled:cursor-not-allowed disabled:opacity-50",
                          )}
                        >
                          <Check className="h-3 w-3" />
                          {savingEdit ? "Saving..." : "Save"}
                        </button>
                        <button
                          onClick={cancelEditing}
                          className={cn(
                            "flex items-center gap-1.5 rounded-lg px-4 py-2",
                            "bg-secondary text-secondary-foreground text-xs font-medium",
                            "transition-colors hover:bg-secondary/80",
                          )}
                        >
                          <X className="h-3 w-3" />
                          Cancel
                        </button>
                      </div>
                    </div>
                  ) : (
                    <div className="flex flex-col gap-4">
                      {/* Text content */}
                      <p className="whitespace-pre-wrap text-sm leading-relaxed text-card-foreground">
                        {part.content}
                      </p>

                      {/* Audio player */}
                      {part.audio_path && (
                        <AudioPlayer path={part.audio_path} />
                      )}

                      {/* Audio generation progress */}
                      {isAudioGenerating && (
                        <div className="flex flex-col gap-2 rounded-lg bg-secondary/50 p-3">
                          <div className="flex items-center gap-2">
                            <Loader2 className="h-3.5 w-3.5 animate-spin text-primary" />
                            <span className="text-xs font-medium text-foreground">
                              Generating audio...
                            </span>
                          </div>
                          <div className="flex items-center gap-2">
                            {AUDIO_STAGES.map((s) => (
                              <span
                                key={s.key}
                                className={cn(
                                  "text-[10px] font-medium px-2 py-0.5 rounded-full",
                                  audioStage === s.key
                                    ? "bg-primary text-primary-foreground"
                                    : "text-muted-foreground",
                                )}
                              >
                                {s.label}
                              </span>
                            ))}
                          </div>
                          {audioProgress > 0 && (
                            <div className="h-1.5 w-full overflow-hidden rounded-full bg-muted">
                              <div
                                className="h-full rounded-full bg-primary transition-all duration-300"
                                style={{ width: `${audioProgress}%` }}
                              />
                            </div>
                          )}
                        </div>
                      )}

                      {/* Audio error */}
                      {audioPartId === part.id && audioError && (
                        <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                          Audio generation failed: {audioError}
                        </div>
                      )}

                      {/* Action buttons */}
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => startEditing(part)}
                          className={cn(
                            "flex items-center gap-1.5 rounded-lg px-3 py-1.5",
                            "bg-secondary text-secondary-foreground text-xs font-medium",
                            "transition-colors hover:bg-secondary/80",
                          )}
                        >
                          <Pencil className="h-3 w-3" />
                          Edit
                        </button>

                        {(part.status === "text_ready" || part.status === "audio_failed") && !isAudioGenerating && (
                          <button
                            onClick={() => handleGenerateAudio(part)}
                            className={cn(
                              "flex items-center gap-1.5 rounded-lg px-3 py-1.5",
                              "bg-primary text-primary-foreground text-xs font-medium",
                              "transition-colors hover:bg-primary/90",
                            )}
                          >
                            <Volume2 className="h-3 w-3" />
                            Generate Audio
                          </button>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          );
        })}

        {/* Continuation section */}
        {showContinuation && (
          <div className="rounded-xl border border-border bg-card p-5">
            <div className="flex flex-col gap-4">
              <h3 className="text-sm font-semibold text-card-foreground">
                Part {parts.length + 1} - Continuation
              </h3>

              {contError && (
                <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
                  {contError}
                </div>
              )}

              <div
                className={cn(
                  "min-h-[120px] rounded-lg border border-border bg-background p-4",
                  "text-sm leading-relaxed text-foreground whitespace-pre-wrap",
                )}
              >
                {contText}
                {contState === "generating" && (
                  <span className="inline-block h-4 w-0.5 animate-pulse bg-primary ml-0.5" />
                )}
                {!contText && contState !== "generating" && (
                  <span className="text-muted-foreground italic">
                    Continuation will appear here...
                  </span>
                )}
              </div>

              {contState === "complete" && (
                <div className="flex items-center gap-2">
                  <button
                    onClick={handleSaveContinuation}
                    className={cn(
                      "flex items-center gap-1.5 rounded-lg px-4 py-2",
                      "bg-primary text-primary-foreground text-xs font-medium",
                      "transition-colors hover:bg-primary/90",
                    )}
                  >
                    <Check className="h-3 w-3" />
                    Save Part
                  </button>
                  <button
                    onClick={() => {
                      resetCont();
                      setShowContinuation(false);
                    }}
                    className={cn(
                      "flex items-center gap-1.5 rounded-lg px-4 py-2",
                      "bg-secondary text-secondary-foreground text-xs font-medium",
                      "transition-colors hover:bg-secondary/80",
                    )}
                  >
                    <X className="h-3 w-3" />
                    Discard
                  </button>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Add continuation button */}
        {parts.length > 0 && !showContinuation && (
          <button
            onClick={handleAddContinuation}
            disabled={contState === "generating"}
            className={cn(
              "flex items-center justify-center gap-2 rounded-xl border-2 border-dashed border-border p-4",
              "text-sm font-medium text-muted-foreground",
              "transition-all duration-200",
              "hover:border-primary/40 hover:bg-accent/30 hover:text-primary",
              "active:scale-[0.99]",
              "disabled:cursor-not-allowed disabled:opacity-50",
            )}
          >
            <Plus className="h-4 w-4" />
            Add Part {parts.length + 1}
          </button>
        )}
      </div>
    </div>
  );
}

function AudioPlayer({ path }: { path: string }) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    getAudioUrl(path).then(setSrc);
  }, [path]);

  if (!src) return null;

  return (
    <div className="flex flex-col gap-1">
      <span className="text-xs font-medium text-muted-foreground">Audio</span>
      <audio controls src={src} className="w-full" />
    </div>
  );
}
