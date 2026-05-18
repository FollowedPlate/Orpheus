<script lang="ts">
  import { readerStore } from '../stores/reader';
  import { settingsStore } from '../stores/settings';

  const state = readerStore;

  $: isPlaying = $state.isPlaying;
  $: progress = $state.totalWords > 0 ? $state.currentIndex / $state.totalWords : 0;
  $: wpm = $state.targetWpm;
  $: currentWpm = $state.currentWpm;

  let scrubTrackEl: HTMLDivElement | null = null;
  let isScrubbing = false;
  let scrubPointerId: number | null = null;
  let scrubOriginY = 0;
  let scrubLastX = 0;
  let scrubDistanceY = 0;
  let currentDragIndex = 0;
  let scrubDeltaAccumulator = 0;

  $: displayIndex = isScrubbing ? currentDragIndex : $state.currentIndex;
  $: displayProgress = $state.totalWords > 0 ? displayIndex / $state.totalWords : 0;
  $: displayPercentText = `${(displayProgress * 100).toFixed(1)}%`;
  $: displayWordText =
    $state.totalWords > 0
      ? `${Math.min(displayIndex + 1, $state.totalWords)} / ${$state.totalWords}`
      : '0 / 0';
  $: scrubMode = scrubDistanceY > 36 ? 'Fine' : 'Coarse';

  function clampIndex(value: number) {
    return Math.min(Math.max(value, 0), Math.max($state.totalWords - 1, 0));
  }

  function getTrackIndex(clientX: number) {
    if (!scrubTrackEl || $state.totalWords <= 0) return 0;
    const rect = scrubTrackEl.getBoundingClientRect();
    if (rect.width <= 0) return 0;
    const ratio = Math.min(Math.max((clientX - rect.left) / rect.width, 0), 1);
    return Math.round(ratio * Math.max($state.totalWords - 1, 0));
  }

  function handleScrubPointerDown(e: PointerEvent) {
    if (!scrubTrackEl) return;
    const target = e.currentTarget as HTMLElement;
    isScrubbing = true;
    scrubPointerId = e.pointerId;
    scrubOriginY = e.clientY;
    scrubLastX = e.clientX;
    scrubDistanceY = 0;
    scrubDeltaAccumulator = 0;
    currentDragIndex = getTrackIndex(e.clientX);
    target.setPointerCapture(e.pointerId);
    readerStore.seekTo(currentDragIndex, true, false, false);
  }

  function handleScrubPointerMove(e: PointerEvent) {
    if (!isScrubbing || scrubPointerId !== e.pointerId || !scrubTrackEl || $state.totalWords <= 0) return;
    const rect = scrubTrackEl.getBoundingClientRect();
    if (rect.width <= 0) return;
    const deltaX = e.clientX - scrubLastX;
    scrubLastX = e.clientX;
    scrubDistanceY = Math.abs(e.clientY - scrubOriginY);
    // Gets fine faster as pointer moves away from the bar.
    const sensitivity = 1 / Math.pow(1 + scrubDistanceY / 8, 2);
    const rawIndexDelta = (deltaX * sensitivity * Math.max($state.totalWords - 1, 0)) / rect.width;
    scrubDeltaAccumulator += rawIndexDelta;

    let indexDelta = 0;
    if (scrubDistanceY >= 36) {
      // Finest mode: only advance one word per update step.
      if (Math.abs(scrubDeltaAccumulator) >= 1) {
        indexDelta = Math.sign(scrubDeltaAccumulator);
        scrubDeltaAccumulator -= indexDelta;
      }
    } else {
      indexDelta = Math.trunc(scrubDeltaAccumulator);
      if (indexDelta !== 0) {
        scrubDeltaAccumulator -= indexDelta;
      }
    }

    if (indexDelta !== 0) {
      currentDragIndex = clampIndex(currentDragIndex + indexDelta);
      readerStore.seekTo(currentDragIndex, true, false, false);
    }
  }

  function handleScrubPointerEnd(e: PointerEvent) {
    if (!isScrubbing || scrubPointerId !== e.pointerId) return;
    const target = e.currentTarget as HTMLElement;
    isScrubbing = false;
    scrubPointerId = null;
    target.releasePointerCapture(e.pointerId);
    readerStore.seekTo(currentDragIndex, true, true, true);
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
    <div
      class="scrub-track"
      bind:this={scrubTrackEl}
      role="slider"
      tabindex="0"
      aria-label="Reading progress"
      aria-valuemin="0"
      aria-valuemax={$state.totalWords}
      aria-valuenow={displayIndex}
      onpointerdown={handleScrubPointerDown}
      onpointermove={handleScrubPointerMove}
      onpointerup={handleScrubPointerEnd}
      onpointercancel={handleScrubPointerEnd}
    >
      <div class="scrub-fill" style={`width: ${displayProgress * 100}%`}></div>
      <div class="scrub-thumb" style={`left: ${displayProgress * 100}%`}></div>
      {#if isScrubbing}
        <div class="scrub-tooltip">
          {displayPercentText} • {displayWordText}
          <span class="scrub-mode">{scrubMode}</span>
        </div>
      {/if}
    </div>
    <span class="progress-label">{Math.round(displayProgress * 100)}%</span>
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

  .scrub-track {
    flex: 1;
    position: relative;
    height: 10px;
    border-radius: 999px;
    background: color-mix(in srgb, var(--border) 75%, transparent);
    cursor: pointer;
    touch-action: none;
  }

  .scrub-fill {
    position: absolute;
    inset: 0 auto 0 0;
    width: 0%;
    border-radius: 999px;
    background: var(--accent);
  }

  .scrub-thumb {
    position: absolute;
    top: 50%;
    left: 0%;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    background: var(--accent);
    border: 2px solid var(--bg);
    box-shadow: 0 1px 6px rgba(0, 0, 0, 0.35);
    pointer-events: none;
  }

  .scrub-tooltip {
    position: absolute;
    left: 50%;
    bottom: calc(100% + 8px);
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 8px;
    white-space: nowrap;
    font-size: 11px;
    color: var(--text);
    background: color-mix(in srgb, var(--surface) 88%, black 12%);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 4px 8px;
    font-family: var(--font-family);
    pointer-events: none;
    z-index: 2;
  }

  .scrub-mode {
    font-size: 10px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--accent);
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
