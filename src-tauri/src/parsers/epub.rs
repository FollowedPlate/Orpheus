use crate::models::{BookMetadata, ParsedBook};
use crate::parsers::{process_text, strip_html};
use epub::doc::EpubDoc;
use std::path::Path;

pub fn parse(path: &Path) -> Result<ParsedBook, String> {
    let mut doc =
        EpubDoc::new(path).map_err(|e| format!("Failed to open EPUB: {e}"))?;

    let title = doc
        .mdata("title")
        .map(|m| m.value.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown Title")
                .to_string()
        });

    let author = doc.mdata("creator").map(|m| m.value.clone());

    let description = doc.mdata("description").map(|m| m.value.clone()).unwrap_or_default();
    let genre = doc.mdata("subject").map(|m| m.value.clone()).unwrap_or_default();
    let year_written = doc.mdata("date").map(|m| m.value.clone()).unwrap_or_default();
    let publisher = doc.mdata("publisher").map(|m| m.value.clone()).unwrap_or_default();
    let language = doc.mdata("language").map(|m| m.value.clone()).unwrap_or_default();
    let identifier = doc.mdata("identifier").map(|m| m.value.clone()).unwrap_or_default();

    let mut full_text = String::new();

    loop {
        if let Some((content, mime)) = doc.get_current_str() {
            if mime.contains("html") || mime.contains("xhtml") {
                full_text.push_str(&strip_html(&content));
                full_text.push_str("\n\n");
            }
        }
        if !doc.go_next() {
            break;
        }
    }

    let mut parsed = process_text(&full_text, &title, author);
    parsed.metadata = Some(BookMetadata {
        genre,
        year_written,
        summary: description,
        themes: Vec::new(),
        setting: String::new(),
        key_characters: Vec::new(),
        notable_context: [publisher, language, identifier]
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" | "),
    });
    Ok(parsed)
}
