import { Plus, BookOpen } from "lucide-react";
import { cn } from "@/lib/utils";
import { useStories, useGenres } from "@/hooks/useDatabase";

interface StoryLibraryProps {
  onSelectStory: (id: string) => void;
  onCreateStory: () => void;
}

export function StoryLibrary({
  onSelectStory,
  onCreateStory,
}: StoryLibraryProps) {
  const { stories, loading } = useStories();
  const { genres } = useGenres();

  const genreMap = new Map(genres.map((g) => [g.id, g.name]));

  const formatDate = (dateStr: string) => {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
        year: "numeric",
      });
    } catch {
      return dateStr;
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center p-16">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-sm text-muted-foreground">
            Loading your stories...
          </p>
        </div>
      </div>
    );
  }

  // Empty state
  if (stories.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-16">
        <div className="flex max-w-md flex-col items-center gap-6 text-center">
          <div className="flex h-20 w-20 items-center justify-center rounded-full bg-secondary">
            <BookOpen className="h-10 w-10 text-primary" />
          </div>
          <div className="flex flex-col gap-2">
            <h2 className="text-2xl font-bold text-foreground">
              Your story library is empty
            </h2>
            <p className="text-sm text-muted-foreground">
              Create your first magical story and it will appear here. Every
              great adventure starts with a single tale!
            </p>
          </div>
          <button
            onClick={onCreateStory}
            className={cn(
              "flex items-center gap-2 rounded-lg px-6 py-3",
              "bg-primary text-primary-foreground font-medium text-sm",
              "transition-colors hover:bg-primary/90",
              "active:scale-[0.98]",
            )}
          >
            <Plus className="h-4 w-4" />
            Create your first story
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="p-8">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Story Library</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            {stories.length} {stories.length === 1 ? "story" : "stories"} in
            your collection
          </p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-4">
        {/* Create new story card */}
        <button
          onClick={onCreateStory}
          className={cn(
            "flex flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed border-border p-8",
            "text-muted-foreground transition-all duration-200",
            "hover:border-primary/40 hover:bg-accent/30 hover:text-primary",
            "active:scale-[0.98]",
          )}
        >
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-secondary">
            <Plus className="h-6 w-6" />
          </div>
          <span className="text-sm font-medium">Create New Story</span>
        </button>

        {/* Story cards */}
        {stories.map((story) => {
          const genreName = genreMap.get(story.genre_id);

          return (
            <button
              key={story.id}
              onClick={() => onSelectStory(story.id)}
              className={cn(
                "flex flex-col gap-3 rounded-xl border border-border bg-card p-5 text-left",
                "transition-all duration-200",
                "hover:border-primary/40 hover:shadow-md",
                "active:scale-[0.98]",
              )}
            >
              {/* Title */}
              <h3 className="line-clamp-2 text-sm font-semibold text-card-foreground">
                {story.title}
              </h3>

              {/* Badges row */}
              <div className="flex flex-wrap items-center gap-1.5">
                {genreName && (
                  <span
                    className={cn(
                      "inline-flex items-center rounded-full px-2 py-0.5",
                      "bg-secondary text-xs font-medium text-secondary-foreground",
                    )}
                  >
                    {genreName}
                  </span>
                )}
                {story.is_sample === 1 && (
                  <span
                    className={cn(
                      "inline-flex items-center rounded-full px-2 py-0.5",
                      "bg-accent text-xs font-medium text-accent-foreground",
                    )}
                  >
                    Sample
                  </span>
                )}
                <span
                  className={cn(
                    "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium",
                    story.status === "complete"
                      ? "bg-green-100 text-green-800"
                      : "bg-amber-100 text-amber-800",
                  )}
                >
                  {story.status === "complete" ? "Complete" : "Draft"}
                </span>
              </div>

              {/* Date */}
              <span className="mt-auto text-xs text-muted-foreground">
                {formatDate(story.created_at)}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
