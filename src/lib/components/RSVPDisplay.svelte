<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { readerStore } from '../stores/reader';
  import { settingsStore } from '../stores/settings';
  import { computeOrpParts } from '../utils/orp';
  import type { AdjacentWordsCount } from '../types';

  // ── Focus-mode hint popup ─────────────────────────────────────────
  let showFocusHint = false;
  let focusHintTimer: ReturnType<typeof setTimeout> | null = null;
  let prevFocusMode = false;

  $: {
    const isFocus = $settingsStore.focus_mode;
    if (isFocus && !prevFocusMode) {
      // Focus mode just turned on — show the hint card
      if (focusHintTimer) clearTimeout(focusHintTimer);
      showFocusHint = true;
      focusHintTimer = setTimeout(() => { showFocusHint = false; }, 5000);
    }
    if (!isFocus) {
      // Immediately hide if focus mode is turned off
      if (focusHintTimer) { clearTimeout(focusHintTimer); focusHintTimer = null; }
      showFocusHint = false;
    }
    prevFocusMode = isFocus;
  }

  $: focusKey = $settingsStore.shortcuts.toggle_focus;
  $: escKey   = $settingsStore.shortcuts.return_to_library;

  // ── Reactive state from stores ────────────────────────────────────
  $: display  = $readerStore.currentDisplay || '';
  $: parts    = computeOrpParts(display, $settingsStore.orp_algorithm);
  $: words    = $readerStore.words;
  $: idx      = $readerStore.currentIndex;

  $: orpStyle = [
    `color: var(--orp)`,
    $settingsStore.orp_bold       ? 'font-weight: 700' : '',
    $settingsStore.orp_underline  ? 'text-decoration: underline' : '',
    $settingsStore.orp_size_multiplier !== 1.0
      ? `font-size: calc(var(--font-size) * ${$settingsStore.orp_size_multiplier})`
      : '',
  ].filter(Boolean).join('; ');

  // ── Dynamic word-count measurement ───────────────────────────────
  let containerEl: HTMLDivElement;
  let measureEl: HTMLSpanElement;
  let containerHalfWidth = 0;

  function measureHalfWidth() {
    if (containerEl) containerHalfWidth = containerEl.clientWidth / 2;
  }

  let ro: ResizeObserver | null = null;
  onMount(() => {
    measureHalfWidth();
    ro = new ResizeObserver(measureHalfWidth);
    ro.observe(containerEl);
  });
  onDestroy(() => ro?.disconnect());

  /**
   * For 'dynamic': greedily count how many consecutive words starting at
   * `startIdx` (going outward from the current word) fit within `halfPx`.
   * Direction -1 = left side (before), +1 = right side (after).
   */
  function countDynamicWords(
    fromIdx: number,
    dir: -1 | 1,
    halfPx: number,
  ): number {
    if (!measureEl || halfPx <= 0) return 5;
    let total = 0;
    let n = 0;
    const step = dir === -1 ? -1 : 1;
    let i = fromIdx;
    while (true) {
      i += step;
      if (i < 0 || i >= words.length) break;
      const w = words[i] ?? '';
      measureEl.textContent = w + ' ';
      const wPx = measureEl.offsetWidth;
      if (total + wPx > halfPx) break;
      total += wPx;
      n++;
    }
    return n;
  }

  // ── Effective count ───────────────────────────────────────────────
  $: effectiveCount = ((): AdjacentWordsCount | 0 => {
    if ($readerStore.isSkipContext && $settingsStore.skip_context_enabled) return 'dynamic';
    if ($settingsStore.adjacent_words_enabled) return $settingsStore.adjacent_words_count;
    return 0;
  })();

  $: hasAdjacent = effectiveCount !== 0;

  // ── Compute the actual before/after word slices ───────────────────
  $: beforeWords = (() => {
    if (!hasAdjacent) return [];
    const n = effectiveCount === 'dynamic'
      ? countDynamicWords(idx, -1, containerHalfWidth)
      : (effectiveCount as number);
    const start = Math.max(0, idx - n);
    return words.slice(start, idx);
  })();

  $: afterWords = (() => {
    if (!hasAdjacent) return [];
    const n = effectiveCount === 'dynamic'
      ? countDynamicWords(idx, 1, containerHalfWidth)
      : (effectiveCount as number);
    const end = Math.min(words.length, idx + 1 + n);
    return words.slice(idx + 1, end);
  })();
</script>

<div
  class="rsvp-outer"
  bind:this={containerEl}
