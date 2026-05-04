<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { libraryStore } from '../stores/library';
  import type { Book } from '../types';

  interface Props {
    entry: Book;
    onDismiss: () => void;
    onAfterRelocate?: (bookId: string) => void | Promise<void>;
    /** Invoked after the book is removed from the library (e.g. navigate away from detail). */
    onRemovedFromLibrary?: () => void;
  }

  let { entry, onDismiss, onAfterRelocate, onRemovedFromLibrary }: Props = $props();

  let busy = $state(false);
  let relocateError = $state<string | null>(null);

  async function chooseNewFile() {
    relocateError = null;
    busy = true;
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
      if (!selected || typeof selected !== 'string') return;

      await libraryStore.relocateBook(entry.id!, selected);
      await onAfterRelocate?.(entry.id!);
      onDismiss();
    } catch (e) {
      relocateError = String(e);
    } finally {
      busy = false;
    }
  }

  async function remove() {
    busy = true;
    relocateError = null;
    try {
      await libraryStore.removeBook(entry.id!);
      onRemovedFromLibrary?.();
      onDismiss();
    } catch (e) {
      relocateError = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="callout" role="alert">
  <div class="callout-head">
    <span class="callout-icon" aria-hidden="true">📂</span>
    <div class="callout-titles">
      <h2 class="callout-title">This book’s file is not on disk</h2>
      <p class="callout-lead">
        <span class="book-name">{entry.title}</span> is still in your library, but nothing exists at the saved
        path anymore. The file may have been moved, renamed, or deleted.
      </p>
    </div>
    <button type="button" class="btn-close" onclick={onDismiss} aria-label="Dismiss">×</button>
  </div>

  <p class="path-label">Saved path</p>
  <code class="path-value">{entry.file_path}</code>

  {#if relocateError}
    <p class="sub-error">{relocateError}</p>
  {/if}

  <div class="actions">
    <button type="button" class="btn-secondary" onclick={chooseNewFile} disabled={busy}>
      {busy ? 'Working…' : 'Choose new file…'}
    </button>
    <button type="button" class="btn-danger" onclick={remove} disabled={busy}>Remove from library</button>
  </div>
</div>

<style>
  .callout {
    margin: 0 40px 20px;
    padding: 20px 22px;
    border-radius: 12px;
    border: 1px solid color-mix(in srgb, var(--accent) 35%, var(--border));
    background: color-mix(in srgb, var(--accent) 8%, var(--surface));
    color: var(--text);
    font-family: var(--font-family);
  }

  .callout-head {
    display: flex;
    align-items: flex-start;
    gap: 14px;
  }

  .callout-icon {
    font-size: 28px;
    line-height: 1;
    flex-shrink: 0;
  }

  .callout-titles {
    flex: 1;
    min-width: 0;
  }

  .callout-title {
    margin: 0 0 8px;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .callout-lead {
    margin: 0;
    font-size: 14px;
    line-height: 1.55;
    color: var(--muted);
  }

  .book-name {
    color: var(--text);
    font-weight: 600;
  }

  .btn-close {
    flex-shrink: 0;
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 22px;
    line-height: 1;
    padding: 0 4px;
    border-radius: 6px;
  }

  .btn-close:hover {
    color: var(--text);
    background: var(--border);
  }

  .path-label {
    margin: 16px 0 6px 42px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--muted);
  }

  .path-value {
    display: block;
    margin-left: 42px;
    margin-right: 0;
    padding: 10px 12px;
    font-size: 12px;
    line-height: 1.45;
    word-break: break-all;
    border-radius: 8px;
    background: var(--bg);
    border: 1px solid var(--border);
    color: var(--muted);
  }

  .sub-error {
    margin: 12px 0 0 42px;
    font-size: 13px;
    color: #f44336;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin: 18px 0 0 42px;
  }

  .btn-secondary,
  .btn-danger {
    border-radius: 8px;
    font-family: var(--font-family);
    font-size: 14px;
    font-weight: 600;
    padding: 10px 16px;
    cursor: pointer;
    border: none;
  }

  .btn-secondary {
    background: var(--accent);
    color: var(--bg);
  }

  .btn-secondary:hover:not(:disabled) {
    filter: brightness(1.12);
  }

  .btn-danger {
    background: transparent;
    color: #f44336;
    border: 1px solid color-mix(in srgb, #f44336 45%, var(--border));
  }

  .btn-danger:hover:not(:disabled) {
    background: color-mix(in srgb, #f44336 12%, transparent);
  }

  .btn-secondary:disabled,
  .btn-danger:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
</style>
