<script lang="ts">
  /**
   * SplitContainer — renders two children with a draggable divider.
   */

  import type { SplitNode } from '../types';
  import type { Snippet } from 'svelte';
  import MergeOverlay from './MergeOverlay.svelte';

  interface Props {
    splitNode: SplitNode;
    first: Snippet;
    second: Snippet;
    divider: Snippet;
    /** Which pane is being targeted for merge (set by Divider via onMergePreview). */
    mergePreview?: 'first' | 'second' | null;
  }

  let { splitNode, first, second, divider, mergePreview = null }: Props = $props();

  let firstSize = $derived(`${splitNode.ratio * 100}%`);
  let secondSize = $derived(`${(1 - splitNode.ratio) * 100}%`);

  let mergeOverlayDirection = $derived.by(() => {
    if (!mergePreview) return null;
    if (splitNode.direction === 'horizontal') {
      return mergePreview === 'first' ? 'right' as const : 'left' as const;
    }
    return mergePreview === 'first' ? 'down' as const : 'up' as const;
  });
</script>

<div
  class="forge-split"
  class:forge-split--horizontal={splitNode.direction === 'horizontal'}
  class:forge-split--vertical={splitNode.direction === 'vertical'}
  data-split-id={splitNode.id}
>
  <div class="forge-split__pane forge-split__pane--first" style:flex-basis={firstSize}>
    {@render first()}
    {#if mergePreview === 'first' && mergeOverlayDirection}
      <MergeOverlay direction={mergeOverlayDirection} />
    {/if}
  </div>
  <div class="forge-split__divider-slot">
    {@render divider()}
  </div>
  <div class="forge-split__pane forge-split__pane--second" style:flex-basis={secondSize}>
    {@render second()}
    {#if mergePreview === 'second' && mergeOverlayDirection}
      <MergeOverlay direction={mergeOverlayDirection} />
    {/if}
  </div>
</div>

<style>
  .forge-split {
    display: flex;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }

  .forge-split--horizontal {
    flex-direction: row;
  }

  .forge-split--vertical {
    flex-direction: column;
  }

  .forge-split__pane {
    overflow: hidden;
    min-width: 0;
    min-height: 0;
    flex-shrink: 1;
    flex-grow: 0;
    position: relative;
    contain: paint;
  }

  .forge-split__divider-slot {
    position: relative;
    flex-shrink: 0;
  }

  .forge-split--horizontal > .forge-split__divider-slot {
    width: 4px;
  }

  .forge-split--vertical > .forge-split__divider-slot {
    height: 4px;
  }
</style>
