import { cn } from "@/lib/utils";
import { AUDIO_STAGES } from "@/lib/constants";
import { CheckCircle2, Loader2, Circle } from "lucide-react";

interface AudioProgressProps {
  currentStage: string;
  error: string | null;
}

export function AudioProgress({ currentStage, error }: AudioProgressProps) {
  if (error) {
    return (
      <div className="rounded-lg border border-red-200 bg-red-50 p-4">
        <p className="text-sm font-medium text-red-800">
          Audio generation failed
        </p>
        <p className="mt-1 text-sm text-red-600">{error}</p>
      </div>
    );
  }

  const currentIndex = AUDIO_STAGES.findIndex((s) => s.key === currentStage);

  return (
    <div className="space-y-3">
      {AUDIO_STAGES.map((stage, index) => {
        const isComplete = index < currentIndex;
        const isCurrent = index === currentIndex;

        return (
          <div key={stage.key} className="flex items-center gap-3">
            {isComplete ? (
              <CheckCircle2 className="h-5 w-5 text-green-600" />
            ) : isCurrent ? (
              <Loader2 className="h-5 w-5 animate-spin text-primary" />
            ) : (
              <Circle className="h-5 w-5 text-muted-foreground/40" />
            )}
            <span
              className={cn(
                "text-sm",
                isComplete && "text-green-700",
                isCurrent && "font-medium text-foreground",
                !isComplete && !isCurrent && "text-muted-foreground/60",
              )}
            >
              {stage.label}
            </span>
          </div>
        );
      })}
    </div>
  );
}
