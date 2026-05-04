use crate::models::{BookMetadata, ParsedBook};
use crate::parsers::{process_text, strip_html};
use std::path::Path;

pub fn parse(path: &Path) -> Result<ParsedBook, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read AZW3/MOBI file: {e}"))?;

    let book = mobi::Mobi::new(bytes)
        .map_err(|e| format!("Failed to parse AZW3/MOBI: {e}"))?;

    let title = {
        let t = book.title();
        if t.is_empty() {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown Title")
                .to_string()
        } else {
            t.to_string()
        }
    };

    let author = book.author().map(|a| a.to_string());

    let content = book
        .content_as_string()
        .unwrap_or_else(|_| book.content_as_string_lossy());

    let text = if content.contains('<') {
        strip_html(&content)
    } else {
        content.to_string()
    };

    let mut context_parts = Vec::new();
    let publisher = book.publisher().unwrap_or_default().to_string();
    if !publisher.trim().is_empty() {
        context_parts.push(publisher);
    }
    let language = format!("{:?}", book.language());
    if !language.trim().is_empty() {
        context_parts.push(language);
    }
    let isbn = book.isbn().unwrap_or_default().to_string();
    if !isbn.trim().is_empty() {
        context_parts.push(isbn);
    }

    let mut parsed = process_text(&text, &title, author);
    parsed.metadata = Some(BookMetadata {
        genre: String::new(),
        year_written: book.publish_date().unwrap_or_default().to_string(),
        summary: book.description().unwrap_or_default().to_string(),
        themes: Vec::new(),
        setting: String::new(),
        key_characters: Vec::new(),
        notable_context: context_parts.join(" | "),
    });
    Ok(parsed)
}
