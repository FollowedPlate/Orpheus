use crate::models::{
    AnswerEvaluation, BookMetadata, ChatMessage, ChatRequest, ChatResponse, LlmProvider,
    LlmQuestion, QuestionStyle, Settings,
};

/// OpenAI uses `{endpoint}/chat/completions` where endpoint is typically `.../v1`.
/// Ollama uses the same response shape via `{host}/v1/chat/completions`; accept either
/// `http://host:port` or `http://host:port/v1` as the configured base.
fn chat_completions_url(endpoint: &str, provider: &LlmProvider) -> String {
    let base = endpoint.trim_end_matches('/');
    match provider {
        LlmProvider::OpenAI => format!("{base}/chat/completions"),
        LlmProvider::Ollama => {
            let root = if base.ends_with("/v1") {
                base.to_string()
            } else {
                format!("{base}/v1")
            };
            format!("{root}/chat/completions")
        }
    }
}

fn build_questions_prompt(
    context: &str,
    style: &QuestionStyle,
    count: usize,
    book_title: Option<&str>,
    book_author: Option<&str>,
    metadata: Option<&BookMetadata>,
) -> String {
    let style_instruction = match style {
        QuestionStyle::MultipleChoice => {
            "Generate ONLY multiple-choice questions. Each question must have exactly 4 options labeled A, B, C, D. Mark the correct answer."
        }
        QuestionStyle::OpenEnded => {
            "Generate ONLY open-ended questions that require a short written answer."
        }
        QuestionStyle::Mixed => {
            "Generate a mix of multiple-choice and open-ended questions. For multiple-choice, provide 4 options (A, B, C, D) and mark the correct one."
        }
    };

    let title_line = book_title.unwrap_or("Unknown");
    let author_line = book_author.unwrap_or("Unknown");
    let metadata_block = if let Some(meta) = metadata {
        let themes = if meta.themes.is_empty() {
            "Unknown".to_string()
        } else {
            meta.themes.join(", ")
        };
        let characters = if meta.key_characters.is_empty() {
            "Unknown".to_string()
        } else {
            meta.key_characters.join(", ")
        };

        format!(
            r#"BOOK CONTEXT:
Title: {title_line}
Author: {author_line}
Genre: {}
Setting: {}
Year Written: {}
Summary: {}
Themes: {themes}
Key Characters: {characters}
Notable Context: {}
"#,
            meta.genre,
            meta.setting,
            meta.year_written,
            meta.summary,
            meta.notable_context,
        )
    } else {
        format!(
            r#"BOOK CONTEXT:
Title: {title_line}
Author: {author_line}
"#
        )
    };

    format!(
        r#"You are a reading comprehension assistant. Use the book context and excerpt below to generate {count} comprehension questions.

{metadata_block}
Use the book context to ask nuanced and relevant questions about the excerpt.

{style_instruction}

Respond with ONLY a JSON array in this exact format (no other text or formatting):
[
  {{
    "question": "Question text here",
    "question_type": "MultipleChoice",
    "options": ["Option A", "Option B", "Option C", "Option D"],
    "correct_answer": "Option A"
  }},
  {{
    "question": "Open ended question here",
    "question_type": "OpenEnded",
    "options": null,
    "correct_answer": "Expected answer or key points"
  }}
]

TEXT EXCERPT:
{context}

Generate the JSON array now:"#
    )
}

fn build_evaluation_prompt(question: &str, expected: &str, user_answer: &str) -> String {
    format!(
        r#"Evaluate this reading comprehension answer:

Question: {question}
Expected Answer/Key Points: {expected}
User's Answer: {user_answer}

Respond with ONLY a JSON object:
{{
  "is_correct": true or false,
  "feedback": "Brief, encouraging feedback (1-2 sentences)",
  "correct_answer": "The correct answer if the user was wrong, otherwise null"
}}"#
    )
}

