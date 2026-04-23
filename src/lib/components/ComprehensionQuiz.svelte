<script lang="ts">
  import { readerStore } from '../stores/reader';
  import { settingsStore } from '../stores/settings';
  import { libraryStore } from '../stores/library';
  import { invoke } from '@tauri-apps/api/core';
  import type { LlmQuestion, AnswerEvaluation } from '../types';

  let loading = false;
  let questions: LlmQuestion[] = [];
  let currentQuestionIdx = 0;
  let selectedOption: string | null = null;
  let openAnswer = '';
  let evaluation: AnswerEvaluation | null = null;
  let totalCorrect = 0;
  let quizDone = false;
  let error: string | null = null;
  let breakWasActive = false;

  // Detect when a break begins and trigger question loading
  $: {
    const isOnBreak = $readerStore.isOnBreak;
    const wantsQuestions = $settingsStore.comprehension_questions_enabled;
    if (isOnBreak && wantsQuestions && !breakWasActive && questions.length === 0 && !loading) {
      breakWasActive = true;
      loadQuestions();
    }
    if (!isOnBreak) {
      breakWasActive = false;
    }
  }

  async function loadQuestions() {
    loading = true;
    error = null;
    try {
      const rs = $readerStore;
      const maxWords = Math.min(
        2000,
        Math.max(50, $settingsStore.quiz_context_max_words),
      );
      const sinceBreakStart = rs.lastBreakWordIndex;
      const maxStart = rs.currentIndex - maxWords;
      const start = Math.max(0, sinceBreakStart, maxStart);
      const contextWords = rs.words.slice(start, rs.currentIndex);
      const contextText = contextWords.join(' ');
      const currentBook = $libraryStore.entries.find((e) => e.book.id === rs.bookId)?.book ?? null;

      questions = await invoke<LlmQuestion[]>('generate_questions', {
        settings: $settingsStore,
        contextText,
        questionCount: 3,
        bookTitle: currentBook?.title ?? null,
        bookAuthor: currentBook?.author ?? null,
        bookMetadata: currentBook?.metadata ?? null,
      });

      currentQuestionIdx = 0;
      selectedOption = null;
      openAnswer = '';
      evaluation = null;
      quizDone = false;
      totalCorrect = 0;
    } catch (e) {
      error = String(e);
      questions = [];
    } finally {
      loading = false;
    }
  }

  async function submitAnswer() {
    const q = questions[currentQuestionIdx];
    if (!q) return;

    if (q.question_type === 'MultipleChoice') {
      if (!selectedOption) return;
      evaluation = {
        is_correct: selectedOption === q.correct_answer,
        feedback:
          selectedOption === q.correct_answer
            ? 'Correct! Well done.'
            : `Not quite. The correct answer was: ${q.correct_answer}`,
        correct_answer: selectedOption !== q.correct_answer ? q.correct_answer ?? null : null,
      };
      if (evaluation.is_correct) totalCorrect++;
    } else {
      if (!openAnswer.trim()) return;
      loading = true;
      try {
        evaluation = await invoke<AnswerEvaluation>('evaluate_answer', {
          settings: $settingsStore,
          question: q.question,
          expectedAnswer: q.correct_answer ?? '',
          userAnswer: openAnswer,
        });
        if (evaluation.is_correct) totalCorrect++;
      } catch (e) {
        evaluation = {
          is_correct: false,
          feedback: 'Could not evaluate answer. ' + String(e),
          correct_answer: null,
        };
      } finally {
        loading = false;
      }
    }
  }

  function nextQuestion() {
    if (currentQuestionIdx + 1 >= questions.length) {
      quizDone = true;
    } else {
      currentQuestionIdx++;
      selectedOption = null;
      openAnswer = '';
      evaluation = null;
    }
  }

  function continueReading() {
    const score = questions.length > 0 ? totalCorrect / questions.length : null;
    questions = [];
    quizDone = false;
    readerStore.dismissBreak(score);
  }

  function skipQuiz() {
    questions = [];
    quizDone = false;
    readerStore.dismissBreak(null);
  }
</script>

