use crate::models::{BookMetadata, ParsedBook};
use crate::parsers::process_text;
use lopdf::{Document, Object};
use std::path::Path;

pub fn parse(path: &Path) -> Result<ParsedBook, String> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| format!("Failed to extract PDF text: {e}"))?;

    let fallback_title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string();

    let (title, author, metadata) = extract_pdf_metadata(path, &fallback_title);
    let mut parsed = process_text(&text, &title, author);
    parsed.metadata = Some(metadata);
    Ok(parsed)
}

fn extract_pdf_metadata(path: &Path, fallback_title: &str) -> (String, Option<String>, BookMetadata) {
    let mut title = fallback_title.to_string();
    let mut author: Option<String> = None;
    let mut summary = String::new();
    let mut themes = Vec::new();
    let mut year_written = String::new();
    let mut context_parts = Vec::new();

    if let Ok(doc) = Document::load(path) {
        if let Ok(info) = doc.trailer.get(b"Info") {
            if let Ok(info_ref) = info.as_reference() {
                if let Ok(dict) = doc.get_dictionary(info_ref) {
                    // if let Some(v) = get_pdf_dict_text(dict.get(b"Title")) { //not working properly, gets junk characters for many pdf files, better to just get the title from file name
                    //     title = v;
                    // }
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Author")) {
                        author = Some(v);
                    }
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Subject")) {
                        summary = v;
                    }
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Keywords")) {
                        themes = split_keywords(&v);
                    }
                    // if let Some(v) = get_pdf_dict_text(dict.get(b"CreationDate")) { //also unreliable, better to generate from llm
                    //     year_written = extract_pdf_year(&v);
                    // }
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Creator")) {
                        context_parts.push(format!("Creator: {v}"));
                    }
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Producer")) {
                        context_parts.push(format!("Producer: {v}"));
                    }
                }
            }
        }
    }

    (
        title,
        author,
        BookMetadata {
            genre: String::new(),
            year_written,
            summary,
            themes,
            setting: String::new(),
            key_characters: Vec::new(),
            notable_context: context_parts.join(" | "),
        },
    )
}

fn get_pdf_dict_text(value: Result<&Object, lopdf::Error>) -> Option<String> {
    let obj = value.ok()?;
    match obj {
        Object::String(bytes, _) => Some(String::from_utf8_lossy(bytes).trim().to_string()),
        Object::Name(bytes) => Some(String::from_utf8_lossy(bytes).trim().to_string()),
        _ => None,
    }
    .filter(|s| !s.is_empty())
}

fn extract_pdf_year(creation_date: &str) -> String {
    if let Some(rest) = creation_date.strip_prefix("D:") {
        return rest.chars().take(4).collect();
    }
    creation_date.chars().take(4).collect()
}

fn split_keywords(raw: &str) -> Vec<String> {
    raw.split([',', ';'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}