fn build_book_metadata_prompt(
    text_excerpt: &str,
    title: &str,
    author: Option<&str>,
) -> String {
    let author_line = author.unwrap_or("Unknown");
    format!(
        r#"You are a literary analysis assistant. Infer structured metadata for a book from known details and an excerpt.

Known details:
Title: {title}
Author: {author_line}

Respond with ONLY a JSON object in this exact shape:
{{
  "genre": "Primary genre and subgenre if known",
  "year_written": "Estimated or known year/time period the book was written",
  "summary": "2-3 sentence high-level summary of what the book is about",
  "themes": ["theme 1", "theme 2", "theme 3"],
  "setting": "Primary setting (time/place)",
  "key_characters": ["Character 1: brief role", "Character 2: brief role"],
  "notable_context": "Important background for comprehension questions"
}}

Rules:
- If uncertain, provide best-effort estimates and say \"Unknown\" only when necessary.
- Keep summary concise and factual.
- Ensure valid JSON.

BOOK EXCERPT:
{text_excerpt}"#
    )
}

async fn call_llm(
    settings: &Settings,
    prompt: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let url = chat_completions_url(&settings.llm_endpoint, &settings.llm_provider);

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: prompt,
    }];

    let request = ChatRequest {
        model: settings.llm_model.clone(),
        messages,
        temperature: 0.7,
    };

    let mut req_builder = client.post(&url).json(&request);

    if let Some(api_key) = &settings.llm_api_key {
        if !api_key.is_empty() {
            req_builder = req_builder.bearer_auth(api_key);
        }
    }

    let response = req_builder
        .send()
        .await
        .map_err(|e| format!("LLM request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("LLM API error {status}: {body}"));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {e}"))?;

    chat_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from LLM".to_string())
}

#[tauri::command]
pub async fn generate_questions(
    settings: Settings,
    context_text: String,
    question_count: usize,
    book_title: Option<String>,
    book_author: Option<String>,
    book_metadata: Option<BookMetadata>,
) -> Result<Vec<LlmQuestion>, String> {
    let count = question_count.clamp(1, 5);
    let prompt = build_questions_prompt(
        &context_text,
        &settings.question_style,
        count,
        book_title.as_deref(),
        book_author.as_deref(),
        book_metadata.as_ref(),
    );

    let response = call_llm(&settings, prompt).await?;

    // Try to extract JSON array from the response
    let json_str = extract_json_array(&response)
        .ok_or_else(|| format!("Could not find JSON in LLM response: {response}"))?;

    serde_json::from_str::<Vec<LlmQuestion>>(&json_str)
        .map_err(|e| format!("Failed to parse questions JSON: {e}\nRaw: {json_str}"))
}

#[tauri::command]
pub async fn generate_book_metadata(
    settings: Settings,
    text_excerpt: String,
    book_title: String,
    book_author: Option<String>,
) -> Result<BookMetadata, String> {
    let prompt = build_book_metadata_prompt(&text_excerpt, &book_title, book_author.as_deref());
    let response = call_llm(&settings, prompt).await?;

    let json_str = extract_json_object(&response)
        .ok_or_else(|| format!("Could not find JSON in LLM response: {response}"))?;

    serde_json::from_str::<BookMetadata>(&json_str)
        .map_err(|e| format!("Failed to parse metadata JSON: {e}\nRaw: {json_str}"))
}

#[tauri::command]
pub async fn evaluate_answer(
    settings: Settings,
    question: String,
    expected_answer: String,
    user_answer: String,
) -> Result<AnswerEvaluation, String> {
    let prompt = build_evaluation_prompt(&question, &expected_answer, &user_answer);

    let response = call_llm(&settings, prompt).await?;

    let json_str = extract_json_object(&response)
        .ok_or_else(|| format!("Could not find JSON in LLM response: {response}"))?;

    serde_json::from_str::<AnswerEvaluation>(&json_str)
        .map_err(|e| format!("Failed to parse evaluation JSON: {e}\nRaw: {json_str}"))
}

fn extract_json_array(text: &str) -> Option<String> {
    let start = text.find('[')?;
    let end = text.rfind(']')?;
    if end >= start {
        Some(text[start..=end].to_string())
    } else {
        None
    }
}

fn extract_json_object(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end >= start {
        Some(text[start..=end].to_string())
    } else {
        None
    }
}
