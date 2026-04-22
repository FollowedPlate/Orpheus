const SHORT_WORD_THRESHOLD = 3;

export interface WordGroup {
  displayText: string;
  startIndex: number;
  endIndex: number;
}

/**
 * Groups consecutive short words together into a single flash.
 * Returns the group starting at `startIndex`.
 */
export function buildWordGroup(
  words: string[],
  startIndex: number,
  enabled: boolean,
): WordGroup {
  const word = words[startIndex] ?? '';

  if (!enabled || startIndex >= words.length) {
    return { displayText: word, startIndex, endIndex: startIndex };
  }

  const isShort = (w: string) =>
    w.replace(/\W/g, '').length <= SHORT_WORD_THRESHOLD;

  // Only group if the current word is short
  if (!isShort(word)) {
    return { displayText: word, startIndex, endIndex: startIndex };
  }

  // Collect consecutive short words (max 2 additional)
  let endIndex = startIndex;
  let display = word;
  let extras = 0;

  while (extras < 2 && endIndex + 1 < words.length) {
    const next = words[endIndex + 1];
    if (!isShort(next)) break;
    endIndex++;
    display += ' ' + next;
    extras++;
  }

  return { displayText: display, startIndex, endIndex };
}
