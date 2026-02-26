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
  }

  let { splitNode, onResize, onJoin, onSwap, onReset, onContextMenu }: Props = $props();

  let dragging = $state(false);
  let containerRect: DOMRect | null = null;

  function onPointerDown(e: PointerEvent) {
    if (e.button !== 0) return; // left click only
    dragging = true;
    const target = e.currentTarget as HTMLElement;
    target.setPointerCapture(e.pointerId);
    containerRect = target.parentElement?.getBoundingClientRect() ?? null;
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging || !containerRect) return;

    let ratio: number;
    if (splitNode.direction === 'horizontal') {
      ratio = (e.clientX - containerRect.left) / containerRect.width;
    } else {
      ratio = (e.clientY - containerRect.top) / containerRect.height;
    }

    onResize(Math.max(0.1, Math.min(0.9, ratio)));
  }

  function onPointerUp() {
    dragging = false;
    containerRect = null;
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
    position: absolute;
    z-index: var(--forge-z-divider, 10);
    background: var(--forge-border-divider);
    transition: background 0.15s ease;
    flex-shrink: 0;
  }

  .forge-divider:hover,
  .forge-divider:focus-visible,
  .forge-divider--dragging {
    background: var(--forge-focus-divider-color);
  }

  .forge-divider--horizontal {
    width: 4px;
    height: 100%;
    cursor: col-resize;
  }

  .forge-divider--vertical {
    width: 100%;
    height: 4px;
    cursor: row-resize;
  }

  .forge-divider:focus-visible {
    outline: none;
    box-shadow: 0 0 0 1px var(--forge-focus-divider-color);
  }
</style>
