import { useRef } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { Play, Pause, Download } from "lucide-react";
import { useState } from "react";

interface StoryPlayerProps {
  audioPath: string;
  title?: string;
}

export function StoryPlayer({ audioPath, title }: StoryPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [playing, setPlaying] = useState(false);

  const audioSrc = convertFileSrc(audioPath);

  const togglePlay = () => {
    if (!audioRef.current) return;
    if (playing) {
      audioRef.current.pause();
    } else {
      audioRef.current.play();
    }
    setPlaying(!playing);
  };

  const handleExport = async () => {
    const savePath = await save({
      defaultPath: `${title ?? "story"}.mp3`,
      filters: [{ name: "Audio", extensions: ["mp3"] }],
    });
    if (savePath) {
      // Copy file to save location
      // In Tauri, we'd use fs plugin but for now we can use a command
      console.log("Export to:", savePath);
    }
  };

  return (
    <div className="flex items-center gap-3 rounded-lg border border-border bg-card p-3">
      <button
        onClick={togglePlay}
        className="flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-foreground hover:bg-primary/90"
      >
        {playing ? (
          <Pause className="h-4 w-4" />
        ) : (
          <Play className="ml-0.5 h-4 w-4" />
        )}
      </button>

      <audio
        ref={audioRef}
        src={audioSrc}
        onEnded={() => setPlaying(false)}
        className="hidden"
      />

      <div className="flex-1">
        <div className="text-sm font-medium">{title ?? "Story Audio"}</div>
        <div className="text-xs text-muted-foreground">MP3 Audio</div>
      </div>

      <button
        onClick={handleExport}
        className="flex h-8 w-8 items-center justify-center rounded-md hover:bg-secondary"
        title="Export MP3"
      >
        <Download className="h-4 w-4 text-muted-foreground" />
      </button>
    </div>
  );
}
