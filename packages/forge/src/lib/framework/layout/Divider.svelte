<script lang="ts">
  /**
   * Draggable divider between two sibling zones.
   *
   * Interactions:
   * - Drag: resizes siblings by adjusting split ratio
   * - Right-click: opens divider context menu
   * - Double-click: resets ratio to 0.5
   * - Keyboard (when focused): arrows adjust ratio, Enter opens context menu
   */

  import type { SplitNode } from '../types';

  interface Props {
    splitNode: SplitNode;
    onResize: (ratio: number) => void;
    onJoin: (keep: 'first' | 'second') => void;
    onSwap: () => void;
    onReset: () => void;
    onContextMenu: (e: MouseEvent) => void;
    /** Called when drag enters/exits merge zone. null clears the preview. */
    onMergePreview?: (direction: 'first' | 'second' | null) => void;
  }

  let { splitNode, onResize, onJoin, onSwap, onReset, onContextMenu, onMergePreview }: Props = $props();

  let dragging = $state(false);
  let splitRect: DOMRect | null = null;

  function findSplitContainer(el: HTMLElement): HTMLElement | null {
    // Walk up to the .forge-split element that owns this divider
    let node: HTMLElement | null = el;
    while (node) {
      if (node.classList.contains('forge-split')) return node;
      node = node.parentElement;
    }
    return null;
  }

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return; // left click only
    dragging = true;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
    const container = findSplitContainer(target);
    splitRect = container?.getBoundingClientRect() ?? null;
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging || !splitRect) return;

    let ratio: number;
    if (splitNode.direction === 'horizontal') {
      ratio = (e.clientX - splitRect.left) / splitRect.width;
    } else {
      ratio = (e.clientY - splitRect.top) / splitRect.height;
    }

    onResize(Math.max(0.1, Math.min(0.9, ratio)));
  }

  function onPointerUp() {
    dragging = false;
    splitRect = null;
  }

  function onDblClick() {
    onReset();
  }

  function onKeyDown(e: KeyboardEvent) {
    const step = 0.02;
    if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      e.preventDefault();
      onResize(splitNode.ratio - step);
    } else if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
      e.preventDefault();
      onResize(splitNode.ratio + step);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      // Synthesise a context menu event
      const el = e.currentTarget as HTMLElement;
      const rect = el.getBoundingClientRect();
      const synthetic = new MouseEvent('contextmenu', {
        clientX: rect.left + rect.width / 2,
        clientY: rect.top + rect.height / 2,
        bubbles: true,
      });
      onContextMenu(synthetic);
    }
  }
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  class="forge-divider"
  class:forge-divider--horizontal={splitNode.direction === 'horizontal'}
  class:forge-divider--vertical={splitNode.direction === 'vertical'}
  class:forge-divider--dragging={dragging}
  role="separator"
  tabindex="0"
  aria-orientation={splitNode.direction}
  aria-valuenow={Math.round(splitNode.ratio * 100)}
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  ondblclick={onDblClick}
  oncontextmenu={(e) => { e.preventDefault(); onContextMenu(e); }}
  onkeydown={onKeyDown}
></div>

<style>
  .forge-divider {
    z-index: var(--forge-z-divider, 10);
    background: var(--forge-border-divider);
    transition: background 0.15s ease;
    flex-shrink: 0;
    width: 100%;
    height: 100%;
  }

  .forge-divider:hover,
  .forge-divider:focus-visible,
  .forge-divider--dragging {
    background: var(--forge-focus-divider-color);
  }

  .forge-divider--horizontal {
    cursor: col-resize;
  }

  .forge-divider--vertical {
    cursor: row-resize;
  }

  .forge-divider:focus-visible {
    outline: none;
    box-shadow: 0 0 0 1px var(--forge-focus-divider-color);
  }
</style>
