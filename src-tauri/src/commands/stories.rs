use crate::db::models::{Genre, Story, StoryPart};
use crate::services::ollama::{OllamaClient, StoryToken};
use crate::services::prompts;
use rusqlite::Connection;
use tauri::ipc::Channel;
use tauri::Manager;

#[tauri::command]
pub async fn generate_story_text(
    genre_name: String,
    genre_description: String,
    title_hint: Option<String>,
    on_token: Channel<StoryToken>,
) -> Result<String, String> {
    let client = OllamaClient::new();
    let system = prompts::build_system_prompt(&genre_name, &genre_description);
    let prompt = prompts::build_story_prompt(title_hint.as_deref());

    client
        .generate_streaming("gemma3", &system, &prompt, &on_token)
        .await
}

#[tauri::command]
pub async fn continue_story(
    genre_name: String,
    genre_description: String,
    previous_text: String,
    part_number: i32,
    on_token: Channel<StoryToken>,
) -> Result<String, String> {
    let client = OllamaClient::new();

    // First, summarize the previous text
    let summary_prompt = prompts::build_summary_prompt(&previous_text);
    let summary = client
        .generate(
            "gemma3",
            "You are a helpful assistant. Summarize concisely.",
            &summary_prompt,
        )
        .await?;

    // Then generate continuation
    let system = prompts::build_system_prompt(&genre_name, &genre_description);
    let prompt = prompts::build_continuation_prompt(&summary, part_number);

    client
        .generate_streaming("gemma3", &system, &prompt, &on_token)
        .await
}

#[derive(serde::Serialize)]
pub struct StoryWithParts {
    pub story: Story,
    pub genre: Genre,
    pub parts: Vec<StoryPart>,
}

#[tauri::command]
pub async fn get_story_detail(
    story_id: String,
    app: tauri::AppHandle,
) -> Result<StoryWithParts, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let db_path = app_data_dir.join("pattikadhai.db");

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open DB: {}", e))?;

    let story = conn
        .query_row(
            "SELECT id, title, genre_id, status, is_sample, created_at, updated_at FROM stories WHERE id = ?1",
            [&story_id],
            |row| {
                Ok(Story {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    genre_id: row.get(2)?,
                    status: row.get(3)?,
                    is_sample: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            },
        )
        .map_err(|e| format!("Story not found: {}", e))?;

    let genre = conn
        .query_row(
            "SELECT id, name, description, icon, display_order, created_at FROM genres WHERE id = ?1",
            [&story.genre_id],
            |row| {
                Ok(Genre {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    display_order: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )
        .map_err(|e| format!("Genre not found: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, story_id, part_number, content, audio_path, status, created_at, updated_at FROM story_parts WHERE story_id = ?1 ORDER BY part_number")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let parts = stmt
        .query_map([&story_id], |row| {
            Ok(StoryPart {
                id: row.get(0)?,
                story_id: row.get(1)?,
                part_number: row.get(2)?,
                content: row.get(3)?,
                audio_path: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query parts: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to read parts: {}", e))?;

    Ok(StoryWithParts {
        story,
        genre,
        parts,
    })
}
