import { writable, get } from 'svelte/store';
import { settingsStore } from './settings';
import { libraryStore } from './library';
import { getWordDelay, getRampedWpm } from '../utils/timing';
import { buildWordGroup } from '../utils/wordGrouping';
import type { ParsedBook, AppView } from '../types';

export interface ReaderState {
  // Book data
  words: string[];
  paragraphIndices: Set<number>;
  sentenceIndices: Set<number>;
  chapterIndices: Set<number>;
  totalWords: number;

  // Playback position
  currentIndex: number;
  currentDisplay: string;  // may be grouped words

  // Playback control
  isPlaying: boolean;
  targetWpm: number;
  currentWpm: number;

  // Ramp-up tracking
  rampStartTime: number | null;
  rampStartIndex: number;

  // Session tracking
  bookId: string;
  sessionStartTime: number | null;
  sessionWordsRead: number;
  playingMs: number;

  // Skip context: true briefly after a skip to trigger full-width adjacent words
  isSkipContext: boolean;

  // App navigation
  view: AppView;
  selectedBookId: string | null;
  showSettings: boolean;
  showStats: boolean;

  // Break/quiz state
  isOnBreak: boolean;
  lastBreakMs: number;
  /** First word index included in "read since last break" for LLM quiz context. */
  lastBreakWordIndex: number;
}

const INITIAL_STATE: ReaderState = {
  words: [],
  paragraphIndices: new Set(),
  sentenceIndices: new Set(),
  chapterIndices: new Set(),
  totalWords: 0,
  currentIndex: 0,
  currentDisplay: '',
  isPlaying: false,
  targetWpm: 300,
  currentWpm: 300,
  rampStartTime: null,
  rampStartIndex: 0,
  bookId: '',
  sessionStartTime: null,
  sessionWordsRead: 0,
  playingMs: 0,
  isSkipContext: false,
  view: 'library',
  selectedBookId: null,
  showSettings: false,
  showStats: false,
  isOnBreak: false,
  lastBreakMs: 0,
  lastBreakWordIndex: 0,
};

