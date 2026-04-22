use crate::models::ParsedBook;
use crate::parsers::process_text;
use std::path::Path;

pub fn parse(path: &Path) -> Result<ParsedBook, String> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| format!("Failed to extract PDF text: {e}"))?;

    let title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string();

    Ok(process_text(&text, &title, None))
}
