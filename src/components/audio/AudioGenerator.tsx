import { useAudioGeneration } from "@/hooks/useAudioGeneration";
import { AudioProgress } from "./AudioProgress";
import { StoryPlayer } from "./StoryPlayer";
import { Music } from "lucide-react";

interface AudioGeneratorProps {
  partId: string;
  text: string;
  genre: string;
  audioPath: string | null;
  title?: string;
}

export function AudioGenerator({
  partId,
  text,
  genre,
  audioPath,
  title,
}: AudioGeneratorProps) {
  const { stage, error, startGeneration } = useAudioGeneration();

  if (audioPath) {
    return <StoryPlayer audioPath={audioPath} title={title} />;
  }

  if (stage !== "idle") {
    return (
      <div className="rounded-lg border border-border bg-card p-4">
        <h4 className="mb-3 text-sm font-medium">Generating Audio</h4>
        <AudioProgress currentStage={stage} error={error} />
      </div>
    );
  }

  return (
    <button
      onClick={() => startGeneration(partId, text, genre)}
      className="flex items-center gap-2 rounded-lg border border-dashed border-border px-4 py-3 text-sm text-muted-foreground hover:border-primary hover:text-primary"
    >
      <Music className="h-4 w-4" />
      Generate Audio
    </button>
  );
}
