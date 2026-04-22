<script lang="ts">
  import { readerStore } from '../stores/reader';
  import { libraryStore } from '../stores/library';

  // Recompute stats whenever playback index changes
  $: stats = ($readerStore.currentIndex, readerStore.getSessionStats());
  $: bookId = $readerStore.bookId;
  $: entry = $libraryStore.entries.find((e) => e.book.id === bookId);
</script>

{#if $readerStore.showStats}
  <div class="stats-panel">
    <div class="stats-header">
      <span class="stats-title">Session Stats</span>
      <button
        class="close-btn"
        onclick={() => readerStore.setShowStats(false)}
        aria-label="Close stats"
      >×</button>
    </div>

    <div class="stats-grid">
      <div class="stat-item">
        <span class="stat-value">{stats.wordsRead.toLocaleString()}</span>
        <span class="stat-label">Words Read</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{stats.avgWpm}</span>
        <span class="stat-label">Avg WPM</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{stats.minutesPlayed}</span>
        <span class="stat-label">Min. Reading</span>
      </div>
      <div class="stat-item">
        <span class="stat-value">{Math.round(stats.progress * 100)}%</span>
        <span class="stat-label">Book Progress</span>
      </div>
    </div>

    {#if entry}
      <div class="book-info">
        <div class="book-title">{entry.book.title}</div>
        {#if entry.book.author}
          <div class="book-author">{entry.book.author}</div>
        {/if}
        <div class="book-words">{entry.book.word_count.toLocaleString()} words total</div>
      </div>

      {#if entry.sessions.length > 1}
        <div class="sessions-label">Past sessions</div>
        <div class="sessions-list">
          {#each entry.sessions.slice(0, -1).reverse() as session}
            <div class="session-row">
              <span class="session-date"
                >{new Date(session.started_at).toLocaleDateString()}</span
              >
              <span class="session-wpm">{Math.round(session.average_wpm)} WPM</span>
              <span class="session-words">{session.words_read.toLocaleString()} words</span>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .stats-panel {
    position: fixed;
    top: 50%;
    right: 24px;
    transform: translateY(-50%);
    width: 240px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 16px;
    z-index: 50;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
    font-family: var(--font-family);
  }

  .stats-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }

  .stats-title {
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--muted);
    cursor: pointer;
    font-size: 18px;
    line-height: 1;
    padding: 0;
  }

  .stats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 16px;
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
  }

  .stat-value {
    font-size: 22px;
    font-weight: 700;
    color: var(--accent);
  }

  .stat-label {
    font-size: 10px;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    text-align: center;
  }

  .book-info {
    border-top: 1px solid var(--border);
    padding-top: 12px;
    margin-bottom: 12px;
  }

  .book-title {
    font-weight: 600;
    font-size: 13px;
    color: var(--text);
    margin-bottom: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .book-author {
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 4px;
  }

  .book-words {
    font-size: 11px;
    color: var(--muted);
  }

  .sessions-label {
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 6px;
  }

  .sessions-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 120px;
    overflow-y: auto;
  }

  .session-row {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--muted);
    padding: 3px 0;
    border-bottom: 1px solid var(--border);
  }

  .session-date {
    color: var(--text);
  }
</style>
