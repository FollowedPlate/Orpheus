<script lang="ts">
  import { readerStore } from '../stores/reader';
  import { settingsStore } from '../stores/settings';

  const state = readerStore;

  $: isPlaying = $state.isPlaying;
  $: progress = $state.totalWords > 0 ? $state.currentIndex / $state.totalWords : 0;
  $: wpm = $state.targetWpm;
  $: currentWpm = $state.currentWpm;

  function handleScrub(e: Event) {
    const input = e.target as HTMLInputElement;
    const idx = Math.round((parseFloat(input.value) / 100) * ($state.totalWords - 1));
    readerStore.seekTo(idx, true);
  }

  function handleWpmInput(e: Event) {
    const input = e.target as HTMLInputElement;
    const val = parseInt(input.value, 10);
    if (!isNaN(val)) {
      readerStore.adjustWpm(val - wpm);
    }
  }
</script>

<div class="controls" class:focus-hidden={$state.showSettings}>
  <!-- Progress bar -->
  <div class="progress-area">
    <input
      type="range"
      class="scrub"
      min="0"
      max="100"
      step="0.01"
      value={progress * 100}
      oninput={handleScrub}
      aria-label="Reading progress"
    />
    <span class="progress-label">{Math.round(progress * 100)}%</span>
  </div>

  <!-- Control buttons row -->
  <div class="buttons">
    <!-- Skip back paragraph -->
    <button
      class="btn icon-btn"
      onclick={() => readerStore.skipToParagraph('back')}
      title="Previous paragraph (Ctrl+←)"
      aria-label="Skip to previous paragraph"
    >
      <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
        <path d="M6 6h2v12H6V6zm3.5 6l8.5 6V6l-8.5 6z" />
      </svg>
    </button>

    <!-- Skip back sentence -->
    <button
      class="btn icon-btn"
      onclick={() => readerStore.skipToSentence('back')}
      title="Previous sentence (←)"
      aria-label="Skip to previous sentence"
    >
      <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
        <path d="M15.41 7.41L14 6l-6 6 6 6 1.41-1.41L10.83 12l4.58-4.59z" />
      </svg>
    </button>

    <!-- Play / Pause -->
    <button
      class="btn play-btn"
      onclick={() => readerStore.toggle()}
      title={isPlaying ? 'Pause (Space)' : 'Play (Space)'}
      aria-label={isPlaying ? 'Pause' : 'Play'}
    >
      {#if isPlaying}
        <!-- Pause icon -->
        <svg viewBox="0 0 24 24" fill="currentColor" width="28" height="28">
          <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
        </svg>
      {:else}
        <!-- Play icon -->
        <svg viewBox="0 0 24 24" fill="currentColor" width="28" height="28">
          <path d="M8 5v14l11-7L8 5z" />
        </svg>
      {/if}
    </button>

    <!-- Skip forward sentence -->
    <button
      class="btn icon-btn"
      onclick={() => readerStore.skipToSentence('forward')}
      title="Next sentence (→)"
      aria-label="Skip to next sentence"
    >
      <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
        <path d="M10 6L8.59 7.41 13.17 12l-4.58 4.59L10 18l6-6-6-6z" />
      </svg>
    </button>

    <!-- Skip forward paragraph -->
    <button
      class="btn icon-btn"
      onclick={() => readerStore.skipToParagraph('forward')}
      title="Next paragraph (Ctrl+→)"
      aria-label="Skip to next paragraph"
    >
      <svg viewBox="0 0 24 24" fill="currentColor" width="20" height="20">
        <path d="M6 18l8.5-6L6 6v12zm2.5-6L13 8.25v7.5L8.5 12zM16 6h2v12h-2V6z" />
      </svg>
    </button>
  </div>

  <!-- WPM control -->
  <div class="wpm-area">
    <button
      class="btn wpm-adj"
      onclick={() => readerStore.adjustWpm(-$settingsStore.wpm_step)}
      title="Decrease WPM (↓)"
      aria-label="Decrease speed"
    >−</button>

    <div class="wpm-display">
      <input
        class="wpm-input"
        type="number"
        min="50"
        max="1000"
        value={wpm}
        onchange={handleWpmInput}
        aria-label="Words per minute"
      />
      <span class="wpm-label">WPM</span>
      {#if $settingsStore.speed_ramp_enabled && currentWpm !== wpm}
        <span class="wpm-ramp">(ramping: {currentWpm})</span>
      {/if}
    </div>

    <button
      class="btn wpm-adj"
      onclick={() => readerStore.adjustWpm($settingsStore.wpm_step)}
      title="Increase WPM (↑)"
      aria-label="Increase speed"
    >+</button>
  </div>
</div>

<style>
  .controls {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 12px 24px;
    width: 100%;
    max-width: 700px;
  }

  .progress-area {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
  }

  .scrub {
    flex: 1;
    height: 4px;
    cursor: pointer;
    accent-color: var(--accent);
  }

  .progress-label {
    font-size: 12px;
    color: var(--muted);
    min-width: 36px;
    text-align: right;
    font-family: var(--font-family);
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .btn {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 8px;
    transition: background 0.15s, transform 0.1s;
  }

  .btn:hover {
    background: var(--border);
  }

  .btn:active {
    transform: scale(0.95);
  }

  .play-btn {
    width: 52px;
    height: 52px;
    border-radius: 50%;
    background: var(--accent);
    color: var(--bg);
    border: none;
  }

  .play-btn:hover {
    filter: brightness(1.15);
    background: var(--accent);
  }

  .icon-btn {
    width: 40px;
    height: 40px;
  }

  .wpm-area {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .wpm-adj {
    width: 32px;
    height: 32px;
    font-size: 18px;
    font-weight: 600;
    padding: 0;
  }

  .wpm-display {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .wpm-input {
    width: 64px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--text);
    font-size: 16px;
    font-weight: 600;
    font-family: var(--font-family);
    padding: 4px 8px;
    text-align: center;
  }

  .wpm-input::-webkit-inner-spin-button,
  .wpm-input::-webkit-outer-spin-button {
    opacity: 0;
  }

  .wpm-label {
    font-size: 12px;
    color: var(--muted);
    font-family: var(--font-family);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .wpm-ramp {
    font-size: 11px;
    color: var(--accent);
    font-family: var(--font-family);
  }
</style>
