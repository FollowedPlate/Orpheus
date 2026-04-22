pub mod azw3;
pub mod epub;
pub mod pdf;
pub mod txt;

use crate::models::ParsedBook;

/// Shared text-to-ParsedBook logic used by all parsers.
/// Takes raw text and optional chapter boundary markers, returns a ParsedBook.
pub fn process_text(text: &str, title: &str, author: Option<String>) -> ParsedBook {
    let mut words: Vec<String> = Vec::new();
    let mut paragraph_indices: Vec<usize> = Vec::new();
    let mut sentence_indices: Vec<usize> = Vec::new();
    let mut chapter_indices: Vec<usize> = Vec::new();

    // Split into paragraphs by double newline or single newline for plain text
    let paragraphs: Vec<&str> = split_paragraphs(text);

    let mut is_first_word = true;

    for paragraph in paragraphs.iter() {
        let para_trimmed = paragraph.trim();
        if para_trimmed.is_empty() {
            continue;
        }

        let word_start = words.len();

        // Heuristic: detect chapter headings (short lines, all caps, or starts with "Chapter")
        if is_chapter_heading(para_trimmed) {
            if word_start > 0 {
                chapter_indices.push(word_start);
            }
        }

        // Track paragraph start
        if !is_first_word {
            paragraph_indices.push(word_start);
        }

        let mut in_sentence_start = true;

        for raw_word in para_trimmed.split_whitespace() {
            if raw_word.is_empty() {
                continue;
            }

            let word_idx = words.len();

            if in_sentence_start {
                sentence_indices.push(word_idx);
                in_sentence_start = false;
            }

            words.push(raw_word.to_string());
            is_first_word = false;

            // Check if this word ends a sentence
            if ends_sentence(raw_word) {
                in_sentence_start = true;
            }
        }
    }

    // Ensure index 0 is a sentence start if we have words
    if !words.is_empty() && !sentence_indices.contains(&0) {
        sentence_indices.insert(0, 0);
    }

    ParsedBook {
        title: title.to_string(),
        author,
        words,
        chapter_indices,
        paragraph_indices,
        sentence_indices,
    }
}

fn split_paragraphs(text: &str) -> Vec<&str> {
    // Try double-newline split first
    let double_newline: Vec<&str> = text.split("\n\n").collect();
    if double_newline.len() > 1 {
        return double_newline;
    }
    // Fall back to single newline
    text.split('\n').collect()
}

fn ends_sentence(word: &str) -> bool {
    // Strip trailing quotes/parentheses and check for sentence-ending punctuation
    let stripped = word.trim_end_matches(|c| matches!(c, '"' | '\'' | ')' | ']' | '»'));
    stripped.ends_with('.')
        || stripped.ends_with('!')
        || stripped.ends_with('?')
        || stripped.ends_with("...")
        || stripped.ends_with('…')
}

fn is_chapter_heading(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.starts_with("chapter ")
        || lower.starts_with("part ")
        || lower.starts_with("book ")
        || lower.starts_with("prologue")
        || lower.starts_with("epilogue")
        || lower.starts_with("introduction")
        || lower.starts_with("conclusion")
    {
        return true;
    }

    // Short line that's all caps (likely a heading)
    let word_count = text.split_whitespace().count();
    if word_count <= 6 && text == text.to_uppercase() && text.chars().any(|c| c.is_alphabetic()) {
        return true;
    }

    false
}
