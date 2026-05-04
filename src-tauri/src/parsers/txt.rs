use crate::models::Book;
use crate::parsers::process_text;
use std::path::Path;

pub fn parse(path: &Path) -> Result<Book, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read text file: {e}"))?;

    // Try to extract a title from the first non-empty line
    let title = extract_title(&text, path);

    Ok(process_text(&text, &title, None, None))
}

fn extract_title(text: &str, path: &Path) -> String {
    // Try first non-empty line as title
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.len() < 100 {
            return trimmed.to_string();
        }
    }

    // Fall back to filename
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string()
}
