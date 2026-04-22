use crate::models::ParsedBook;
use crate::parsers;
use std::path::Path;

#[tauri::command]
pub async fn parse_book(path: String) -> Result<ParsedBook, String> {
    let file_path = Path::new(&path);

    let extension = file_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match extension.as_str() {
        "txt" => parsers::txt::parse(file_path),
        "pdf" => parsers::pdf::parse(file_path),
        "epub" => parsers::epub::parse(file_path),
        "azw3" | "mobi" => parsers::azw3::parse(file_path),
        other => Err(format!("Unsupported file format: .{other}")),
    }
}
