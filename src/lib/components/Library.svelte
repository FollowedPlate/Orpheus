<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import MissingBookFileCallout from './MissingBookFileCallout.svelte';
  import { isMissingBookFileError } from '../missingBookFile';
  import { readerStore } from '../stores/reader';
  import { libraryStore } from '../stores/library';
  import { settingsStore } from '../stores/settings';
  import type { BookMetadata, LibraryEntry, ParsedBook, Settings } from '../types';

  let opening = false;
  let openError: string | null = null;
  let missingFileEntry: LibraryEntry | null = null;
  let confirmRemove: string | null = null;

  $: entries = $libraryStore.entries.slice().sort((a, b) => {
    const aDate = a.book.last_read_at ?? a.book.added_at;
    const bDate = b.book.last_read_at ?? b.book.added_at;
    return new Date(bDate).getTime() - new Date(aDate).getTime();
  });

  function canGenerateMetadata(settings: Settings): boolean {
    if (settings.llm_provider === 'OpenAI') {
      return !!settings.llm_api_key?.trim();
    }
    return !!settings.llm_endpoint?.trim() && !!settings.llm_model?.trim();
  }

  async function generateMetadataInBackground(entry: LibraryEntry, parsed: ParsedBook) {
    if (!canGenerateMetadata($settingsStore)) return;

    try {
      const maxWords = Math.min(5000, Math.max(500, $settingsStore.metadata_context_max_words));
      const textExcerpt = parsed.words.slice(0, maxWords).join(' ');
      if (!textExcerpt.trim()) return;

      const metadata = await invoke<BookMetadata>('generate_book_metadata', {
        settings: $settingsStore,
        textExcerpt,
        bookTitle: entry.book.title,
        bookAuthor: entry.book.author,
      });

      await libraryStore.updateBookMetadata(entry.book.id, metadata);
    } catch (e) {
      console.warn('Metadata generation skipped/failed:', e);
    }
  }

  async function openFile() {
    opening = true;
    openError = null;
    missingFileEntry = null;
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: 'Books',
            extensions: ['txt', 'pdf', 'epub', 'azw3', 'mobi'],
          },
        ],
      });

      if (!selected || typeof selected !== 'string') {
        opening = false;
        return;
      }

      const filePath = selected;
      const parsed = await invoke<ParsedBook>('parse_book', { path: filePath });

      // Build library entry
      const id = crypto.randomUUID();
      const entry: LibraryEntry = {
        book: {
          id,
          title: parsed.title,
          author: parsed.author,
          file_path: filePath,
          file_format: filePath.split('.').pop()?.toLowerCase() ?? 'txt',
          word_count: parsed.words.length,
          added_at: new Date().toISOString(),
          last_read_at: null,
          metadata: null,
        },
        current_word_index: 0,
        progress: 0,
        sessions: [],
      };

      await libraryStore.addBook(entry);
      await invoke('start_session', { bookId: id });

      readerStore.loadBook(parsed, id, 0);
      void generateMetadataInBackground(entry, parsed);
    } catch (e) {
      if (isMissingBookFileError(e)) {
        openError =
          'That file is no longer at this location. It may have been moved or deleted. Choose another file or restore it to this path and try again.';
      } else {
        openError = String(e);
      }
    } finally {
      opening = false;
    }
  }

  async function openBook(entry: LibraryEntry) {
    opening = true;
    openError = null;
    missingFileEntry = null;
    try {
      const parsed = await invoke<ParsedBook>('parse_book', {
        path: entry.book.file_path,
      });

      await invoke('start_session', { bookId: entry.book.id });
      readerStore.loadBook(parsed, entry.book.id, entry.current_word_index);
      if (!entry.book.metadata) {
        void generateMetadataInBackground(entry, parsed);
      }
    } catch (e) {
      if (isMissingBookFileError(e)) {
        missingFileEntry = entry;
      } else {
        openError = String(e);
      }
    } finally {
      opening = false;
    }
  }

  async function removeBook(id: string) {
    await libraryStore.removeBook(id);
    confirmRemove = null;
    if (missingFileEntry?.book.id === id) missingFileEntry = null;
  }

  function openBookDetail(id: string) {
    readerStore.openBookDetail(id);
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return '—';
    return new Date(dateStr).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    });
  }
</script>

