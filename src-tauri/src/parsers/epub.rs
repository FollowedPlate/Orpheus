use crate::models::ParsedBook;
use crate::parsers::process_text;
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

    Ok(process_text(&full_text, &title, author))
}

/// Strip HTML tags from a string, preserving paragraph structure.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '<' => {
                in_tag = true;
                // Check for block-level tags to insert newlines
                let tag_start: String = chars
                    .clone()
                    .take(10)
                    .collect::<String>()
                    .to_lowercase();
                if tag_start.starts_with("p")
                    || tag_start.starts_with("/p")
                    || tag_start.starts_with("br")
                    || tag_start.starts_with("h1")
                    || tag_start.starts_with("h2")
                    || tag_start.starts_with("h3")
                    || tag_start.starts_with("h4")
                    || tag_start.starts_with("h5")
                    || tag_start.starts_with("h6")
                    || tag_start.starts_with("div")
                    || tag_start.starts_with("/div")
                {
                    result.push('\n');
                }
            }
            '>' => {
                in_tag = false;
            }
            '&' if !in_tag => {
                // Decode common HTML entities
                let entity: String = chars.clone().take(10).collect();
                if entity.starts_with("nbsp;") {
                    result.push(' ');
                    for _ in 0..5 {
                        chars.next();
                    }
                } else if entity.starts_with("amp;") {
                    result.push('&');
                    for _ in 0..4 {
                        chars.next();
                    }
                } else if entity.starts_with("lt;") {
                    result.push('<');
                    for _ in 0..3 {
                        chars.next();
                    }
                } else if entity.starts_with("gt;") {
                    result.push('>');
                    for _ in 0..3 {
                        chars.next();
                    }
                } else if entity.starts_with("quot;") {
                    result.push('"');
                    for _ in 0..5 {
                        chars.next();
                    }
                } else if entity.starts_with("apos;") {
                    result.push('\'');
                    for _ in 0..5 {
                        chars.next();
                    }
                } else {
                    result.push(ch);
                }
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
}
