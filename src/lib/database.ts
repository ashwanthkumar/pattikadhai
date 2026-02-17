import Database from "@tauri-apps/plugin-sql";
import type { Genre, Story, StoryPart, AudioJob } from "@/types";

let db: Database | null = null;

async function getDb(): Promise<Database> {
  if (!db) {
    db = await Database.load("sqlite:pattikadhai.db");
  }
  return db;
}

// Genre queries
export async function getGenres(): Promise<Genre[]> {
  const conn = await getDb();
  return conn.select<Genre[]>(
    "SELECT * FROM genres ORDER BY display_order",
  );
}

export async function getGenre(id: string): Promise<Genre | null> {
  const conn = await getDb();
  const rows = await conn.select<Genre[]>(
    "SELECT * FROM genres WHERE id = $1",
    [id],
  );
  return rows[0] ?? null;
}

// Story queries
export async function getStories(): Promise<Story[]> {
  const conn = await getDb();
  return conn.select<Story[]>(
    "SELECT * FROM stories ORDER BY updated_at DESC",
  );
}

export async function getStory(id: string): Promise<Story | null> {
  const conn = await getDb();
  const rows = await conn.select<Story[]>(
    "SELECT * FROM stories WHERE id = $1",
    [id],
  );
  return rows[0] ?? null;
}

export async function createStory(
  id: string,
  title: string,
  genreId: string,
): Promise<void> {
  const conn = await getDb();
  await conn.execute(
    "INSERT INTO stories (id, title, genre_id) VALUES ($1, $2, $3)",
    [id, title, genreId],
  );
}

export async function updateStoryTitle(
  id: string,
  title: string,
): Promise<void> {
  const conn = await getDb();
  await conn.execute(
    "UPDATE stories SET title = $1, updated_at = datetime('now') WHERE id = $2",
    [title, id],
  );
}

export async function updateStoryStatus(
  id: string,
  status: string,
): Promise<void> {
  const conn = await getDb();
  await conn.execute(
    "UPDATE stories SET status = $1, updated_at = datetime('now') WHERE id = $2",
    [status, id],
  );
}

export async function deleteStory(id: string): Promise<void> {
  const conn = await getDb();
  await conn.execute("DELETE FROM stories WHERE id = $1", [id]);
}

// StoryPart queries
export async function getStoryParts(storyId: string): Promise<StoryPart[]> {
  const conn = await getDb();
  return conn.select<StoryPart[]>(
    "SELECT * FROM story_parts WHERE story_id = $1 ORDER BY part_number",
    [storyId],
  );
}

export async function createStoryPart(
  id: string,
  storyId: string,
  partNumber: number,
  content: string,
): Promise<void> {
  const conn = await getDb();
  await conn.execute(
    "INSERT INTO story_parts (id, story_id, part_number, content, status) VALUES ($1, $2, $3, $4, 'text_ready')",
    [id, storyId, partNumber, content],
  );
}

export async function updateStoryPartContent(
  id: string,
  content: string,
): Promise<void> {
  const conn = await getDb();
  await conn.execute(
    "UPDATE story_parts SET content = $1, updated_at = datetime('now') WHERE id = $2",
    [content, id],
  );
}

// AudioJob queries
export async function createAudioJob(
  id: string,
  storyPartId: string,
): Promise<void> {
  const conn = await getDb();
  await conn.execute(
    "INSERT INTO audio_jobs (id, story_part_id) VALUES ($1, $2)",
    [id, storyPartId],
  );
}

export async function getAudioJob(id: string): Promise<AudioJob | null> {
  const conn = await getDb();
  const rows = await conn.select<AudioJob[]>(
    "SELECT * FROM audio_jobs WHERE id = $1",
    [id],
  );
  return rows[0] ?? null;
}

export async function getAudioJobForPart(
  partId: string,
): Promise<AudioJob | null> {
  const conn = await getDb();
  const rows = await conn.select<AudioJob[]>(
    "SELECT * FROM audio_jobs WHERE story_part_id = $1 ORDER BY created_at DESC LIMIT 1",
    [partId],
  );
  return rows[0] ?? null;
}

// Settings queries
export interface VoiceSettingsData {
  tts_voice: string;
  tts_seed: string;
  tts_temperature: string;
}

export async function getVoiceSettings(): Promise<VoiceSettingsData> {
  const conn = await getDb();
  const rows = await conn.select<{ key: string; value: string }[]>(
    "SELECT key, value FROM app_settings WHERE key LIKE 'tts_%'",
  );
  const settings: VoiceSettingsData = {
    tts_voice: "Vivian",
    tts_seed: "42",
    tts_temperature: "0.3",
  };
  for (const row of rows) {
    if (row.key in settings) {
      settings[row.key as keyof VoiceSettingsData] = row.value;
    }
  }
  return settings;
}

export async function updateSetting(
  key: string,
  value: string,
): Promise<void> {
  const conn = await getDb();
  await conn.execute(
    "UPDATE app_settings SET value = $1, updated_at = datetime('now') WHERE key = $2",
    [value, key],
  );
}