<div class="library-page">
  <header class="library-header">
    <div class="header-left">
      <h1 class="app-title">RSVP Reader</h1>
      <p class="app-subtitle">Speed reading, reimagined</p>
    </div>
    <div class="header-actions">
      <button
        class="btn-settings"
        onclick={() => readerStore.setShowSettings(true)}
        aria-label="Open settings"
        title="Settings"
      >
        <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
          <path
            d="M19.14 12.94a7 7 0 0 0 .06-.94c0-.32-.03-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7 7 0 0 0-1.62-.94l-.36-2.54A.48.48 0 0 0 14 2h-4a.48.48 0 0 0-.48.41l-.36 2.54a7 7 0 0 0-1.62.94l-2.39-.96a.49.49 0 0 0-.59.22L2.64 8.47a.48.48 0 0 0 .12.61l2.03 1.58a7.1 7.1 0 0 0-.07.94c0 .32.03.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96a7 7 0 0 0 1.62.94l.36 2.54c.05.24.27.41.48.41h4c.24 0 .44-.17.48-.41l.36-2.54a7 7 0 0 0 1.62-.94l2.39.96a.49.49 0 0 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.03-1.58zM12 15.6A3.6 3.6 0 1 1 12 8.4a3.6 3.6 0 0 1 0 7.2z"
          />
        </svg>
      </button>
      <button class="btn-open" onclick={openFile} disabled={opening}>
        {opening ? 'Opening…' : '+ Open Book'}
      </button>
    </div>
  </header>

  {#if missingFileEntry}
    <MissingBookFileCallout
      entry={missingFileEntry}
      onDismiss={() => (missingFileEntry = null)}
      onAfterRelocate={async (bookId) => {
        const next = $libraryStore.entries.find((e) => e.book.id === bookId);
        if (next) await openBook(next);
      }}
    />
  {:else if openError}
    <div class="error-banner">
      <span>{openError}</span>
      <button onclick={() => (openError = null)}>×</button>
    </div>
  {/if}

  {#if entries.length === 0}
    <div class="empty-state">
      <div class="empty-icon">📚</div>
      <h2>No books yet</h2>
      <p>Open a .txt, .pdf, .epub, or .azw3 file to get started.</p>
      <button class="btn-open large" onclick={openFile} disabled={opening}>
        Open a Book
      </button>
    </div>
  {:else}
    <div class="book-grid">
      {#each entries as entry (entry.book.id)}
        <div class="book-card" role="button" tabindex="0" onclick={() => openBookDetail(entry.book.id)} onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && openBookDetail(entry.book.id)}>
          <!-- Book spine color based on format -->
          <div
            class="book-spine"
            style="background: {entry.book.file_format === 'pdf'
              ? '#e53935'
              : entry.book.file_format === 'epub'
              ? '#1565c0'
              : entry.book.file_format === 'azw3' || entry.book.file_format === 'mobi'
              ? '#ff8f00'
              : '#4caf50'}"
          ></div>

          <div class="book-body">
            <div class="book-meta-top">
              <span class="book-format">{entry.book.file_format.toUpperCase()}</span>
              <button
                class="remove-btn"
                onclick={(e) => {
                  e.stopPropagation();
                  confirmRemove = entry.book.id;
                }}
                aria-label="Remove book"
                title="Remove from library"
              >×</button>
            </div>

            <div class="book-info">
              <h3 class="book-title">{entry.book.title}</h3>
              {#if entry.book.author}
                <p class="book-author">{entry.book.author}</p>
              {/if}
              {#if entry.book.metadata}
                <p class="book-metadata">
                  {entry.book.metadata.genre}
                  {#if entry.book.metadata.year_written}
                    · {entry.book.metadata.year_written}
                  {/if}
                </p>
              {/if}
              <p class="book-words">{entry.book.word_count.toLocaleString()} words</p>
            </div>

            <!-- Progress bar -->
            <div class="progress-bar-wrap">
              <div class="progress-bar" style="width: {Math.round(entry.progress * 100)}%"></div>
            </div>
            <div class="progress-text">
              {Math.round(entry.progress * 100)}% complete
            </div>

            <div class="book-footer">
              <span class="last-read">
                {entry.book.last_read_at ? 'Read ' + formatDate(entry.book.last_read_at) : 'Never read'}
              </span>
              <button class="read-btn" onclick={(e) => { e.stopPropagation(); openBook(entry); }} disabled={opening}>
                {entry.progress > 0 ? 'Continue' : 'Read'}
              </button>
            </div>
          </div>

          {#if confirmRemove === entry.book.id}
            <div
              class="confirm-overlay"
              role="alertdialog"
              aria-modal="true"
              aria-labelledby="confirm-remove-title-{entry.book.id}"
              tabindex="-1"
              onclick={(e) => e.stopPropagation()}
              onkeydown={(e) => {
                e.stopPropagation();
                if (e.key === 'Escape') confirmRemove = null;
              }}
            >
              <p id="confirm-remove-title-{entry.book.id}">Remove from library?</p>
              <div class="confirm-actions">
                <button
                  class="btn-cancel"
                  onclick={(e) => {
                    e.stopPropagation();
                    confirmRemove = null;
                  }}>Cancel</button>
                <button
                  class="btn-confirm"
                  onclick={(e) => {
                    e.stopPropagation();
                    removeBook(entry.book.id);
                  }}>Remove</button>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .library-page {
    min-height: 100vh;
    background: var(--bg);
    font-family: var(--font-family);
    padding: 0 0 48px 0;
  }

  .library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 28px 40px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--bg);
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .header-left {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .app-title {
    font-size: 22px;
    font-weight: 800;
    color: var(--text);
    margin: 0;
    letter-spacing: -0.02em;
  }

  .app-subtitle {
    font-size: 12px;
    color: var(--muted);
    margin: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .btn-settings {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text);
    cursor: pointer;
    padding: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background 0.15s;
  }

  .btn-settings:hover {
    background: var(--border);
  }

  .btn-open {
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

  .btn-open:hover:not(:disabled) {
    filter: brightness(1.15);
  }

  .btn-open:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-open.large {
    padding: 14px 28px;
    font-size: 16px;
  }

  .error-banner {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: color-mix(in srgb, #f44336 15%, transparent);
    border: 1px solid #f44336;
    color: #f44336;
    padding: 10px 24px;
    font-size: 13px;
  }

  .error-banner button {
    background: none;
    border: none;
    color: #f44336;
    cursor: pointer;
    font-size: 18px;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 60vh;
    gap: 12px;
    text-align: center;
    color: var(--text);
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 8px;
  }

  .empty-state h2 {
    font-size: 22px;
    font-weight: 700;
    margin: 0;
  }

  .empty-state p {
    font-size: 14px;
    color: var(--muted);
    margin: 0 0 16px 0;
  }

  .book-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 20px;
    padding: 32px 40px;
  }

  .book-card {
    position: relative;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    overflow: hidden;
    display: flex;
    transition: transform 0.15s, box-shadow 0.15s;
    cursor: pointer;
  }

  .book-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
  }

  .book-spine {
    width: 6px;
    flex-shrink: 0;
  }

  .book-body {
    flex: 1;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }

  .book-meta-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .book-format {
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--muted);
    background: var(--border);
    padding: 2px 6px;
    border-radius: 3px;
  }

  .remove-btn {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
    padding: 0;
    opacity: 0.5;
    transition: opacity 0.15s;
  }

  .remove-btn:hover {
    opacity: 1;
    color: #f44336;
  }

  .book-info {
    flex: 1;
  }

  .book-title {
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
    margin: 0 0 4px 0;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .book-author {
    font-size: 12px;
    color: var(--muted);
    margin: 0 0 4px 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .book-words {
    font-size: 11px;
    color: var(--muted);
    margin: 0;
  }

  .book-metadata {
    font-size: 11px;
    color: var(--accent);
    margin: 0 0 4px 0;
    font-weight: 500;
  }

  .progress-bar-wrap {
    height: 3px;
    background: var(--border);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    transition: width 0.3s;
  }

  .progress-text {
    font-size: 11px;
    color: var(--muted);
  }

  .book-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 4px;
  }

  .last-read {
    font-size: 11px;
    color: var(--muted);
  }

  .read-btn {
    background: var(--accent);
    color: var(--bg);
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 12px;
    font-weight: 600;
    font-family: var(--font-family);
    cursor: pointer;
    transition: filter 0.15s;
  }

  .read-btn:hover:not(:disabled) {
    filter: brightness(1.15);
  }

  .read-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .confirm-overlay {
    position: absolute;
    inset: 0;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 16px;
    text-align: center;
  }

  .confirm-overlay p {
    font-size: 13px;
    color: var(--text);
    margin: 0;
  }

  .confirm-actions {
    display: flex;
    gap: 8px;
  }

  .btn-cancel {
    background: var(--border);
    color: var(--text);
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 12px;
    font-family: var(--font-family);
    cursor: pointer;
  }

  .btn-confirm {
    background: #f44336;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 12px;
    font-family: var(--font-family);
    cursor: pointer;
  }
</style>
