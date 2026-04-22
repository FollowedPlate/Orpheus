use crate::models::{
    AnswerEvaluation, ChatMessage, ChatRequest, ChatResponse, LlmProvider, LlmQuestion,
    QuestionStyle, Settings,
};

fn build_questions_prompt(context: &str, style: &QuestionStyle, count: usize) -> String {
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

    format!(
        r#"You are a reading comprehension assistant. Based on the following text excerpt, generate {count} comprehension questions.

{style_instruction}

Respond with ONLY a JSON array in this exact format:
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

async fn call_llm(
    settings: &Settings,
    prompt: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    let url = match settings.llm_provider {
        LlmProvider::OpenAI => format!("{}/chat/completions", settings.llm_endpoint),
        LlmProvider::Ollama => format!("{}/api/chat", settings.llm_endpoint),
    };

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
) -> Result<Vec<LlmQuestion>, String> {
    let count = question_count.clamp(1, 5);
    let prompt = build_questions_prompt(&context_text, &settings.question_style, count);

    let response = call_llm(&settings, prompt).await?;

    // Try to extract JSON array from the response
    let json_str = extract_json_array(&response)
        .ok_or_else(|| format!("Could not find JSON in LLM response: {response}"))?;

    serde_json::from_str::<Vec<LlmQuestion>>(&json_str)
        .map_err(|e| format!("Failed to parse questions JSON: {e}\nRaw: {json_str}"))
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
