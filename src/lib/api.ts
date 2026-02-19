import { invoke, Channel } from "@tauri-apps/api/core";
import type { DependencyStatus, StoryToken } from "@/types";

export async function checkDependency(
  name: string,
): Promise<DependencyStatus> {
  return invoke<DependencyStatus>("check_dependency", { name });
}

export async function installDependency(
  name: string,
): Promise<{ success: boolean; output: string }> {
  return invoke("install_dependency", { name });
}

export async function generateStoryText(
  genreName: string,
  genreDescription: string,
  titleHint: string | null,
  onToken: (token: StoryToken) => void,
): Promise<string> {
  const channel = new Channel<StoryToken>();
  channel.onmessage = onToken;

  return invoke<string>("generate_story_text", {
    genreName,
    genreDescription,
    titleHint,
    onToken: channel,
  });
}

export async function continueStory(
  genreName: string,
  genreDescription: string,
  previousText: string,
  partNumber: number,
  onToken: (token: StoryToken) => void,
): Promise<string> {
  const channel = new Channel<StoryToken>();
  channel.onmessage = onToken;

  return invoke<string>("continue_story", {
    genreName,
    genreDescription,
    previousText,
    partNumber,
    onToken: channel,
  });
}

export async function startAudioGeneration(
  jobId: string,
  partId: string,
  text: string,
): Promise<{ job_id: string; status: string }> {
  return invoke("start_audio_generation", {
    jobId,
    partId,
    text,
  });
}

export async function applyMigrations(): Promise<string> {
  return invoke<string>("apply_migrations");
}

export async function getAudioJobStatus(
  jobId: string,
): Promise<{
  id: string;
  status: string;
  error_message: string | null;
  final_path: string | null;
}> {
  return invoke("get_audio_job_status", { jobId });
}
