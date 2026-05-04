<script lang="ts">
  import { tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import MissingBookFileCallout from './MissingBookFileCallout.svelte';
  import { isMissingBookFileError } from '../missingBookFile';
  import { libraryStore } from '../stores/library';
  import { readerStore } from '../stores/reader';
  import { settingsStore } from '../stores/settings';
  import type { Book, BookMetadata } from '../types';

  $: entry = $libraryStore.entries.find((e) => e.id === $readerStore.selectedBookId) ?? null;
  let loading = false;
  let opening = false;
  let missingFileBanner = false;

  function hasAnyMetadata(e: Book): boolean {
    return !!(
      e.genre?.trim() ||
      e.year_written?.trim() ||
      e.summary?.trim() ||
      (e.themes && e.themes.length > 0) ||
      e.setting?.trim() ||
      (e.key_characters && e.key_characters.length > 0) ||
      e.notable_context?.trim()
    );
  }

  function canGenerateMetadata(): boolean {
    if (!entry) return false;
    if ($settingsStore.llm_provider === 'OpenAI') {
      return !!$settingsStore.llm_api_key?.trim();
    }
    return !!$settingsStore.llm_endpoint?.trim() && !!$settingsStore.llm_model?.trim();
  }

  function shouldGenerateMetadataFromParser(parsed: Book): boolean {
    const themes = parsed.themes ?? [];
    const kc = parsed.key_characters ?? [];
    return themes.length === 0 || kc.length === 0;
  }

  async function generateMetadataInBackground(targetEntry: Book, parsed: Book) {
    if (!canGenerateMetadata()) return;
    try {
      const maxWords = Math.min(5000, Math.max(500, $settingsStore.metadata_context_max_words));
      const textExcerpt = (parsed.words ?? []).slice(0, maxWords).join(' ');
      if (!textExcerpt.trim()) return;

      const metadata = await invoke<BookMetadata>('generate_book_metadata', {
        settings: $settingsStore,
        textExcerpt,
        bookTitle: targetEntry.title,
        bookAuthor: targetEntry.author,
      });

      await libraryStore.updateBookMetadata(targetEntry.id!, metadata);
    } catch (e) {
      console.warn('Metadata generation skipped/failed:', e);
    }
  }

  async function startReading() {
    if (!entry) return;
    opening = true;
    missingFileBanner = false;
    try {
      const parsed = await invoke<Book>('parse_book', { path: entry.file_path! });
      await invoke('start_session', { bookId: entry.id });
      readerStore.loadBook(parsed, entry.id!, entry.current_word_index ?? 0);
      if (shouldGenerateMetadataFromParser(parsed)) {
        void generateMetadataInBackground(entry, parsed);
      }
    } catch (e) {
      if (isMissingBookFileError(e)) {
        missingFileBanner = true;
      } else {
        console.error('Failed to open book:', e);
      }
    } finally {
      opening = false;
    }
  }

  async function generateMetadataNow() {
    if (!entry || loading) return;
    loading = true;
    missingFileBanner = false;
    try {
      const parsed = await invoke<Book>('parse_book', { path: entry.file_path! });
      await generateMetadataInBackground(entry, parsed);
    } catch (e) {
      if (isMissingBookFileError(e)) {
        missingFileBanner = true;
      } else {
        console.error('Failed to generate metadata:', e);
      }
    } finally {
      loading = false;
    }
  }
</script>

<div class="book-detail-page">
  {#if !entry}
    <div class="empty-state">
      <h2>Book not found</h2>
      <button class="btn-secondary" onclick={() => readerStore.navigateTo('library')}>Back to Library</button>
    </div>
  {:else}
    <header class="detail-header">
      <button class="btn-secondary" onclick={() => readerStore.navigateTo('library')}>Back to Library</button>
      <button class="btn-primary" onclick={startReading} disabled={opening}>
        {opening ? 'Opening…' : (entry.progress ?? 0) > 0 ? 'Continue Reading' : 'Start Reading'}
      </button>
    </header>

    {#if missingFileBanner}
      <div class="callout-wrap">
        <MissingBookFileCallout
          entry={entry}
          onDismiss={() => (missingFileBanner = false)}
          onAfterRelocate={async () => {
            await tick();
            await startReading();
          }}
          onRemovedFromLibrary={() => readerStore.navigateTo('library')}
        />
      </div>
    {/if}

    <section class="book-overview">
      <h1>{entry.title}</h1>
      {#if entry.author}
        <p class="author">{entry.author}</p>
      {/if}
      <p class="stats">
        {(entry.file_format ?? '').toUpperCase()} · {(entry.word_count ?? 0).toLocaleString()} words · {Math.round((entry.progress ?? 0) * 100)}% complete
      </p>
    </section>

    {#if hasAnyMetadata(entry)}
      <section class="metadata-grid">
        <article>
          <h3>Genre</h3>
          <p>{entry.genre ?? ''}</p>
        </article>
        <article>
          <h3>Year Written</h3>
          <p>{entry.year_written ?? ''}</p>
        </article>
        <article class="wide">
          <h3>Summary</h3>
          <p>{entry.summary ?? ''}</p>
        </article>
        <article>
          <h3>Setting</h3>
          <p>{entry.setting ?? ''}</p>
        </article>
        <article class="wide">
          <h3>Themes</h3>
          <div class="tags">
            {#each entry.themes ?? [] as theme (theme)}
              <span>{theme}</span>
            {/each}
          </div>
        </article>
        <article class="wide">
          <h3>Key Characters</h3>
          <ul>
            {#each entry.key_characters ?? [] as character (character)}
              <li>{character}</li>
            {/each}
          </ul>
        </article>
        <article class="wide">
          <h3>Notable Context</h3>
          <p>{entry.notable_context ?? ''}</p>
        </article>
      </section>
    {:else}
      <section class="metadata-loading">
        <h3>Metadata not available yet</h3>
        <p>
          Metadata is generated in the background when possible. You can start reading now, or generate it manually.
        </p>
        <button class="btn-primary" onclick={generateMetadataNow} disabled={loading || !canGenerateMetadata()}>
          {loading ? 'Generating…' : 'Generate Metadata'}
        </button>
      </section>
    {/if}
  {/if}
</div>

<style>
  .book-detail-page {
    min-height: 100vh;
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-family);
    padding: 28px 40px;
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 28px;
  }

  .callout-wrap :global(.callout) {
    margin: 0 0 24px;
  }

  .book-overview h1 {
    margin: 0;
    font-size: 34px;
    line-height: 1.2;
  }

  .author {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 18px;
  }

  .stats {
    margin: 10px 0 0;
    color: var(--muted);
    font-size: 13px;
  }

  .metadata-grid {
    margin-top: 28px;
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
  }

  article {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 14px;
  }

  article h3 {
    margin: 0 0 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  article p,
  article li {
    margin: 0;
    font-size: 14px;
    line-height: 1.5;
  }

  article ul {
    margin: 0;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .wide {
    grid-column: 1 / -1;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .tags span {
    background: color-mix(in srgb, var(--accent) 16%, transparent);
    color: var(--accent);
    border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border));
    border-radius: 999px;
    padding: 4px 10px;
    font-size: 12px;
  }

  .metadata-loading {
    margin-top: 24px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 620px;
  }

  .metadata-loading p {
    margin: 0;
    color: var(--muted);
  }

  .btn-primary,
  .btn-secondary {
    border-radius: 8px;
    font-family: var(--font-family);
    font-size: 14px;
    cursor: pointer;
    padding: 10px 16px;
  }

  .btn-primary {
    background: var(--accent);
    color: var(--bg);
    border: none;
    font-weight: 600;
  }

  .btn-secondary {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn-primary:disabled,
  .btn-secondary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .empty-state {
    min-height: 60vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
  }
</style>
