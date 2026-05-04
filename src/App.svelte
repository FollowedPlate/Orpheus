<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { readerStore } from './lib/stores/reader';
  import { settingsStore } from './lib/stores/settings';
  import { libraryStore } from './lib/stores/library';
  import { applyTheme } from './lib/stores/theme';
  import { shortcuts } from './lib/utils/shortcuts';
  import Library from './lib/components/Library.svelte';
  import RSVPDisplay from './lib/components/RSVPDisplay.svelte';
  import Controls from './lib/components/Controls.svelte';
  import SettingsPanel from './lib/components/SettingsPanel.svelte';
  import ComprehensionQuiz from './lib/components/ComprehensionQuiz.svelte';
  import ReadingStats from './lib/components/ReadingStats.svelte';
  import BookDetail from './lib/components/BookDetail.svelte';
  import TableOfContents from './lib/components/TableOfContents.svelte';

  onMount(async () => {
    // Load persisted data
    await settingsStore.load();
    await libraryStore.load();

    // Apply theme from loaded settings
    applyTheme(get(settingsStore));

    // Subscribe to settings changes to keep theme and shortcuts in sync
    const unsubSettings = settingsStore.subscribe((s) => {
      applyTheme(s);
      shortcuts.setMap(s.shortcuts);
    });

    // Register keyboard shortcut actions
    shortcuts.on('play_pause', () => readerStore.toggle());
    shortcuts.on('skip_forward_sentence', () => readerStore.skipToSentence('forward'));
    shortcuts.on('skip_back_sentence', () => readerStore.skipToSentence('back'));
    shortcuts.on('skip_forward_word', () => readerStore.skipWord('forward'));
    shortcuts.on('skip_back_word', () => readerStore.skipWord('back'));
    shortcuts.on('skip_forward_paragraph', () => readerStore.skipToParagraph('forward'));
    shortcuts.on('skip_back_paragraph', () => readerStore.skipToParagraph('back'));
    shortcuts.on('increase_wpm', () => readerStore.adjustWpm(get(settingsStore).wpm_step));
    shortcuts.on('decrease_wpm', () => readerStore.adjustWpm(-get(settingsStore).wpm_step));
    shortcuts.on('toggle_focus', () => {
      const s = get(settingsStore);
      settingsStore.patch({ focus_mode: !s.focus_mode });
    });
    shortcuts.on('open_settings', () => {
      readerStore.setShowSettings(!get(readerStore).showSettings);
    });
    shortcuts.on('return_to_library', () => {
      const rs = get(readerStore);
      if (rs.showSettings) {
        readerStore.setShowSettings(false);
      } else if (rs.view === 'reader' || rs.view === 'book_detail') {
        readerStore.navigateTo('library');
      }
    });

    return () => {
      unsubSettings();
      shortcuts.clear();
    };
  });
</script>

<svelte:window onkeydown={(e) => shortcuts.handle(e)} />

<div
  class="app"
  class:focus-mode={$settingsStore.focus_mode && $readerStore.view === 'reader'}
