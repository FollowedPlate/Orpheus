use crate::models::ParsedBook;
use crate::parsers::process_text;
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

    let content = book.content_as_string_lossy();

    // MOBI content may contain HTML
    let text = if content.contains('<') {
        strip_basic_html(&content)
    } else {
        content.to_string()
    };

    Ok(process_text(&text, &title, author))
}

fn strip_basic_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                result.push(' ');
            }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
