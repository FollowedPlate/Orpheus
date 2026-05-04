use crate::models::{BookMetadata, ParsedBook, TocEntry};
use crate::parsers::{count_words_like_process, process_text, strip_html};
use epub::doc::{EpubDoc, NavPoint};
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};

pub fn parse(path: &Path) -> Result<ParsedBook, String> {
    let mut doc = EpubDoc::new(path).map_err(|e| format!("Failed to open EPUB: {e}"))?;

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

    let description = doc
        .mdata("description")
        .map(|m| m.value.clone())
        .unwrap_or_default();
    let genre = doc.mdata("subject").map(|m| m.value.clone()).unwrap_or_default();
    let year_written = doc.mdata("date").map(|m| m.value.clone()).unwrap_or_default();
    let publisher = doc.mdata("publisher").map(|m| m.value.clone()).unwrap_or_default();
    let language = doc.mdata("language").map(|m| m.value.clone()).unwrap_or_default();
    let identifier = doc.mdata("identifier").map(|m| m.value.clone()).unwrap_or_default();

    let n_chapters = doc.get_num_chapters();
    let mut spine_word_starts: Vec<usize> = Vec::with_capacity(n_chapters);
    let mut full_text = String::new();

    for i in 0..n_chapters {
        doc.set_current_chapter(i);
        spine_word_starts.push(count_words_like_process(&full_text));

        if let Some((content, mime)) = doc.get_current_str() {
            if mime.contains("html") || mime.contains("xhtml") {
                full_text.push_str(&strip_html(&content));
                full_text.push_str("\n\n");
            }
        }
    }

    let toc_override = if doc.toc.is_empty() {
        Some(build_spine_flat_toc(&mut doc, &spine_word_starts))
    } else {
        let converted = navpoints_to_toc(&doc.toc, &doc, &spine_word_starts);
        if converted.is_empty() {
            Some(build_spine_flat_toc(&mut doc, &spine_word_starts))
        } else {
            Some(converted)
        }
    };

    let mut parsed = process_text(&full_text, &title, author, toc_override);
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

/// EPUB nav / NCX paths may include `#fragment`; spine lookup uses the file path only.
fn path_without_fragment(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(i) = s.find('#') {
        PathBuf::from(s[..i].to_string())
    } else {
        path.to_path_buf()
    }
}

fn navpoints_to_toc<R: Read + Seek>(nav: &[NavPoint], doc: &EpubDoc<R>, spine_starts: &[usize]) -> Vec<TocEntry> {
    let mut out = Vec::new();
    for np in nav {
        if let Some(entry) = navpoint_to_entry(np, doc, spine_starts) {
            out.push(entry);
        }
    }
    out
}

fn navpoint_to_entry<R: Read + Seek>(
    np: &NavPoint,
    doc: &EpubDoc<R>,
    spine_starts: &[usize],
) -> Option<TocEntry> {
    let path = path_without_fragment(&np.content);
    let word_index = doc
        .resource_uri_to_chapter(&path)
        .and_then(|c| spine_starts.get(c).copied())
        .unwrap_or(0);
    let children = navpoints_to_toc(&np.children, doc, spine_starts);
    let label = np.label.trim();
    if label.is_empty() && children.is_empty() {
        return None;
    }
    Some(TocEntry {
        title: if label.is_empty() {
            "Untitled".to_string()
        } else {
            label.to_string()
        },
        word_index,
        children,
    })
}

fn build_spine_flat_toc<R: Read + Seek>(doc: &mut EpubDoc<R>, spine_starts: &[usize]) -> Vec<TocEntry> {
    let n = doc.get_num_chapters();
    let mut entries = Vec::with_capacity(n);
    for i in 0..n {
        doc.set_current_chapter(i);
        let label = if let Some((html, mime)) = doc.get_current_str() {
            if mime.contains("html") || mime.contains("xhtml") {
                first_heading_from_html(&html).unwrap_or_else(|| spine_chapter_fallback(doc, i))
            } else {
                spine_chapter_fallback(doc, i)
            }
        } else {
            spine_chapter_fallback(doc, i)
        };
        let word_index = spine_starts.get(i).copied().unwrap_or(0);
        entries.push(TocEntry {
            title: label,
            word_index,
            children: Vec::new(),
        });
    }
    entries
}

fn spine_chapter_fallback<R: Read + Seek>(doc: &EpubDoc<R>, i: usize) -> String {
    doc.get_current_id()
        .map(|id| {
            if id.to_lowercase().ends_with(".xhtml") || id.to_lowercase().ends_with(".html") {
                id.trim_end_matches(".xhtml")
                    .trim_end_matches(".html")
                    .replace(['_', '-'], " ")
            } else {
                id
            }
        })
        .filter(|s| !s.is_empty())
        .map(|s| format!("Section: {s}"))
        .unwrap_or_else(|| format!("Chapter {}", i + 1))
}

fn first_heading_from_html(html: &str) -> Option<String> {
    let stripped = strip_html(html);
    let line = stripped.lines().map(str::trim).find(|l| !l.is_empty())?;
    if line.len() < 500 {
        Some(line.to_string())
    } else {
        None
    }
}
