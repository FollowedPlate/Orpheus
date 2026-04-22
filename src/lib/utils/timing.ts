import type { Settings } from '../types';

/**
 * Computes the display delay in ms for a given word.
 */
export function getWordDelay(
  word: string,
  wordIndex: number,
  paragraphIndices: number[],
  settings: Settings,
  currentWpm: number,
): number {
  const baseDelay = 60_000 / currentWpm;
  const stripped = word.replace(/\W/g, '');
  let multiplier = 1.0;

  const nextIsParagraphStart = paragraphIndices.includes(wordIndex + 1);
  const isSentenceEnd = /[.!?…]["')\]»]?$/.test(word);
  const isClauseEnd = /[,;:]$/.test(word);

  if (nextIsParagraphStart) {
    multiplier = settings.paragraph_multiplier;
  } else if (isSentenceEnd) {
    multiplier = settings.sentence_end_multiplier;
  } else if (isClauseEnd) {
    multiplier = settings.clause_multiplier;
  }

  if (stripped.length > settings.long_word_threshold) {
    const excess = stripped.length - settings.long_word_threshold;
    const scaledMultiplier = 1.0 + (settings.long_word_multiplier - 1.0) * excess;
    multiplier = Math.max(multiplier, scaledMultiplier);
  }

  return baseDelay * multiplier;
}

/**
 * Returns the current WPM accounting for a speed ramp-up.
 */
export function getRampedWpm(
  targetWpm: number,
  startWpm: number,
  rampDurationMs: number,
  elapsedMs: number,
): number {
  if (elapsedMs >= rampDurationMs) return targetWpm;
  const progress = elapsedMs / rampDurationMs;
  return Math.round(startWpm + (targetWpm - startWpm) * progress);
}

/** Returns true if a word ends a sentence (for skip boundary detection). */
export function isSentenceBoundary(word: string): boolean {
  return /[.!?…]["')\]»]?$/.test(word);
}