function createReaderStore() {
  const { subscribe, set, update } = writable<ReaderState>({ ...INITIAL_STATE });

  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  let skipContextTimeoutId: ReturnType<typeof setTimeout> | null = null;
  let animationFrameId: number | null = null;
  let animationToken = 0;
  let progressSaveInterval: ReturnType<typeof setInterval> | null = null;
  let playbackStartTime: number | null = null;
  let accumulatedPlayMs = 0;

  function cancelAnimation() {
    animationToken += 1;
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }
  }

  function clearTimers() {
    cancelAnimation();
    if (timeoutId) {
      clearTimeout(timeoutId);
      timeoutId = null;
    }
    if (skipContextTimeoutId) {
      clearTimeout(skipContextTimeoutId);
      skipContextTimeoutId = null;
    }
  }

  function getState(): ReaderState {
    return get({ subscribe });
  }

  function getSettings() {
    return get(settingsStore);
  }

  function currentPlayingMs(): number {
    if (playbackStartTime !== null) {
      return accumulatedPlayMs + (Date.now() - playbackStartTime);
    }
    return accumulatedPlayMs;
  }

  function scheduleNext() {
    const state = getState();
    if (!state.isPlaying) return;

    const settings = getSettings();
    const paraArray = Array.from(state.paragraphIndices);

    // Get the word group at the current index
    const group = buildWordGroup(
      state.words,
      state.currentIndex,
      settings.word_grouping_enabled,
    );

    // Update the display text
    update((s) => ({ ...s, currentDisplay: group.displayText }));

    // Compute timing for this word (use last word of group for punctuation check)
    const lastWordInGroup = state.words[group.endIndex] ?? '';

    // Handle ramp-up
    let currentWpm = state.targetWpm;
    if (settings.speed_ramp_enabled && state.rampStartTime !== null) {
      const elapsed = Date.now() - state.rampStartTime;
      currentWpm = getRampedWpm(
        state.targetWpm,
        settings.speed_ramp_start_wpm,
        settings.speed_ramp_duration_secs * 1000,
        elapsed,
      );
      update((s) => ({ ...s, currentWpm }));
    }

    const delay = getWordDelay(
      lastWordInGroup,
      group.endIndex,
      paraArray,
      settings,
      currentWpm,
    );

    // Schedule next word
    const nextIndex = group.endIndex + 1;

    timeoutId = setTimeout(() => {
      update((s) => {
        if (!s.isPlaying) return s;

        // Reached end of book
        if (nextIndex >= s.words.length) {
          saveProgress(s.currentIndex, true);
          return { ...s, isPlaying: false, currentIndex: 0 };
        }

        const newSessionWords = s.sessionWordsRead + (group.endIndex - s.currentIndex + 1);

        // Check for break
        const playMs = currentPlayingMs();
        const settings2 = getSettings();
        if (
          settings2.break_interval_minutes > 0 &&
          playMs - s.lastBreakMs >= settings2.break_interval_minutes * 60_000
        ) {
          // Trigger break
          if (progressSaveInterval) clearInterval(progressSaveInterval);
          if (playbackStartTime !== null) {
            accumulatedPlayMs += Date.now() - playbackStartTime;
            playbackStartTime = null;
          }
          return {
            ...s,
            isPlaying: false,
            currentIndex: nextIndex,
            sessionWordsRead: newSessionWords,
            isOnBreak: true,
            lastBreakMs: playMs,
          };
        }

        return {
          ...s,
          currentIndex: nextIndex,
          sessionWordsRead: newSessionWords,
        };
      });

      const newState = getState();
      if (newState.isPlaying) {
        scheduleNext();
      } else if (newState.isOnBreak) {
        // Break triggered — stop saving interval
      }
    }, delay);
  }

  function saveProgress(currentIndex: number, sessionEnded: boolean) {
    const state = getState();
    if (!state.bookId) return;

    const playMs = currentPlayingMs();
    const minutesPlayed = playMs / 60_000;
    const avgWpm = minutesPlayed > 0 ? state.sessionWordsRead / minutesPlayed : 0;

    libraryStore.updateProgress(
      state.bookId,
      currentIndex,
      state.sessionWordsRead,
      Math.round(avgWpm),
      null,
      sessionEnded,
    );
  }

  function animateToIndex(
    targetIndex: number,
    showSkipContext: boolean,
    onDone?: () => void,
  ) {
    const state = getState();
    const startIndex = state.currentIndex;
    const distance = targetIndex - startIndex;
    if (distance === 0) {
      onDone?.();
      return;
    }

    cancelAnimation();
    const token = animationToken;
    const settings = getSettings();
    const startTs = performance.now();
    const durationMs = Math.max(100, Math.min(1200, settings.skip_animation_duration_ms));

    const step = (now: number) => {
      if (token !== animationToken) return;

      const t = Math.min(1, (now - startTs) / durationMs);
      const eased = t < 0.5
        ? 4 * t ** 3
        : 1 - ((-2 * t + 2) ** 3) / 2;
      const nextIndex = Math.round(startIndex + distance * eased);
      const clamped = Math.max(0, Math.min(nextIndex, Math.max(state.words.length - 1, 0)));

      update((s) => ({
        ...s,
        currentIndex: clamped,
        currentDisplay: s.words[clamped] ?? '',
        isSkipContext: showSkipContext && settings.skip_context_enabled,
      }));

      if (t < 1) {
        animationFrameId = requestAnimationFrame(step);
      } else {
        animationFrameId = null;
        onDone?.();
      }
    };

    animationFrameId = requestAnimationFrame(step);
  }

  return {
    subscribe,

    loadBook(book: ParsedBook, bookId: string, startIndex = 0) {
      clearTimers();
      if (progressSaveInterval) clearInterval(progressSaveInterval);
      accumulatedPlayMs = 0;
      playbackStartTime = null;

      const settings = getSettings();
      const sortedParagraphIndices = [...book.paragraph_indices].sort((a, b) => a - b);
      const sortedSentenceIndices = [...book.sentence_indices].sort((a, b) => a - b);
      const sortedChapterIndices = [...book.chapter_indices].sort((a, b) => a - b);

      update(() => ({
        ...INITIAL_STATE,
        words: book.words,
        paragraphIndices: new Set(sortedParagraphIndices),
        sentenceIndices: new Set(sortedSentenceIndices),
        chapterIndices: new Set(sortedChapterIndices),
        totalWords: book.words.length,
        currentIndex: startIndex,
        currentDisplay: book.words[startIndex] ?? '',
        targetWpm: settings.wpm,
        currentWpm: settings.speed_ramp_enabled ? settings.speed_ramp_start_wpm : settings.wpm,
        bookId,
        view: 'reader',
        lastBreakWordIndex: startIndex,
      }));
    },

    play() {
      cancelAnimation();
      const state = getState();
      if (state.isPlaying || state.words.length === 0) return;

      const settings = getSettings();

      update((s) => {
        const lastBreakWordIndex =
          s.currentIndex < s.lastBreakWordIndex ? s.currentIndex : s.lastBreakWordIndex;
        return {
          ...s,
          isPlaying: true,
          sessionStartTime: s.sessionStartTime ?? Date.now(),
          rampStartTime: settings.speed_ramp_enabled && s.rampStartTime === null ? Date.now() : s.rampStartTime,
          isSkipContext: false,
          isOnBreak: false,
          lastBreakWordIndex,
        };
      });

      playbackStartTime = Date.now();

      // Auto-save progress every 10 seconds
      if (progressSaveInterval) clearInterval(progressSaveInterval);
      progressSaveInterval = setInterval(() => {
        const s = getState();
        saveProgress(s.currentIndex, false);
      }, 10_000);

      scheduleNext();
    },

    pause() {
      cancelAnimation();
      clearTimers();
      if (progressSaveInterval) {
        clearInterval(progressSaveInterval);
        progressSaveInterval = null;
      }
      if (playbackStartTime !== null) {
        accumulatedPlayMs += Date.now() - playbackStartTime;
        playbackStartTime = null;
      }
      update((s) => ({ ...s, isPlaying: false }));
      const state = getState();
      saveProgress(state.currentIndex, false);
    },

    toggle() {
      const state = getState();
      if (state.isPlaying) {
        this.pause();
      } else {
        this.play();
      }
    },

    seekTo(index: number, showSkipContext = false, animate = true) {
      clearTimers();
      const state = getState();
      const settings = getSettings();
      const clamped = Math.max(0, Math.min(index, state.words.length - 1));
      const shouldAnimate =
        animate &&
        settings.skip_animation_enabled &&
        !state.isPlaying &&
        Math.abs(clamped - state.currentIndex) > 1;

      const finishSkipContext = () => {
        if (!showSkipContext || !settings.skip_context_enabled) return;
        if (skipContextTimeoutId) clearTimeout(skipContextTimeoutId);
        skipContextTimeoutId = setTimeout(() => {
          update((s) => ({ ...s, isSkipContext: false }));
        }, 1500);
      };

      if (shouldAnimate) {
        animateToIndex(clamped, showSkipContext, finishSkipContext);
      } else {
        update((s) => ({
          ...s,
          currentIndex: clamped,
          currentDisplay: s.words[clamped] ?? '',
          isSkipContext: showSkipContext && settings.skip_context_enabled,
        }));
        finishSkipContext();
      }

      if (state.isPlaying && !shouldAnimate) {
        scheduleNext();
      }
    },

    skipToSentence(direction: 'forward' | 'back') {
      cancelAnimation();
      const wasPlaying = getState().isPlaying;
      this.pause();

      const state = getState();
      const settings = getSettings();
      const sentenceArray = Array.from(state.sentenceIndices);

      let targetIndex: number;
      if (direction === 'forward') {
        targetIndex = sentenceArray.find((i) => i > state.currentIndex) ?? state.currentIndex;
      } else {
        const prev = sentenceArray.filter((i) => i < state.currentIndex);
        if (!state.sentenceIndices.has(state.currentIndex) && prev.length > 0) {
          targetIndex = prev[prev.length - 1];
        } else {
          targetIndex = prev[prev.length - 1] ?? 0;
        }
      }

      const finish = () => {
        if (settings.skip_context_enabled) {
          if (skipContextTimeoutId) clearTimeout(skipContextTimeoutId);
          skipContextTimeoutId = setTimeout(() => {
            update((s) => ({ ...s, isSkipContext: false }));
            if (wasPlaying) this.play();
          }, 1500);
        } else if (wasPlaying) {
          this.play();
        }
      };

      if (settings.skip_animation_enabled && Math.abs(targetIndex - state.currentIndex) > 1) {
        animateToIndex(targetIndex, settings.skip_context_enabled, finish);
      } else {
        update((s) => ({
          ...s,
          currentIndex: targetIndex,
          currentDisplay: s.words[targetIndex] ?? '',
          isSkipContext: settings.skip_context_enabled,
        }));
        finish();
      }
    },

    skipToParagraph(direction: 'forward' | 'back') {
      cancelAnimation();
      const wasPlaying = getState().isPlaying;
      this.pause();

      const state = getState();
      const settings = getSettings();
      const paraArray = Array.from(state.paragraphIndices);

      let targetIndex: number;
      if (direction === 'forward') {
        targetIndex = paraArray.find((i) => i > state.currentIndex) ?? state.currentIndex;
      } else {
        const prev = paraArray.filter((i) => i < state.currentIndex);
        targetIndex = prev.length > 0 ? prev[prev.length - 1] : 0;
      }

      const finish = () => {
        if (settings.skip_context_enabled) {
          if (skipContextTimeoutId) clearTimeout(skipContextTimeoutId);
          skipContextTimeoutId = setTimeout(() => {
            update((s) => ({ ...s, isSkipContext: false }));
            if (wasPlaying) this.play();
          }, 1500);
        } else if (wasPlaying) {
          this.play();
        }
      };

      if (settings.skip_animation_enabled && Math.abs(targetIndex - state.currentIndex) > 1) {
        animateToIndex(targetIndex, settings.skip_context_enabled, finish);
      } else {
        update((s) => ({
          ...s,
          currentIndex: targetIndex,
          currentDisplay: s.words[targetIndex] ?? '',
          isSkipContext: settings.skip_context_enabled,
        }));
        finish();
      }
    },

    adjustWpm(delta: number) {
      const settings = getSettings();
      update((s) => {
        const newWpm = Math.max(50, Math.min(1000, s.targetWpm + delta));
        return { ...s, targetWpm: newWpm, currentWpm: newWpm };
      });
      // Persist new WPM to settings
      settingsStore.patch({ wpm: getState().targetWpm });
    },

    dismissBreak(quizScore: number | null) {
      const state = getState();
      saveProgress(state.currentIndex, false);

      if (quizScore !== null) {
        libraryStore.updateProgress(
          state.bookId,
          state.currentIndex,
          state.sessionWordsRead,
          0,
          quizScore,
          false,
        );
      }

      update((s) => ({
        ...s,
        isOnBreak: false,
        lastBreakWordIndex: s.currentIndex,
      }));
      this.pause();
    },

    navigateTo(view: AppView) {
      if (getState().isPlaying) this.pause();
      update((s) => ({ ...s, view }));
    },

    openBookDetail(bookId: string) {
      if (getState().isPlaying) this.pause();
      update((s) => ({ ...s, view: 'book_detail', selectedBookId: bookId }));
    },

    setShowSettings(show: boolean) {
      update((s) => ({ ...s, showSettings: show }));
    },

    setShowStats(show: boolean) {
      update((s) => ({ ...s, showStats: show }));
    },

    getSessionStats() {
      const state = getState();
      const playMs = currentPlayingMs();
      const minutesPlayed = playMs / 60_000;
      const avgWpm = minutesPlayed > 0 ? Math.round(state.sessionWordsRead / minutesPlayed) : 0;
      return {
        wordsRead: state.sessionWordsRead,
        minutesPlayed: Math.round(minutesPlayed * 10) / 10,
        avgWpm,
        progress: state.totalWords > 0 ? state.currentIndex / state.totalWords : 0,
      };
    },
  };
}

export const readerStore = createReaderStore();
