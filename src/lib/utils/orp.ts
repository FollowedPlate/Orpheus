import type { OrpAlgorithm } from '../types';

/**
 * Returns the 0-based index of the Optimal Recognition Point character within a word.
 */
export function getOrpIndex(word: string, algorithm: OrpAlgorithm): number {
  // Work on the alphabetic content only for length calculation,
  // but apply the index to the original word.
  const len = word.length;
  if (len === 0) return 0;
  const spritzIndex = getSpritzOrpIndex(len);

  if (algorithm === 'Center') {
    return Math.floor((len - 1) / 2);
  }

  if (algorithm === 'Dynamic') {
    if (len < 13) return spritzIndex;
    const centerIndex = Math.floor((len - 1) / 2);
    const blend = Math.min(1, (len - 13) / 12);
    return Math.round(spritzIndex + (centerIndex - spritzIndex) * blend);
  }

  return spritzIndex;
}

function getSpritzOrpIndex(len: number): number {
  // Spritz-style: optimized for visual recognition
  if (len === 1) return 0;
  if (len <= 5) return 1;
  if (len <= 9) return 2;
  if (len <= 13) return 3;
  return 4;
}

/**
 * Splits a word into [before, orp_char, after] based on the ORP index.
 */
export function splitAtOrp(word: string, orpIndex: number): [string, string, string] {
  const clamped = Math.min(orpIndex, word.length - 1);
  return [word.slice(0, clamped), word[clamped] ?? '', word.slice(clamped + 1)];
}

/**
 * Given a display string (may be multiple grouped words), compute the ORP split.
 * For multi-word groups we apply ORP to the first non-trivial word.
 */
export function computeOrpParts(
  displayText: string,
  algorithm: OrpAlgorithm,
): { before: string; orp: string; after: string } {
  const words = displayText.split(' ');

  if (words.length === 1) {
    const idx = getOrpIndex(displayText, algorithm);
    const [before, orp, after] = splitAtOrp(displayText, idx);
    return { before, orp, after };
  }

  // For grouped words, apply ORP to the first word
  const firstWord = words[0];
  const rest = words.slice(1).join(' ');
  const idx = getOrpIndex(firstWord, algorithm);
  const [before, orp, after] = splitAtOrp(firstWord, idx);
  return { before, orp, after: after + (rest ? ' ' + rest : '') };
}
