import type { Book } from '../types';

/** Case-insensitive path compare (normalizes slashes for Windows). */
export function filePathsEqual(
  a: string | null | undefined,
  b: string | null | undefined,
): boolean {
  if (!a || !b) return false;
  return a.replace(/\\/g, '/').toLowerCase() === b.replace(/\\/g, '/').toLowerCase();
}

function pickStr(
  existing: string | null | undefined,
  parsed: string | null | undefined,
): string | null {
  if (existing?.trim()) return existing;
  if (parsed?.trim()) return parsed;
  return existing ?? parsed ?? null;
}

/** Keep existing list if non-empty; otherwise use parsed. If both non-empty, append parsed items not already present (case-insensitive). */
function mergeStrArrays(
  existing: string[] | null | undefined,
  parsed: string[] | null | undefined,
): string[] | null {
  const ex = existing?.map((s) => s.trim()).filter(Boolean) ?? [];
  const pr = parsed?.map((s) => s.trim()).filter(Boolean) ?? [];
  if (ex.length === 0) return pr.length > 0 ? pr : null;
  if (pr.length === 0) return ex;
  const seen = new Set(ex.map((s) => s.toLowerCase()));
  const out = [...ex];
  for (const s of pr) {
    const k = s.toLowerCase();
    if (!seen.has(k)) {
      seen.add(k);
      out.push(s);
    }
  }
  return out;
}

/** True when parsed word count suggests the on-disk file changed since last library snapshot. */
export function fileWordCountLikelyChanged(oldWc: number, newWc: number): boolean {
  if (oldWc <= 0 || newWc <= 0) return oldWc > 0 && newWc === 0;
  const d = Math.abs(newWc - oldWc);
  if (d <= 20) return false;
  const rel = d / oldWc;
  return rel > 0.05 || d > 400;
}

/**
 * When reopening a path that already exists in the library: keep id, dates, sessions,
 * progress index (clamped), and existing metadata; fill empty metadata from the fresh parse.
 */
export function mergeReopenBookWithParsed(
  existing: Book,
  parsed: Book,
  filePath: string,
): { entry: Book; fileLikelyChanged: boolean } {
  const oldWc = existing.word_count ?? 0;
  const newWc = parsed.words?.length ?? 0;
  const fileLikelyChanged = fileWordCountLikelyChanged(oldWc, newWc);

  const prevIdx = existing.current_word_index ?? 0;
  const clampedIdx = newWc === 0 ? 0 : Math.min(prevIdx, newWc - 1);

  const entry: Book = {
    ...existing,
    id: existing.id,
    title: parsed.title,
    author: pickStr(existing.author, parsed.author),
    file_path: filePath,
    file_format: filePath.split('.').pop()?.toLowerCase() ?? existing.file_format ?? null,
    word_count: newWc,
    added_at: existing.added_at,
    last_read_at: existing.last_read_at,
    current_word_index: clampedIdx,
    progress: newWc > 0 ? clampedIdx / newWc : 0,
    sessions: existing.sessions ?? [],
    genre: pickStr(existing.genre, parsed.genre),
    year_written: pickStr(existing.year_written, parsed.year_written),
    summary: pickStr(existing.summary, parsed.summary),
    setting: pickStr(existing.setting, parsed.setting),
    notable_context: pickStr(existing.notable_context, parsed.notable_context),
    themes: mergeStrArrays(existing.themes, parsed.themes),
    key_characters: mergeStrArrays(existing.key_characters, parsed.key_characters),
  };

  return { entry, fileLikelyChanged };
}

/** After merge, still worth calling LLM metadata if themes/characters are sparse. */
export function bookNeedsLlmMetadataEnhancement(b: Book): boolean {
  const themes = b.themes ?? [];
  const kc = b.key_characters ?? [];
  return themes.length === 0 || kc.length === 0;
}
