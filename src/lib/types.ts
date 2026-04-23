export type OrpAlgorithm = 'Spritz' | 'Center' | 'Dynamic';
export type AdjacentWordsCount = 1 | 2 | 3 | 'dynamic';
export type ThemePreset = 'Light' | 'Dark' | 'Sepia' | 'HighContrast' | 'Custom';
export type LlmProvider = 'OpenAI' | 'Ollama';
export type QuestionStyle = 'MultipleChoice' | 'OpenEnded' | 'Mixed';
export type QuestionType = 'MultipleChoice' | 'OpenEnded';
export type AppView = 'library' | 'reader' | 'book_detail';

export interface ShortcutMap {
  play_pause: string;
  skip_forward_sentence: string;
  skip_back_sentence: string;
  skip_forward_paragraph: string;
  skip_back_paragraph: string;
  increase_wpm: string;
  decrease_wpm: string;
  toggle_focus: string;
  open_settings: string;
  return_to_library: string;
}

export interface Settings {
  font: string;
  font_size: number;
  orp_algorithm: OrpAlgorithm;
  orp_color: string;
  orp_bold: boolean;
  orp_underline: boolean;
  orp_size_multiplier: number;
  wpm: number;
  wpm_step: number;
  sentence_end_multiplier: number;
  clause_multiplier: number;
  paragraph_multiplier: number;
  long_word_threshold: number;
  long_word_multiplier: number;
  speed_ramp_enabled: boolean;
  speed_ramp_start_wpm: number;
  speed_ramp_duration_secs: number;
  word_grouping_enabled: boolean;
  adjacent_words_enabled: boolean;
  adjacent_words_count: AdjacentWordsCount;
  skip_context_enabled: boolean;
  focus_mode: boolean;
  break_interval_minutes: number;
  comprehension_questions_enabled: boolean;
  theme_preset: ThemePreset;
  custom_bg_color: string | null;
  custom_text_color: string | null;
  custom_orp_color: string | null;
  custom_accent_color: string | null;
  custom_guide_color: string | null;
  llm_provider: LlmProvider;
  llm_endpoint: string;
  llm_api_key: string | null;
  llm_model: string;
  /** Max words of read text sent to the LLM per break (from since-last-break window). */
  quiz_context_max_words: number;
  /** Max words from the beginning of a book sent to the LLM for metadata generation. */
  metadata_context_max_words: number;
  question_style: QuestionStyle;
  shortcuts: ShortcutMap;
}

export interface BookMetadata {
  genre: string;
  year_written: string;
  summary: string;
  themes: string[];
  setting: string;
  key_characters: string[];
  notable_context: string;
}

export interface Book {
  id: string;
  title: string;
  author: string | null;
  file_path: string;
  file_format: string;
  word_count: number;
  added_at: string;
  last_read_at: string | null;
  metadata: BookMetadata | null;
}

export interface ReadingSession {
  started_at: string;
  ended_at: string | null;
  words_read: number;
  average_wpm: number;
  quiz_score: number | null;
}

export interface LibraryEntry {
  book: Book;
  current_word_index: number;
  progress: number;
  sessions: ReadingSession[];
}

export interface Library {
  entries: LibraryEntry[];
}

export interface ParsedBook {
  title: string;
  author: string | null;
  words: string[];
  chapter_indices: number[];
  paragraph_indices: number[];
  sentence_indices: number[];
}

export interface LlmQuestion {
  question: string;
  question_type: QuestionType;
  options: string[] | null;
  correct_answer: string | null;
}

export interface AnswerEvaluation {
  is_correct: boolean;
  feedback: string;
  correct_answer: string | null;
}

export const DEFAULT_SHORTCUTS: ShortcutMap = {
  play_pause: 'Space',
  skip_forward_sentence: 'ArrowRight',
  skip_back_sentence: 'ArrowLeft',
  skip_forward_paragraph: 'Ctrl+ArrowRight',
  skip_back_paragraph: 'Ctrl+ArrowLeft',
  increase_wpm: 'ArrowUp',
  decrease_wpm: 'ArrowDown',
  toggle_focus: 'KeyF',
  open_settings: 'Ctrl+Comma',
  return_to_library: 'Escape',
};

export const DEFAULT_SETTINGS: Settings = {
  font: 'Inter',
  font_size: 32,
  orp_algorithm: 'Spritz',
  orp_color: '#FF4444',
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
  adjacent_words_count: 3,
  skip_context_enabled: true,
  focus_mode: false,
  break_interval_minutes: 0,
  comprehension_questions_enabled: false,
  theme_preset: 'Dark',
  custom_bg_color: null,
  custom_text_color: null,
  custom_orp_color: null,
  custom_accent_color: null,
  custom_guide_color: null,
  llm_provider: 'OpenAI',
  llm_endpoint: 'https://api.openai.com/v1',
  llm_api_key: null,
  llm_model: 'gpt-4o-mini',
  quiz_context_max_words: 300,
  metadata_context_max_words: 2000,
  question_style: 'Mixed',
  shortcuts: DEFAULT_SHORTCUTS,
};

export const AVAILABLE_FONTS = [
  'Inter',
  'Source Sans 3',
  'Literata',
  'JetBrains Mono',
  'Atkinson Hyperlegible',
] as const;

export type AvailableFont = (typeof AVAILABLE_FONTS)[number];
