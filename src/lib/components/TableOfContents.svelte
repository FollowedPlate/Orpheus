<script lang="ts">
  import { readerStore } from '../stores/reader';
  import type { TocEntry } from '../types';
  import TocTreeNode from './TocTreeNode.svelte';

  function computeActiveWordIndex(entries: TocEntry[], currentIndex: number): number {
    let best = -1;
    function walk(nodes: TocEntry[]) {
      for (const n of nodes) {
        if (n.word_index <= currentIndex && n.word_index > best) {
          best = n.word_index;
        }
        walk(n.children);
      }
    }
    walk(entries);
    return best;
  }

  $: activeWordIndex = computeActiveWordIndex($readerStore.toc, $readerStore.currentIndex);
</script>

<aside
  class="toc-panel"
  class:collapsed={!$readerStore.tocOpen}
  aria-hidden={!$readerStore.tocOpen}
>
  <div class="toc-scroll">
    <h3 class="toc-heading">Contents</h3>
    {#if $readerStore.toc.length === 0}
      <p class="toc-empty">No outline for this book.</p>
    {:else}
      {#each $readerStore.toc as entry}
        <TocTreeNode {entry} depth={0} activeWordIndex={activeWordIndex} />
      {/each}
    {/if}
  </div>
</aside>

<style>
  .toc-panel {
    flex-shrink: 0;
    width: min(300px, 42vw);
    max-width: 100%;
    border-right: 1px solid var(--border);
    background: var(--bg);
    overflow: hidden;
    transition:
      width 0.2s ease,
      opacity 0.2s ease,
      border-color 0.2s ease;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .toc-panel.collapsed {
    width: 0;
    opacity: 0;
    border-right-color: transparent;
    pointer-events: none;
  }

  .toc-scroll {
    flex: 1;
    overflow: auto;
    padding: 12px 10px 16px 12px;
    min-height: 0;
  }

  .toc-heading {
    margin: 0 0 10px 0;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .toc-empty {
    margin: 8px 0 0 0;
    font-size: 13px;
    color: var(--muted);
    line-height: 1.4;
  }
</style>
