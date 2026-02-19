import { useState } from "react";
import { BookOpen, PenLine, Settings, Sun, Moon } from "lucide-react";
import { cn } from "@/lib/utils";
import { StoryLibrary } from "@/components/story/StoryLibrary";
import { StoryGenerator } from "@/components/story/StoryGenerator";
import { StoryDetail } from "@/components/story/StoryDetail";
import { VoiceSettings } from "@/components/settings/VoiceSettings";
import { useTheme } from "@/hooks/useTheme";
import pattiAvatar from "@/assets/patti-avatar.jpeg";

type Page = "library" | "create" | "story-detail" | "settings";

interface NavItem {
  page: Page;
  label: string;
  icon: typeof BookOpen;
}

const navItems: NavItem[] = [
  { page: "library", label: "Library", icon: BookOpen },
  { page: "create", label: "Create Story", icon: PenLine },
  { page: "settings", label: "Settings", icon: Settings },
];

export function AppShell() {
  const [page, setPage] = useState<Page>("library");
  const [selectedStoryId, setSelectedStoryId] = useState<string | null>(null);
  const { theme, toggleTheme } = useTheme();

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-background">
      {/* Sidebar */}
      <aside className="flex w-64 flex-col border-r border-sidebar-border bg-sidebar-background">
        {/* App branding */}
        <div className="flex items-center gap-3 px-6 py-6">
          <img
            src={pattiAvatar}
            alt="Patti"
            className="h-11 w-11 rounded-full object-cover ring-2 ring-primary/20"
          />
          <div className="flex flex-col gap-0.5">
            <h1 className="text-2xl font-bold tracking-tight text-primary">
              PattiKadhai
            </h1>
            <span className="text-xs font-medium text-muted-foreground">
              Grandmother's Stories
            </span>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex flex-1 flex-col gap-1 px-3 pt-2">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive =
              page === item.page ||
              (item.page === "library" && page === "story-detail");

            return (
              <button
                key={item.page}
                onClick={() => setPage(item.page)}
                className={cn(
                  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors",
                  isActive
                    ? "bg-sidebar-primary text-sidebar-primary-foreground"
                    : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                )}
              >
                <Icon className="h-4 w-4" />
                {item.label}
              </button>
            );
          })}
        </nav>

        {/* Footer */}
        <div className="flex items-center justify-between px-6 py-4">
          <p className="text-[10px] text-muted-foreground">
            Stories crafted with love
          </p>
          <button
            onClick={toggleTheme}
            className={cn(
              "flex h-7 w-7 items-center justify-center rounded-lg",
              "text-muted-foreground transition-colors",
              "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
            )}
            title={theme === "light" ? "Switch to dark mode" : "Switch to light mode"}
          >
            {theme === "light" ? (
              <Moon className="h-3.5 w-3.5" />
            ) : (
              <Sun className="h-3.5 w-3.5" />
            )}
          </button>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-y-auto">
        {page === "library" && (
          <StoryLibrary
            onSelectStory={(id) => {
              setSelectedStoryId(id);
              setPage("story-detail");
            }}
            onCreateStory={() => setPage("create")}
          />
        )}
        {page === "create" && (
          <StoryGenerator
            onSave={(storyId) => {
              setSelectedStoryId(storyId);
              setPage("story-detail");
            }}
          />
        )}
        {page === "story-detail" && (
          <StoryDetail
            storyId={selectedStoryId!}
            onBack={() => setPage("library")}
          />
        )}
        {page === "settings" && <VoiceSettings />}
      </main>
    </div>
  );
}