>
  <!--
    Hidden span used only for measuring word pixel widths in dynamic mode.
    Invisible, zero-height, matches reading font so measurements are accurate.
  -->
  <span class="measure-span" bind:this={measureEl} aria-hidden="true"></span>

  <div
    class="word-display"
    class:has-adjacent={hasAdjacent}
    aria-label={display}
    aria-live="polite"
    aria-atomic="true"
  >
    <!--
      LEFT HALF — flex row-reverse so the first DOM item (current-seg) anchors to
      the RIGHT edge (next to the ORP letter). Overflow therefore goes LEFT, making
      far-away words disappear at the screen edge rather than bumping into center.
    -->
    <span class="before-orp">
      <span class="current-seg">{parts.before}</span>{#if beforeWords.length > 0}<span class="adjacent">{beforeWords.join(' ')}</span>{/if}
    </span><!--
    --><span class="orp-letter" style={orpStyle}>{parts.orp}</span><!--
    RIGHT HALF — flex row so the first DOM item (current-seg) anchors to the
    LEFT edge. Overflow goes RIGHT, making far-away words disappear at the edge.
    --><span class="after-orp">
      <span class="current-seg">{parts.after}</span>{#if afterWords.length > 0}<span class="adjacent">{afterWords.join(' ')}</span>{/if}
    </span>
  </div>

  <div
    class="word-count"
    class:hidden={$settingsStore.focus_mode}
    aria-label={`Word ${$readerStore.currentIndex + 1} of ${$readerStore.totalWords}`}
  >
    {$readerStore.currentIndex + 1} / {$readerStore.totalWords}
  </div>
</div>

{#if showFocusHint}
  <div class="focus-hint" role="status" aria-live="polite">
    Focus mode on — controls hidden.<br />
    Press <kbd>{focusKey}</kbd> to exit focus mode, or <kbd>{escKey}</kbd> to return to library.
  </div>
{/if}

<style>
  .rsvp-outer {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 180px;
    user-select: none;
  }

  /* Hidden measuring span — must match the reading font exactly */
  .measure-span {
    position: absolute;
    visibility: hidden;
    white-space: nowrap;
    font-family: var(--font-family);
    font-size: var(--font-size);
    pointer-events: none;
    height: 0;
    overflow: hidden;
  }

  .word-display {
    display: flex;
    align-items: baseline;
    font-family: var(--font-family);
    font-size: var(--font-size);
    color: var(--text);
    line-height: 1.2;
    width: 100%;
  }

  /*
   * Equal flex keeps the ORP letter pinned at the horizontal screen center.
   *
   * row-reverse on before-orp: the first DOM child (current-seg) is placed at
   * the RIGHT edge (main-start in row-reverse). Additional words extend leftward.
   * Overflow escapes to the LEFT, so far words disappear without touching center.
   *
   * row on after-orp: the first DOM child (current-seg) is placed at the LEFT
   * edge. Additional words extend rightward. Overflow escapes to the RIGHT.
   */
  .before-orp {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: row-reverse;
    align-items: baseline;
    overflow: hidden;
    gap: 0.3em;
  }

  .orp-letter {
    flex-shrink: 0;
    color: var(--orp);
  }

  .after-orp {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: row;
    align-items: baseline;
    overflow: hidden;
    gap: 0.3em;
  }

  /* Prevent inner spans from shrinking — let them overflow instead */
  :global(.current-seg),
  :global(.adjacent) {
    flex-shrink: 0;
    white-space: nowrap;
  }

  /* Adjacent words — dimmer than the current word */
  :global(.adjacent) {
    color: var(--muted);
  }

  /* Current word fragments stay in full text color */
  :global(.current-seg) {
    color: var(--text);
  }

  /* Edge fade — only applied when adjacent words are visible */
  .has-adjacent .before-orp {
    mask-image: linear-gradient(to right, transparent 0%, black 40%);
    -webkit-mask-image: linear-gradient(to right, transparent 0%, black 40%);
  }

  .has-adjacent .after-orp {
    mask-image: linear-gradient(to left, transparent 0%, black 40%);
    -webkit-mask-image: linear-gradient(to left, transparent 0%, black 40%);
  }

  .word-count {
    position: absolute;
    bottom: 8px;
    right: 16px;
    font-size: 11px;
    color: var(--muted);
    font-family: var(--font-family);
    letter-spacing: 0.05em;
    opacity: 0.6;
  }

  .hidden {
    display: none;
  }

  .focus-hint {
    position: fixed;
    bottom: 36px;
    left: 50%;
    transform: translateX(-50%);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px 20px;
    font-family: var(--font-family);
    font-size: 13px;
    color: var(--muted);
    line-height: 1.6;
    text-align: center;
    white-space: nowrap;
    pointer-events: none;
    z-index: 200;
    animation: hint-fade 5s ease forwards;
  }

  .focus-hint kbd {
    display: inline-block;
    padding: 1px 6px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--bg);
    color: var(--text);
    font-family: var(--font-family);
    font-size: 12px;
  }

  @keyframes hint-fade {
    0%   { opacity: 0; }
    10%  { opacity: 1; }
    75%  { opacity: 1; }
    100% { opacity: 0; }
  }
</style>