{#if $readerStore.isOnBreak}
  <div class="break-overlay">
    <div class="break-card">
      <div class="break-header">
        <h2 class="break-title">Time for a Break</h2>
        <p class="break-subtitle">
          You've been reading for a while. Take a moment to rest.
        </p>
      </div>

      {#if !$settingsStore.comprehension_questions_enabled}
        <div class="no-quiz">
          <p>Ready to continue?</p>
          <button class="btn-primary" onclick={continueReading}>Continue Reading</button>
        </div>
      {:else if loading}
        <div class="loading">
          <div class="spinner"></div>
          <p>Generating comprehension questions…</p>
        </div>
      {:else if error}
        <div class="error-block">
          <p class="error-text">Could not load questions: {error}</p>
          <div class="error-actions">
            <button class="btn-secondary" onclick={loadQuestions}>Retry</button>
            <button class="btn-primary" onclick={continueReading}>Skip Questions</button>
          </div>
        </div>
      {:else if quizDone}
        <div class="quiz-done">
          <div class="score-display">
            <span class="score-number">{totalCorrect}/{questions.length}</span>
            <span class="score-label">Correct</span>
          </div>
          <p class="score-message">
            {totalCorrect === questions.length
              ? 'Perfect score! Excellent comprehension.'
              : totalCorrect >= questions.length / 2
              ? 'Good work! Keep it up.'
              : 'Keep reading — comprehension improves with practice.'}
          </p>
          <button class="btn-primary" onclick={continueReading}>Continue Reading</button>
        </div>
      {:else if questions.length > 0}
        {@const q = questions[currentQuestionIdx]}
        <div class="question-block">
          <div class="question-progress">
            Question {currentQuestionIdx + 1} of {questions.length}
          </div>
          <p class="question-text">{q.question}</p>

          {#if q.question_type === 'MultipleChoice' && q.options}
            <div class="options-list">
              {#each q.options as option}
                <label
                  class="option-label"
                  class:selected={selectedOption === option}
                  class:correct={evaluation !== null && option === q.correct_answer}
                  class:wrong={evaluation !== null && selectedOption === option && !evaluation.is_correct}
                >
                  <input
                    type="radio"
                    name="mc"
                    value={option}
                    bind:group={selectedOption}
                    disabled={evaluation !== null}
                  />
                  <span>{option}</span>
                </label>
              {/each}
            </div>
          {:else}
            <textarea
              class="open-answer"
              placeholder="Type your answer…"
              bind:value={openAnswer}
              disabled={evaluation !== null}
              rows="3"
            ></textarea>
          {/if}

          {#if evaluation}
            <div class="evaluation" class:correct={evaluation.is_correct} class:wrong={!evaluation.is_correct}>
              <span class="eval-icon">{evaluation.is_correct ? '✓' : '✗'}</span>
              <span class="eval-feedback">{evaluation.feedback}</span>
            </div>
            <div class="question-actions">
              {#if currentQuestionIdx + 1 < questions.length}
                <button class="btn-primary" onclick={nextQuestion}>Next Question</button>
              {:else}
                <button class="btn-primary" onclick={() => (quizDone = true)}>See Results</button>
              {/if}
            </div>
          {:else}
            <div class="question-actions">
              <button
                class="btn-secondary"
                onclick={skipQuiz}
              >Skip Quiz</button>
              <button
                class="btn-primary"
                onclick={submitAnswer}
                disabled={loading || (q.question_type === 'MultipleChoice' ? !selectedOption : !openAnswer.trim())}
              >
                {loading ? 'Evaluating…' : 'Submit Answer'}
              </button>
            </div>
          {/if}
        </div>
      {:else}
        <div class="no-quiz">
          <button class="btn-primary" onclick={continueReading}>Continue Reading</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .break-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .break-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 36px;
    width: 90%;
    max-width: 560px;
    font-family: var(--font-family);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  }

  .break-header {
    margin-bottom: 24px;
    text-align: center;
  }

  .break-title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
    margin: 0 0 6px 0;
  }

  .break-subtitle {
    font-size: 14px;
    color: var(--muted);
    margin: 0;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 24px 0;
    color: var(--muted);
    font-size: 14px;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .question-progress {
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 10px;
  }

  .question-text {
    font-size: 16px;
    color: var(--text);
    line-height: 1.5;
    margin: 0 0 20px 0;
    font-weight: 500;
  }

  .options-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 16px;
  }

  .option-label {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 14px;
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.12s;
    color: var(--text);
    font-size: 14px;
  }

  .option-label:hover {
    background: var(--border);
  }

  .option-label.selected {
    border-color: var(--accent);
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }

  .option-label.correct {
    border-color: #4caf50;
    background: color-mix(in srgb, #4caf50 12%, transparent);
    color: #4caf50;
  }

  .option-label.wrong {
    border-color: #f44336;
    background: color-mix(in srgb, #f44336 12%, transparent);
    color: #f44336;
  }

  .open-answer {
    width: 100%;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text);
    font-family: var(--font-family);
    font-size: 14px;
    padding: 10px 12px;
    resize: vertical;
    margin-bottom: 16px;
    box-sizing: border-box;
  }

  .evaluation {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: 8px;
    margin-bottom: 16px;
    font-size: 14px;
  }

  .evaluation.correct {
    background: color-mix(in srgb, #4caf50 12%, transparent);
    border: 1px solid #4caf50;
    color: #4caf50;
  }

  .evaluation.wrong {
    background: color-mix(in srgb, #f44336 12%, transparent);
    border: 1px solid #f44336;
    color: #f44336;
  }

  .eval-icon {
    font-weight: 700;
    font-size: 16px;
    flex-shrink: 0;
  }

  .eval-feedback {
    line-height: 1.4;
    color: var(--text);
  }

  .question-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }

  .score-display {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 12px;
  }

  .score-number {
    font-size: 48px;
    font-weight: 800;
    color: var(--accent);
  }

  .score-label {
    font-size: 13px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .score-message {
    text-align: center;
    font-size: 14px;
    color: var(--text);
    margin-bottom: 24px;
  }

  .no-quiz {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 16px 0;
    color: var(--text);
    font-size: 14px;
  }

  .error-block {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .error-text {
    color: #f44336;
    font-size: 13px;
  }

  .error-actions {
    display: flex;
    gap: 10px;
    justify-content: flex-end;
  }

  .btn-primary {
    background: var(--accent);
    color: var(--bg);
    border: none;
    border-radius: 8px;
    padding: 10px 20px;
    font-size: 14px;
    font-weight: 600;
    font-family: var(--font-family);
    cursor: pointer;
    transition: filter 0.15s;
  }

  .btn-primary:hover:not(:disabled) {
    filter: brightness(1.15);
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 10px 20px;
    font-size: 14px;
    font-family: var(--font-family);
    cursor: pointer;
    transition: background 0.15s;
  }

  .btn-secondary:hover {
    background: var(--border);
  }
</style>
