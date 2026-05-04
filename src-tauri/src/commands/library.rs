use super::files;
use crate::models::{Book, BookMetadata, Library, ProgressUpdate, ReadingSession};
use chrono::Utc;
use serde::Deserialize;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Deserialize)]
struct LegacyBook {
    id: String,
    title: String,
    author: Option<String>,
    file_path: String,
    file_format: String,
    word_count: usize,
    added_at: chrono::DateTime<Utc>,
    last_read_at: Option<chrono::DateTime<Utc>>,
    metadata: Option<BookMetadata>,
}

#[derive(Debug, Deserialize)]
struct LegacyLibraryEntry {
    book: LegacyBook,
    current_word_index: usize,
    progress: f64,
    sessions: Vec<ReadingSession>,
}

#[derive(Debug, Deserialize)]
struct LegacyLibrary {
    entries: Vec<LegacyLibraryEntry>,
}

fn library_paths_equivalent(a: &Option<String>, b: &Option<String>) -> bool {
    match (a.as_deref(), b.as_deref()) {
        (Some(x), Some(y)) => {
            x.replace('\\', "/").to_lowercase() == y.replace('\\', "/").to_lowercase()
        }
        _ => false,
    }
}

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

    if let Ok(current) = serde_json::from_str::<Library>(&content) {
        return Ok(current);
    }

    // Backward compatibility: migrate old nested shape:
    // { entries: [ { book: {...}, current_word_index, progress, sessions } ] }
    if let Ok(legacy) = serde_json::from_str::<LegacyLibrary>(&content) {
        let mut migrated = Library {
            entries: Vec::with_capacity(legacy.entries.len()),
        };
        for old in legacy.entries {
            let mut next = Book {
                title: old.book.title,
                id: Some(old.book.id),
                author: old.book.author,
                file_path: Some(old.book.file_path),
                file_format: Some(old.book.file_format),
                word_count: Some(old.book.word_count),
                added_at: Some(old.book.added_at),
                last_read_at: old.book.last_read_at,
                words: None,
                chapter_indices: None,
                paragraph_indices: None,
                sentence_indices: None,
                toc: None,
                current_word_index: Some(old.current_word_index),
                progress: Some(old.progress),
                sessions: Some(old.sessions),
                genre: None,
                year_written: None,
                summary: None,
                themes: None,
                setting: None,
                key_characters: None,
                notable_context: None,
            };
            if let Some(meta) = old.book.metadata {
                next.set_metadata(meta);
            }
            migrated.entries.push(next);
        }

        // Best-effort rewrite to the new format after successful migration.
        if let Ok(serialized) = serde_json::to_string_pretty(&migrated) {
            let _ = std::fs::write(&path, serialized);
        }
        return Ok(migrated);
    }

    Err("Failed to parse library: unsupported library.json format".to_string())
}

#[tauri::command]
pub async fn save_library(app: tauri::AppHandle, library: Library) -> Result<(), String> {
    let path = library_path(&app)?;

    let content = serde_json::to_string_pretty(&library)
        .map_err(|e| format!("Failed to serialize library: {e}"))?;

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
        .find(|e| e.id.as_deref() == Some(update.book_id.as_str()))
    {
        entry.current_word_index = Some(update.current_word_index);
        let wc = entry.word_count.unwrap_or(0);
        entry.progress = Some(if wc > 0 {
            update.current_word_index as f64 / wc as f64
        } else {
            0.0
        });
        entry.last_read_at = Some(Utc::now());

        if update.session_ended {
            if let Some(sessions) = entry.sessions.as_mut() {
                if let Some(session) = sessions.last_mut() {
                    if session.ended_at.is_none() {
                        session.ended_at = Some(Utc::now());
                        session.words_read = update.words_read;
                        session.average_wpm = update.average_wpm;
                        session.quiz_score = update.quiz_score;
                    }
                }
            }
        }
    }

    save_library(app, library).await
}

#[tauri::command]
pub async fn add_book_to_library(app: tauri::AppHandle, entry: Book) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;

    // Remove existing entry for same file if present (path compare is case- and slash-insensitive)
    library
        .entries
        .retain(|e| !library_paths_equivalent(&e.file_path, &entry.file_path));

    library.entries.push(entry);
    save_library(app, library).await
}

#[tauri::command]
pub async fn remove_book(app: tauri::AppHandle, book_id: String) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;
    library
        .entries
        .retain(|e| e.id.as_deref() != Some(book_id.as_str()));
    save_library(app, library).await
}

#[tauri::command]
pub async fn start_session(app: tauri::AppHandle, book_id: String) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;

    if let Some(entry) = library
        .entries
        .iter_mut()
        .find(|e| e.id.as_deref() == Some(book_id.as_str()))
    {
        let sessions = entry.sessions.get_or_insert_with(Vec::new);
        sessions.push(ReadingSession {
            started_at: Utc::now(),
            ended_at: None,
            words_read: 0,
            average_wpm: 0.0,
            quiz_score: None,
        });
    }

    save_library(app, library).await
}

#[tauri::command]
pub async fn update_book_metadata(
    app: tauri::AppHandle,
    book_id: String,
    metadata: BookMetadata,
) -> Result<(), String> {
    let mut library = load_library(app.clone()).await?;

    if let Some(entry) = library
        .entries
        .iter_mut()
        .find(|e| e.id.as_deref() == Some(book_id.as_str()))
    {
        entry.set_metadata(metadata);
    }

    save_library(app, library).await
}

/// Point an existing library entry at a new file on disk (after move/rename). Re-parses the file
/// and updates title, author, word count, and format. Progress index is clamped to the new length.
#[tauri::command]
pub async fn update_book_file_path(
    app: tauri::AppHandle,
    book_id: String,
    new_path: String,
) -> Result<(), String> {
    let parsed = files::parse_book_sync(&new_path)?;

    let mut library = load_library(app.clone()).await?;

    let entry = library
        .entries
        .iter_mut()
        .find(|e| e.id.as_deref() == Some(book_id.as_str()))
        .ok_or_else(|| "Book not found in library".to_string())?;

    let path_buf = PathBuf::from(&new_path);
    let file_format = path_buf
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("txt")
        .to_lowercase();

    let word_count = parsed.words.as_ref().map(|w| w.len()).unwrap_or(0);

    entry.file_path = Some(new_path);
    entry.file_format = Some(file_format);
    entry.title = parsed.title;
    entry.author = parsed.author;
    entry.word_count = Some(word_count);
    entry.clear_metadata();

    let prev_idx = entry.current_word_index.unwrap_or(0);
    entry.current_word_index = Some(if word_count == 0 {
        0
    } else {
        prev_idx.min(word_count - 1)
    });
    entry.progress = Some(if word_count > 0 {
        entry.current_word_index.unwrap_or(0) as f64 / word_count as f64
    } else {
        0.0
    });

    save_library(app, library).await
}
