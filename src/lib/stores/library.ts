import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Book, BookMetadata, Library } from '../types';

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

    async addBook(entry: Book) {
      try {
        await invoke('add_book_to_library', { entry });
        update((lib) => {
          const filtered = lib.entries.filter(
            (e) => e.file_path !== entry.file_path,
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
          entries: lib.entries.filter((e) => e.id !== bookId),
        }));
      } catch (e) {
        console.error('Failed to remove book:', e);
      }
    },

    async relocateBook(bookId: string, newPath: string) {
      await invoke('update_book_file_path', { bookId, newPath });
      const fresh = await invoke<Library>('load_library');
      set(fresh);
    },

    async updateBookMetadata(bookId: string, metadata: BookMetadata) {
      try {
        await invoke('update_book_metadata', { bookId, metadata });
        update((lib) => {
          const entry = lib.entries.find((e) => e.id === bookId);
          if (entry) {
            entry.genre = metadata.genre;
            entry.year_written = metadata.year_written;
            entry.summary = metadata.summary;
            entry.themes = metadata.themes;
            entry.setting = metadata.setting;
            entry.key_characters = metadata.key_characters;
            entry.notable_context = metadata.notable_context;
          }
          return { ...lib };
        });
      } catch (e) {
        console.error('Failed to update book metadata:', e);
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
          const entry = lib.entries.find((e) => e.id === bookId);
          if (entry) {
            entry.current_word_index = currentWordIndex;
            const wc = entry.word_count ?? 0;
            entry.progress = wc > 0 ? currentWordIndex / wc : 0;
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
