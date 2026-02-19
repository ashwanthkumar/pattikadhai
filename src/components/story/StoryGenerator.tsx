import { useState } from "react";
import { ArrowLeft, Wand2, Save } from "lucide-react";
import { cn } from "@/lib/utils";
import { GenreSelector } from "@/components/story/GenreSelector";
import { useStoryGeneration } from "@/hooks/useStoryGeneration";
import { createStory, createStoryPart } from "@/lib/database";
import type { Genre } from "@/types";
import pattiAvatar from "@/assets/patti-avatar.jpeg";

type Step = 1 | 2 | 3 | 4;

interface StoryGeneratorProps {
  onSave: (storyId: string) => void;
}

export function StoryGenerator({ onSave }: StoryGeneratorProps) {
  const [step, setStep] = useState<Step>(1);
  const [selectedGenre, setSelectedGenre] = useState<Genre | null>(null);
  const [titleHint, setTitleHint] = useState("");
  const [editableText, setEditableText] = useState("");
  const [saving, setSaving] = useState(false);

  const { state, text, error, generate, reset, setText } =
    useStoryGeneration();

  const handleGenreSelect = (genre: Genre) => {
    setSelectedGenre(genre);
    setStep(2);
  };

  const handleGenerate = async () => {
    if (!selectedGenre) return;
    setStep(3);
    await generate(
      selectedGenre.name,
      selectedGenre.description,
      titleHint.trim() || null,
    );
  };

  const handleGoToEdit = () => {
    setEditableText(text);
    setStep(4);
  };

  const handleSave = async () => {
    if (!selectedGenre) return;
    setSaving(true);

    try {
      const storyId = crypto.randomUUID();
      const partId = crypto.randomUUID();
      const title =
        titleHint.trim() || `${selectedGenre.name} Story`;

      await createStory(storyId, title, selectedGenre.id);
      await createStoryPart(partId, storyId, 1, editableText);
      onSave(storyId);
    } catch (e) {
      console.error("Failed to save story:", e);
      setSaving(false);
    }
  };

  const handleBack = () => {
    if (step === 2) {
      setSelectedGenre(null);
      setStep(1);
    } else if (step === 3) {
      reset();
      setStep(2);
    } else if (step === 4) {
      setText(editableText);
      setStep(3);
    }
  };

  const stepLabels = ["Genre", "Title", "Generate", "Edit & Save"];

  return (
    <div className="mx-auto max-w-3xl p-8">
      {/* Header with back button and step indicator */}
      <div className="mb-8 flex flex-col gap-4">
        <div className="flex items-center gap-3">
          {step > 1 && (
            <button
              onClick={handleBack}
              className={cn(
                "flex h-8 w-8 items-center justify-center rounded-lg",
                "text-muted-foreground transition-colors",
                "hover:bg-secondary hover:text-foreground",
              )}
            >
              <ArrowLeft className="h-4 w-4" />
            </button>
          )}
          <h1 className="text-2xl font-bold text-foreground">
            Create a New Story
          </h1>
        </div>

        {/* Step indicator */}
        <div className="flex items-center gap-2">
          {stepLabels.map((label, i) => (
            <div key={label} className="flex items-center gap-2">
              <div
                className={cn(
                  "flex h-7 items-center gap-1.5 rounded-full px-3 text-xs font-medium transition-colors",
                  i + 1 === step
                    ? "bg-primary text-primary-foreground"
                    : i + 1 < step
                      ? "bg-secondary text-secondary-foreground"
                      : "bg-muted text-muted-foreground",
                )}
              >
                <span>{i + 1}</span>
                <span className="hidden sm:inline">{label}</span>
              </div>
              {i < stepLabels.length - 1 && (
                <div
                  className={cn(
                    "h-px w-6",
                    i + 1 < step ? "bg-primary/40" : "bg-border",
                  )}
                />
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Step content */}
      {step === 1 && <GenreSelector onSelect={handleGenreSelect} />}

      {step === 2 && (
        <div className="flex flex-col gap-6">
          <div>
            <h2 className="text-xl font-bold text-foreground">
              Story Title Hint
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Give a hint for the story title, or leave it blank and let the AI
              surprise you.
            </p>
          </div>

          {selectedGenre && (
            <div className="flex items-center gap-2 rounded-lg bg-secondary px-3 py-2">
              <span className="text-xs font-medium text-muted-foreground">
                Genre:
              </span>
              <span className="text-sm font-semibold text-secondary-foreground">
                {selectedGenre.name}
              </span>
            </div>
          )}

          <div className="flex flex-col gap-3">
            <input
              type="text"
              value={titleHint}
              onChange={(e) => setTitleHint(e.target.value)}
              placeholder="e.g. The Brave Little Sparrow..."
              className={cn(
                "w-full rounded-lg border border-input bg-card px-4 py-3 text-sm",
                "text-foreground placeholder:text-muted-foreground",
                "focus:border-primary focus:outline-none focus:ring-2 focus:ring-ring/20",
              )}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleGenerate();
              }}
            />
            <button
              onClick={handleGenerate}
              className={cn(
                "flex items-center justify-center gap-2 self-start rounded-lg px-6 py-3",
                "bg-primary text-primary-foreground font-medium text-sm",
                "transition-colors hover:bg-primary/90",
                "active:scale-[0.98]",
              )}
            >
              <Wand2 className="h-4 w-4" />
              Generate Story
            </button>
          </div>
        </div>
      )}

      {step === 3 && (
        <div className="flex flex-col gap-6">
          <div className="flex items-center gap-4">
            <img
              src={pattiAvatar}
              alt="Patti"
              className={cn(
                "h-14 w-14 rounded-full object-cover ring-2 ring-primary/20",
                state === "generating" && "animate-pulse",
              )}
            />
            <div>
              <h2 className="text-xl font-bold text-foreground">
                {state === "generating"
                  ? "Patti is telling a story..."
                  : state === "complete"
                    ? "Patti's story is ready!"
                    : "Generation"}
              </h2>
              {state === "generating" && (
                <p className="mt-1 text-sm text-muted-foreground">
                  Sit back and enjoy the tale!
                </p>
              )}
            </div>
          </div>

          {error && (
            <div className="rounded-lg border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive">
              {error}
            </div>
          )}

          <div
            className={cn(
              "min-h-[200px] rounded-xl border border-border bg-card p-6",
              "text-sm leading-relaxed text-card-foreground",
              "whitespace-pre-wrap",
            )}
          >
            {text}
            {state === "generating" && (
              <span className="inline-block h-4 w-0.5 animate-pulse bg-primary ml-0.5" />
            )}
            {!text && state !== "generating" && (
              <span className="text-muted-foreground italic">
                Your story will appear here...
              </span>
            )}
          </div>

          {state === "complete" && (
            <button
              onClick={handleGoToEdit}
              className={cn(
                "flex items-center justify-center gap-2 self-start rounded-lg px-6 py-3",
                "bg-primary text-primary-foreground font-medium text-sm",
                "transition-colors hover:bg-primary/90",
                "active:scale-[0.98]",
              )}
            >
              Review & Save
            </button>
          )}
        </div>
      )}

      {step === 4 && (
        <div className="flex flex-col gap-6">
          <div>
            <h2 className="text-xl font-bold text-foreground">
              Review & Save
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Make any edits you like, then save to your library.
            </p>
          </div>

          <textarea
            value={editableText}
            onChange={(e) => setEditableText(e.target.value)}
            rows={16}
            className={cn(
              "w-full rounded-xl border border-input bg-card px-4 py-3 text-sm",
              "leading-relaxed text-foreground",
              "resize-y",
              "focus:border-primary focus:outline-none focus:ring-2 focus:ring-ring/20",
            )}
          />

          <button
            onClick={handleSave}
            disabled={saving || !editableText.trim()}
            className={cn(
              "flex items-center justify-center gap-2 self-start rounded-lg px-6 py-3",
              "bg-primary text-primary-foreground font-medium text-sm",
              "transition-colors hover:bg-primary/90",
              "active:scale-[0.98]",
              "disabled:cursor-not-allowed disabled:opacity-50",
            )}
          >
            <Save className="h-4 w-4" />
            {saving ? "Saving..." : "Save to Library"}
          </button>
        </div>
      )}
    </div>
  );
}
