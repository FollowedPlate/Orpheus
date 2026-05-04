use crate::models::{BookMetadata, ParsedBook, TocEntry};
use crate::parsers::{count_words_like_process, process_text};
use indexmap::IndexMap;
use lopdf::{Destination, Document, Object, Outline};
use std::path::Path;

pub fn parse(path: &Path) -> Result<ParsedBook, String> {
    let text = pdf_extract::extract_text(path)
        .map_err(|e| format!("Failed to extract PDF text: {e}"))?;

    let fallback_title = path
        .file_stem()
        .and_then(|s| -> Option<&str> { Some(s.to_str()?) })
        .unwrap_or("Unknown Title")
        .to_string();

    let (title, author, metadata) = extract_pdf_metadata(path, &fallback_title);

    let toc_override = outline_text_to_toc(path, &text);

    let mut parsed = process_text(&text, &title, author, toc_override);
    parsed.metadata = Some(metadata);
    Ok(parsed)
}

/// Map PDF outline titles to word indices by searching the extracted text.
fn outline_text_to_toc(path: &Path, full_text: &str) -> Option<Vec<TocEntry>> {
    let doc = Document::load(path).ok()?;
    let mut named = IndexMap::new();
    let roots = doc.get_outlines(None, None, &mut named).ok()??;
    if roots.is_empty() {
        return None;
    }
    let toc = outlines_to_toc(&roots, full_text);
    if toc.is_empty() {
        None
    } else {
        Some(toc)
    }
}

fn outlines_to_toc(items: &[Outline], full_text: &str) -> Vec<TocEntry> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < items.len() {
        match &items[i] {
            Outline::Destination(dest) => {
                let mut entry = match destination_to_entry(dest, full_text) {
                    Some(e) => e,
                    None => {
                        i += 1;
                        continue;
                    }
                };

                if i + 1 < items.len() {
                    if let Outline::SubOutlines(sub) = &items[i + 1] {
                        entry.children = outlines_to_toc(sub, full_text);
                        i += 2;
                        out.push(entry);
                        continue;
                    }
                }
                i += 1;
                out.push(entry);
            }
            Outline::SubOutlines(sub) => {
                out.extend(outlines_to_toc(sub, full_text));
                i += 1;
            }
        }
    }
    out
}

fn destination_to_entry(dest: &Destination, full_text: &str) -> Option<TocEntry> {
    let title = pdf_destination_title(dest)?;
    let word_index = find_title_word_index(full_text, &title)?;
    Some(TocEntry {
        title,
        word_index,
        children: Vec::new(),
    })
}

fn pdf_destination_title(dest: &Destination) -> Option<String> {
    let t = dest.title()?;
    let s = t.as_string().ok()?;
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_string())
    }
}

fn find_title_word_index(full_text: &str, title: &str) -> Option<usize> {
    let t = title.trim();
    if t.len() < 2 {
        return None;
    }
    if let Some(byte_idx) = full_text.find(t) {
        return Some(byte_offset_to_word_index(full_text, byte_idx));
    }
    // Try first line of title (bookmarks sometimes include extra text)
    if let Some(first) = t.lines().next() {
        if first.len() >= 2 {
            if let Some(byte_idx) = full_text.find(first) {
                return Some(byte_offset_to_word_index(full_text, byte_idx));
            }
        }
    }
    None
}

fn byte_offset_to_word_index(text: &str, byte_off: usize) -> usize {
    let end = byte_off.min(text.len());
    let prefix = text.get(..end).unwrap_or("");
    count_words_like_process(prefix)
}

fn extract_pdf_metadata(path: &Path, fallback_title: &str) -> (String, Option<String>, BookMetadata) {
    let title = fallback_title.to_string();
    let mut author: Option<String> = None;
    let mut summary = String::new();
    let mut themes = Vec::new();
    let year_written = String::new();
    let mut context_parts = Vec::new();

    if let Ok(doc) = Document::load(path) {
        if let Ok(info) = doc.trailer.get(b"Info") {
            if let Ok(info_ref) = info.as_reference() {
                if let Ok(dict) = doc.get_dictionary(info_ref) {
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Author")) {
                        author = Some(v);
                    }
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Subject")) {
                        summary = v;
                    }
                    if let Some(v) = get_pdf_dict_text(dict.get(b"Keywords")) {
                        themes = split_keywords(&v);
                    }
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

#[allow(dead_code)]
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
