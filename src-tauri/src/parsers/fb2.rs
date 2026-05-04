use crate::models::{BookMetadata, ParsedBook, TocEntry};
use crate::parsers::process_text;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::path::Path;

struct OpenSection {
    title: String,
    start_word_index: Option<usize>,
    children: Vec<TocEntry>,
}

pub fn parse(path: &Path) -> Result<ParsedBook, String> {
    let xml = std::fs::read_to_string(path).map_err(|e| format!("Failed to read FB2 file: {e}"))?;

    let mut reader = Reader::from_str(&xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut current_text = String::new();

    let mut title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Title")
        .to_string();
    let mut author = None::<String>;
    let mut genres = Vec::new();
    let mut annotation_parts = Vec::new();
    let mut date = String::new();
    let mut keywords = Vec::new();
    let mut lang = String::new();
    let mut sequence = String::new();

    let mut body_parts = Vec::new();
    let mut in_body = false;
    let mut first_name = String::new();
    let mut last_name = String::new();

    let mut word_count: usize = 0;
    let mut section_stack: Vec<OpenSection> = Vec::new();
    let mut root_toc: Vec<TocEntry> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if tag == "body" {
                    in_body = true;
                }
                if in_body && tag == "section" {
                    section_stack.push(OpenSection {
                        title: String::new(),
                        start_word_index: None,
                        children: Vec::new(),
                    });
                }
                if tag == "sequence" {
                    let mut name = String::new();
                    let mut number = String::new();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let value = String::from_utf8_lossy(&attr.value).to_string();
                        if key == "name" {
                            name = value;
                        } else if key == "number" {
                            number = value;
                        }
                    }
                    sequence = if number.is_empty() {
                        name
                    } else {
                        format!("{name} #{number}")
                    };
                }
                stack.push(tag);
            }
            Ok(Event::End(e)) => {
                let tag = String::from_utf8_lossy(e.local_name().as_ref()).to_string();

                let text_value = current_text.trim().to_string();
                if !text_value.is_empty() {
                    if in_tag_path(&stack, &["description", "title-info", "book-title"]) {
                        title = text_value.clone();
                    } else if in_tag_path(&stack, &["description", "title-info", "genre"]) {
                        genres.push(text_value.clone());
                    } else if in_tag_path(&stack, &["description", "title-info", "annotation", "p"]) {
                        annotation_parts.push(text_value.clone());
                    } else if in_tag_path(&stack, &["description", "title-info", "date"]) {
                        date = text_value.clone();
                    } else if in_tag_path(&stack, &["description", "title-info", "lang"]) {
                        lang = text_value.clone();
                    } else if in_tag_path(&stack, &["description", "title-info", "keywords"]) {
                        keywords.extend(
                            text_value
                                .split([',', ';'])
                                .map(str::trim)
                                .filter(|s| !s.is_empty())
                                .map(ToString::to_string),
                        );
                    } else if in_body
                        && matches!(
                            stack.last().map(String::as_str),
                            Some("p") | Some("title") | Some("subtitle") | Some("v")
                        )
                    {
                        let trimmed = text_value.trim();
                        if !trimmed.is_empty() {
                            let words_before = word_count;
                            for raw_word in trimmed.split_whitespace() {
                                if !raw_word.is_empty() {
                                    word_count += 1;
                                }
                            }
                            body_parts.push(trimmed.to_string());

                            if in_tag_path(&stack, &["body", "section", "title"]) {
                                if let Some(sec) = section_stack.last_mut() {
                                    sec.title = trimmed.to_string();
                                }
                            }

                            for sec in section_stack.iter_mut().rev() {
                                if sec.start_word_index.is_none() {
                                    sec.start_word_index = Some(words_before);
                                } else {
                                    break;
                                }
                            }
                        }
                    } else if in_tag_path(&stack, &["description", "title-info", "author", "first-name"]) {
                        first_name = text_value.clone();
                    } else if in_tag_path(&stack, &["description", "title-info", "author", "last-name"]) {
                        last_name = text_value.clone();
                    }
                }

                if tag == "section" && in_body {
                    if let Some(done) = section_stack.pop() {
                        let entry = TocEntry {
                            title: if done.title.trim().is_empty() {
                                "Untitled".to_string()
                            } else {
                                done.title
                            },
                            word_index: done.start_word_index.unwrap_or(0),
                            children: done.children,
                        };
                        if let Some(parent) = section_stack.last_mut() {
                            parent.children.push(entry);
                        } else {
                            root_toc.push(entry);
                        }
                    }
                }

                if tag == "body" {
                    in_body = false;
                }
                current_text.clear();
                if !stack.is_empty() {
                    stack.pop();
                }
            }
            Ok(Event::Text(e)) => {
                if let Ok(decoded) = e.unescape() {
                    if !current_text.is_empty() {
                        current_text.push(' ');
                    }
                    current_text.push_str(&decoded);
                }
            }
            Ok(Event::CData(e)) => {
                let decoded = String::from_utf8_lossy(e.as_ref());
                if !current_text.is_empty() {
                    current_text.push(' ');
                }
                current_text.push_str(&decoded);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Failed to parse FB2 XML: {e}")),
            _ => {}
        }
        buf.clear();
    }

    if author.is_none() {
        let full = format!("{first_name} {last_name}").trim().to_string();
        if !full.is_empty() {
            author = Some(full);
        }
    }

    let text = body_parts.join("\n\n");
    let toc_override = if root_toc.is_empty() {
        None
    } else {
        Some(root_toc)
    };

    let mut parsed = process_text(&text, &title, author, toc_override);
    parsed.metadata = Some(BookMetadata {
        genre: genres.join(", "),
        year_written: extract_year(&date),
        summary: annotation_parts.join("\n\n"),
        themes: keywords,
        setting: String::new(),
        key_characters: Vec::new(),
        notable_context: [lang, sequence]
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" | "),
    });

    Ok(parsed)
}

fn in_tag_path(stack: &[String], path: &[&str]) -> bool {
    if stack.len() < path.len() {
        return false;
    }
    stack[stack.len() - path.len()..]
        .iter()
        .zip(path.iter())
        .all(|(a, b)| a == b)
}

fn extract_year(date: &str) -> String {
    let year: String = date.chars().filter(|c| c.is_ascii_digit()).take(4).collect();
    if year.len() == 4 {
        year
    } else {
        date.to_string()
    }
}
