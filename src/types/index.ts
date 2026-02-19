export interface Genre {
  id: string;
  name: string;
  description: string;
  icon: string | null;
  display_order: number;
  created_at: string;
}

export interface Story {
  id: string;
  title: string;
  genre_id: string;
  status: "draft" | "complete";
  is_sample: number;
  created_at: string;
  updated_at: string;
}

export interface StoryPart {
  id: string;
  story_id: string;
  part_number: number;
  content: string;
  audio_path: string | null;
  status:
    | "draft"
    | "text_ready"
    | "audio_processing"
    | "audio_ready"
    | "audio_failed";
  created_at: string;
  updated_at: string;
}

export interface AudioJob {
  id: string;
  story_part_id: string;
  voice_path: string | null;
  final_path: string | null;
  status:
    | "pending"
    | "voice_generating"
    | "complete"
    | "failed";
  error_message: string | null;
  created_at: string;
  updated_at: string;
}

export interface StoryToken {
  token: string;
  done: boolean;
}

export interface DependencyStatus {
  name: string;
  installed: boolean;
  version: string | null;
  install_command: string;
}

export interface PipelineProgress {
  job_id: string;
  stage: string;
  progress: number;
  error: string | null;
}

export type Page =
  | "library"
  | "create"
  | "story-detail"
  | "story-editor"
  | "audio-generator";
