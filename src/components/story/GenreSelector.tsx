import {
  Compass,
  Sparkles,
  Heart,
  Moon,
  PawPrint,
  FlaskConical,
  type LucideIcon,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useGenres } from "@/hooks/useDatabase";
import type { Genre } from "@/types";

const iconMap: Record<string, LucideIcon> = {
  compass: Compass,
  sparkles: Sparkles,
  heart: Heart,
  moon: Moon,
  "paw-print": PawPrint,
  flask: FlaskConical,
};

interface GenreSelectorProps {
  onSelect: (genre: Genre) => void;
}

export function GenreSelector({ onSelect }: GenreSelectorProps) {
  const { genres, loading } = useGenres();

  if (loading) {
    return (
      <div className="flex items-center justify-center py-16">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">Loading genres...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-xl font-bold text-foreground">
          Choose a Genre
        </h2>
        <p className="mt-1 text-sm text-muted-foreground">
          What kind of story shall we tell today?
        </p>
      </div>

      <div className="grid grid-cols-3 gap-4">
        {genres.map((genre) => {
          const Icon = genre.icon ? iconMap[genre.icon] : null;

          return (
            <button
              key={genre.id}
              onClick={() => onSelect(genre)}
              className={cn(
                "group flex flex-col items-center gap-3 rounded-xl border border-border bg-card p-6",
                "text-center transition-all duration-200",
                "hover:border-primary/40 hover:bg-accent/50 hover:shadow-md",
                "active:scale-[0.98]",
              )}
            >
              {Icon && (
                <div
                  className={cn(
                    "flex h-12 w-12 items-center justify-center rounded-full",
                    "bg-secondary text-primary transition-colors",
                    "group-hover:bg-primary group-hover:text-primary-foreground",
                  )}
                >
                  <Icon className="h-6 w-6" />
                </div>
              )}
              <div className="flex flex-col gap-1">
                <span className="text-sm font-semibold text-card-foreground">
                  {genre.name}
                </span>
                {genre.description && (
                  <span className="text-xs leading-relaxed text-muted-foreground">
                    {genre.description}
                  </span>
                )}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