>
  {#if $readerStore.view === 'library'}
    <Library />
  {:else if $readerStore.view === 'book_detail'}
    <BookDetail />
  {:else if $readerStore.view === 'reader'}
    <div class="reader-view">
      <div class="reader-topbar">
        <button
          class="back-btn"
          onclick={() => readerStore.navigateTo('library')}
          title="Back to Library (Esc)"
          aria-label="Back to library"
        >
          <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
            <path d="M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z" />
          </svg>
          Library
        </button>

        <div class="topbar-title">
          {$libraryStore.entries.find((e) => e.id === $readerStore.bookId)?.title ?? ''}
        </div>

        <div class="topbar-actions">
          <button
            class="icon-action toc-toolbar-btn"
            class:active={$readerStore.tocOpen}
            onclick={() => readerStore.toggleToc()}
            title="Table of contents"
            aria-label="Toggle table of contents"
          >
            <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18" aria-hidden="true">
              <path
                d="M4 6h16v2H4V6zm0 5h16v2H4v-2zm0 5h10v2H4v-2z"
              />
            </svg>
          </button>

          <button
            class="icon-action"
            onclick={() => readerStore.setShowStats(!$readerStore.showStats)}
            title="Reading stats"
            aria-label="Toggle stats"
          >
            <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
              <path d="M5 9.2h3V19H5zM10.6 5h2.8v14h-2.8zm5.6 8H19v6h-2.8z" />
            </svg>
          </button>

          <button
            class="icon-action"
            class:active={$settingsStore.focus_mode}
            onclick={() => settingsStore.patch({ focus_mode: !$settingsStore.focus_mode })}
            title="Toggle focus mode (F)"
            aria-label="Toggle focus mode"
          >
            <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
              <path
                d="M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z"
              />
            </svg>
          </button>

          <button
            class="icon-action"
            onclick={() => readerStore.setShowSettings(!$readerStore.showSettings)}
            title="Settings (Ctrl+,)"
            aria-label="Settings"
          >
            <svg viewBox="0 0 24 24" fill="currentColor" width="18" height="18">
              <path
                d="M19.14 12.94a7 7 0 0 0 .06-.94c0-.32-.03-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.49.49 0 0 0-.59-.22l-2.39.96a7 7 0 0 0-1.62-.94l-.36-2.54A.48.48 0 0 0 14 2h-4a.48.48 0 0 0-.48.41l-.36 2.54a7 7 0 0 0-1.62.94l-2.39-.96a.49.49 0 0 0-.59.22L2.64 8.47a.48.48 0 0 0 .12.61l2.03 1.58a7.1 7.1 0 0 0-.07.94c0 .32.03.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96a7 7 0 0 0 1.62.94l.36 2.54c.05.24.27.41.48.41h4c.24 0 .44-.17.48-.41l.36-2.54a7 7 0 0 0 1.62-.94l2.39.96a.49.49 0 0 0 .59-.22l1.92-3.32a.49.49 0 0 0-.12-.61l-2.03-1.58zM12 15.6A3.6 3.6 0 1 1 12 8.4a3.6 3.6 0 0 1 0 7.2z"
              />
            </svg>
          </button>
        </div>
      </div>

      <div class="reader-body">
        <TableOfContents />
        <main class="reader-main">
          <RSVPDisplay />
          <Controls />
        </main>
      </div>
    </div>
  {/if}

  {#if $readerStore.showSettings}
    <SettingsPanel />
  {/if}

  <ComprehensionQuiz />
  <ReadingStats />
</div>

<style>
  .app {
    min-height: 100vh;
    background: var(--bg);
    color: var(--text);
    transition: background 0.3s, color 0.3s;
  }

  .reader-view {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .reader-body {
    flex: 1;
    display: flex;
    flex-direction: row;
    min-height: 0;
    overflow: hidden;
  }

  .reader-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 20px;
    border-bottom: 1px solid var(--border);
    background: var(--bg);
    flex-shrink: 0;
    transition: opacity 0.3s;
  }

  .focus-mode .reader-topbar {
    opacity: 0;
    pointer-events: none;
    position: absolute;
    width: 100%;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 13px;
    font-family: var(--font-family);
    padding: 4px 8px;
    border-radius: 6px;
    transition: background 0.15s, color 0.15s;
  }

  .back-btn:hover {
    background: var(--surface);
    color: var(--text);
  }

  .topbar-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
    max-width: 400px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    opacity: 0.7;
  }

  .topbar-actions {
    display: flex;
    gap: 4px;
  }

  .icon-action {
    background: none;
    border: none;
    border-radius: 6px;
    color: var(--muted);
    cursor: pointer;
    padding: 6px;
    transition: background 0.15s, color 0.15s;
    display: flex;
    align-items: center;
  }

  .icon-action:hover {
    background: var(--surface);
    color: var(--text);
  }

  .icon-action.active {
    color: var(--accent);
    background: color-mix(in srgb, var(--accent) 10%, transparent);
  }

  .reader-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 24px;
    padding: 24px;
    overflow: hidden;
    min-width: 0;
  }

  .focus-mode .reader-main {
    justify-content: center;
    padding-top: 0;
  }

  :global(.focus-mode .controls) {
    display: none;
  }

  :global(.focus-mode .toc-panel),
  :global(.focus-mode .toc-toolbar-btn) {
    display: none !important;
  }
</style>
