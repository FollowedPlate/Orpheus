use crate::models::{Library, LibraryEntry, ProgressUpdate, ReadingSession};
use chrono::Utc;
use std::path::PathBuf;
use tauri::Manager;

fn library_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))?;
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data dir: {e}"))?;
    Ok(data_dir.join("library.json"))
}

#[tauri::command]
pub async fn load_library(app: tauri::AppHandle) -> Result<Library, String> {
    let path = library_path(&app)?;

    if !path.exists() {
        return Ok(Library::default());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read library: {e}"))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse library: {e}"))
}

#[tauri::command]
pub async fn save_library(app: tauri::AppHandle, library: Library) -> Result<(), String> {
    let path = library_path(&app)?;

    let content =
        serde_json::to_string_pretty(&library).map_err(|e| format!("Failed to serialize library: {e}"))?;

    std::fs::write(&path, content).map_err(|e| format!("Failed to write library: {e}"))
}

#[tauri::command]
pub async fn update_progress(
    app: tauri::AppHandle,
    update: ProgressUpdate,
) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;

    if let Some(entry) = library
        .entries
        .iter_mut()
        .find(|e| e.book.id == update.book_id)
    {
        entry.current_word_index = update.current_word_index;
        entry.progress = if entry.book.word_count > 0 {
            update.current_word_index as f64 / entry.book.word_count as f64
        } else {
            0.0
        };
        entry.book.last_read_at = Some(Utc::now());

        if update.session_ended {
            if let Some(session) = entry.sessions.last_mut() {
                if session.ended_at.is_none() {
                    session.ended_at = Some(Utc::now());
                    session.words_read = update.words_read;
                    session.average_wpm = update.average_wpm;
                    session.quiz_score = update.quiz_score;
                }
            }
        }
    }

    save_library(app, library).await
}

#[tauri::command]
pub async fn add_book_to_library(app: tauri::AppHandle, entry: LibraryEntry) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;

    // Remove existing entry for same file if present
    library
        .entries
        .retain(|e| e.book.file_path != entry.book.file_path);

    library.entries.push(entry);
    save_library(app, library).await
}

#[tauri::command]
pub async fn remove_book(app: tauri::AppHandle, book_id: String) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;
    library.entries.retain(|e| e.book.id != book_id);
    save_library(app, library).await
}

#[tauri::command]
pub async fn start_session(app: tauri::AppHandle, book_id: String) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;

    if let Some(entry) = library.entries.iter_mut().find(|e| e.book.id == book_id) {
        entry.sessions.push(ReadingSession {
            started_at: Utc::now(),
            ended_at: None,
            words_read: 0,
            average_wpm: 0.0,
            quiz_score: None,
        });
    }

    save_library(app, library).await
}
