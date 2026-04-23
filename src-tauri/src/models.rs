use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub file_path: String,
    pub file_format: String,
    pub word_count: usize,
    pub added_at: DateTime<Utc>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub metadata: Option<BookMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub genre: String,
    pub year_written: String,
    pub summary: String,
    pub themes: Vec<String>,
    pub setting: String,
    pub key_characters: Vec<String>,
    pub notable_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub book: Book,
    pub current_word_index: usize,
    pub progress: f64,
    pub sessions: Vec<ReadingSession>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingSession {
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub words_read: usize,
    pub average_wpm: f64,
    pub quiz_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedBook {
    pub title: String,
    pub author: Option<String>,
    pub words: Vec<String>,
    pub chapter_indices: Vec<usize>,
    pub paragraph_indices: Vec<usize>,
    pub sentence_indices: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrpAlgorithm {
    Spritz,
    Center,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum AdjacentWordsCount {
    #[serde(rename = "1")]
    N1,
    #[serde(rename = "2")]
    N2,
    #[serde(rename = "3")]
    N3,
    #[serde(rename = "dynamic")]
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemePreset {
    Light,
    Dark,
    Sepia,
    HighContrast,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LlmProvider {
    OpenAI,
    Ollama,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestionStyle {
    MultipleChoice,
    OpenEnded,
    Mixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutMap {
    pub play_pause: String,
    pub skip_forward_sentence: String,
    pub skip_back_sentence: String,
    pub skip_forward_paragraph: String,
    pub skip_back_paragraph: String,
    pub increase_wpm: String,
    pub decrease_wpm: String,
    pub toggle_focus: String,
    pub open_settings: String,
    pub return_to_library: String,
}

impl Default for ShortcutMap {
    fn default() -> Self {
        ShortcutMap {
            play_pause: "Space".to_string(),
            skip_forward_sentence: "ArrowRight".to_string(),
            skip_back_sentence: "ArrowLeft".to_string(),
            skip_forward_paragraph: "Ctrl+ArrowRight".to_string(),
            skip_back_paragraph: "Ctrl+ArrowLeft".to_string(),
            increase_wpm: "ArrowUp".to_string(),
            decrease_wpm: "ArrowDown".to_string(),
            toggle_focus: "KeyF".to_string(),
            open_settings: "Ctrl+Comma".to_string(),
            return_to_library: "Escape".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub font: String,
    pub font_size: u32,
    pub orp_algorithm: OrpAlgorithm,
    pub orp_color: String,
    pub orp_bold: bool,
    pub orp_underline: bool,
    pub orp_size_multiplier: f64,
    pub wpm: u32,
    pub wpm_step: u32,
    pub sentence_end_multiplier: f64,
    pub clause_multiplier: f64,
    pub paragraph_multiplier: f64,
    pub long_word_threshold: usize,
    pub long_word_multiplier: f64,
    pub speed_ramp_enabled: bool,
    pub speed_ramp_start_wpm: u32,
    pub speed_ramp_duration_secs: u32,
    pub word_grouping_enabled: bool,
    pub adjacent_words_enabled: bool,
    pub adjacent_words_count: AdjacentWordsCount,
    pub skip_context_enabled: bool,
    pub focus_mode: bool,
    pub break_interval_minutes: u32,
    pub comprehension_questions_enabled: bool,
    pub theme_preset: ThemePreset,
    pub custom_bg_color: Option<String>,
    pub custom_text_color: Option<String>,
    pub custom_orp_color: Option<String>,
    pub custom_accent_color: Option<String>,
    pub custom_guide_color: Option<String>,
    pub llm_provider: LlmProvider,
    pub llm_endpoint: String,
    pub llm_api_key: Option<String>,
    pub llm_model: String,
    /// Max words of read text sent to the LLM per break (from since-last-break window).
    pub quiz_context_max_words: u32,
    /// Max words from the beginning of a book sent to the LLM for metadata generation.
    pub metadata_context_max_words: u32,
    pub question_style: QuestionStyle,
    pub shortcuts: ShortcutMap,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            font: "Inter".to_string(),
            font_size: 32,
            orp_algorithm: OrpAlgorithm::Spritz,
            orp_color: "#FF4444".to_string(),
            orp_bold: true,
            orp_underline: false,
            orp_size_multiplier: 1.0,
            wpm: 300,
            wpm_step: 25,
            sentence_end_multiplier: 2.5,
            clause_multiplier: 1.5,
            paragraph_multiplier: 3.0,
            long_word_threshold: 8,
            long_word_multiplier: 1.3,
            speed_ramp_enabled: false,
            speed_ramp_start_wpm: 150,
            speed_ramp_duration_secs: 30,
            word_grouping_enabled: false,
            adjacent_words_enabled: false,
            adjacent_words_count: AdjacentWordsCount::N3,
            skip_context_enabled: true,
            focus_mode: false,
            break_interval_minutes: 0,
            comprehension_questions_enabled: false,
            theme_preset: ThemePreset::Dark,
            custom_bg_color: None,
            custom_text_color: None,
            custom_orp_color: None,
            custom_accent_color: None,
            custom_guide_color: None,
            llm_provider: LlmProvider::OpenAI,
            llm_endpoint: "https://api.openai.com/v1".to_string(),
            llm_api_key: None,
            llm_model: "gpt-4o-mini".to_string(),
            quiz_context_max_words: 300,
            metadata_context_max_words: 2000,
            question_style: QuestionStyle::Mixed,
            shortcuts: ShortcutMap::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub entries: Vec<LibraryEntry>,
}

impl Default for Library {
    fn default() -> Self {
        Library { entries: vec![] }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestionType {
    MultipleChoice,
    OpenEnded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmQuestion {
    pub question: String,
    pub question_type: QuestionType,
    pub options: Option<Vec<String>>,
    pub correct_answer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub questions: Vec<LlmQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerEvaluation {
    pub is_correct: bool,
    pub feedback: String,
    pub correct_answer: Option<String>,
}

// OpenAI API request/response types
#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f64,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatMessageResponse,
}

#[derive(Debug, Deserialize)]
pub struct ChatMessageResponse {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub book_id: String,
    pub current_word_index: usize,
    pub words_read: usize,
    pub average_wpm: f64,
    pub quiz_score: Option<f64>,
    pub session_ended: bool,
}
