/** Matches the stable code from `parse_book` when the path is not a readable file. */
export const LIBRARY_FILE_NOT_FOUND = 'LIBRARY_FILE_NOT_FOUND';

export function isMissingBookFileError(err: unknown): boolean {
  const msg = String(err);
  if (msg.includes(LIBRARY_FILE_NOT_FOUND)) return true;
  // Fallback for older builds or race where the file vanishes between stat and read.
  if (/os error 2\)/.test(msg) && /find|exist|specified/i.test(msg)) return true;
  if (/ENOENT/.test(msg)) return true;
  return false;
}
