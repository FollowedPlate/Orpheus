use crate::models::Book;
use crate::parsers;
use crate::paths;
use std::path::Path;

/// Returned when the path is missing or not a regular file (used by the UI for friendly handling).
pub const LIBRARY_FILE_NOT_FOUND: &str = "LIBRARY_FILE_NOT_FOUND";

pub fn parse_book_sync(path: &str) -> Result<Book, String> {
    let file_path = Path::new(path);

    if !file_path.is_file() {
        return Err(LIBRARY_FILE_NOT_FOUND.to_string());
    }

    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "txt" => parsers::txt::parse(file_path),
        "pdf" => parsers::pdf::parse(file_path),
        "epub" => parsers::epub::parse(file_path),
        "azw3" | "mobi" => parsers::mobi::parse(file_path),
        "fb2" => parsers::fb2::parse(file_path),
        other => Err(format!("Unsupported file format: .{other}")),
    }
}

#[tauri::command]
pub async fn parse_book(path: String) -> Result<Book, String> {
    parse_book_sync(&path)
}

#[tauri::command]
pub async fn open_user_book_files_dir(app: tauri::AppHandle) -> Result<(), String> {
    let dir = paths::user_book_files_dir(&app)?;
    open::that(&dir).map_err(|e| format!("Failed to open book files folder: {e}"))?;
    Ok(())
}
