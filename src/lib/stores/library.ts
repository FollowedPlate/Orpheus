import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Library, LibraryEntry } from '../types';

function createLibraryStore() {
  const { subscribe, set, update } = writable<Library>({ entries: [] });

  return {
    subscribe,

    async load() {
      try {
        const lib = await invoke<Library>('load_library');
        set(lib);
      } catch (e) {
        console.error('Failed to load library:', e);
      }
    },

    async addBook(entry: LibraryEntry) {
      try {
        await invoke('add_book_to_library', { entry });
        update((lib) => {
          const filtered = lib.entries.filter(
            (e) => e.book.file_path !== entry.book.file_path,
          );
          return { entries: [...filtered, entry] };
        });
      } catch (e) {
        console.error('Failed to add book:', e);
      }
    },

    async removeBook(bookId: string) {
      try {
        await invoke('remove_book', { bookId });
        update((lib) => ({
          entries: lib.entries.filter((e) => e.book.id !== bookId),
        }));
      } catch (e) {
        console.error('Failed to remove book:', e);
      }
    },

    async updateProgress(
      bookId: string,
      currentWordIndex: number,
      wordsRead: number,
      averageWpm: number,
      quizScore: number | null,
      sessionEnded: boolean,
    ) {
      try {
        await invoke('update_progress', {
          update: {
            book_id: bookId,
            current_word_index: currentWordIndex,
            words_read: wordsRead,
            average_wpm: averageWpm,
            quiz_score: quizScore,
            session_ended: sessionEnded,
          },
        });
        update((lib) => {
          const entry = lib.entries.find((e) => e.book.id === bookId);
          if (entry) {
            entry.current_word_index = currentWordIndex;
            entry.progress =
              entry.book.word_count > 0
                ? currentWordIndex / entry.book.word_count
                : 0;
          }
          return { ...lib };
        });
      } catch (e) {
        console.error('Failed to update progress:', e);
      }
    },
  };
}

export const libraryStore = createLibraryStore();
