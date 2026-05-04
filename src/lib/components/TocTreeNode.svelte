<script lang="ts">
  import { readerStore } from '../stores/reader';
  import type { TocEntry } from '../types';
  import TocTreeNode from './TocTreeNode.svelte';

  export let entry: TocEntry;
  export let depth = 0;
  export let activeWordIndex: number;

  let expanded = true;

  $: isActive = entry.word_index === activeWordIndex;

  function go() {
    readerStore.seekTo(entry.word_index, true);
  }
</script>

<div class="toc-row" style="padding-left: {depth * 14}px">
  {#if entry.children.length > 0}
    <button
      type="button"
      class="toc-chevron"
      aria-expanded={expanded}
      aria-label={expanded ? 'Collapse section' : 'Expand section'}
      on:click={() => (expanded = !expanded)}
    >
      {expanded ? '▼' : '▶'}
    </button>
  {:else}
    <span class="toc-chevron-spacer" aria-hidden="true"></span>
  {/if}
  <button type="button" class="toc-title" class:active={isActive} on:click={go}>
    {entry.title}
  </button>
</div>
{#if entry.children.length > 0 && expanded}
  {#each entry.children as child}
    <TocTreeNode entry={child} depth={depth + 1} activeWordIndex={activeWordIndex} />
  {/each}
{/if}

<style>
  .toc-row {
    display: flex;
    align-items: flex-start;
    gap: 4px;
    min-height: 28px;
  }

  .toc-chevron {
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    padding: 0;
    margin-top: 2px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    font-size: 10px;
    line-height: 1;
    border-radius: 4px;
  }

  .toc-chevron:hover {
    background: var(--surface);
    color: var(--text);
  }

  .toc-chevron-spacer {
    flex-shrink: 0;
    width: 22px;
  }

  .toc-title {
    flex: 1;
    text-align: left;
    border: none;
    background: none;
    color: var(--text);
    font-size: 13px;
    line-height: 1.35;
    padding: 4px 6px;
    border-radius: 6px;
    cursor: pointer;
    font-family: var(--font-family);
  }

  .toc-title:hover {
    background: var(--surface);
  }

  .toc-title.active {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
    color: var(--accent);
    font-weight: 600;
  }
</style>
