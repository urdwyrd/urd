<script lang="ts">
  /**
   * FloatingDialog — Blender-style floating panel overlay.
   *
   * Renders a centred, draggable dialog over the workspace.
   * Escape or clicking the backdrop closes it.
   */

  import { onMount } from 'svelte';
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    width?: string;
    height?: string;
    onClose: () => void;
    children: Snippet;
  }

  let { title, width = '700px', height = '500px', onClose, children }: Props = $props();

  let dialogEl: HTMLDivElement | undefined = $state(undefined);
  let offsetX = $state(0);
  let offsetY = $state(0);
  let dragging = $state(false);
  let posX = $state<number | null>(null);
  let posY = $state<number | null>(null);

  function handleTitleBarPointerDown(e: PointerEvent): void {
    if (!dialogEl) return;
    dragging = true;
    const rect = dialogEl.getBoundingClientRect();
    offsetX = e.clientX - rect.left;
    offsetY = e.clientY - rect.top;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent): void {
    if (!dragging) return;
    posX = e.clientX - offsetX;
    posY = e.clientY - offsetY;
  }

  function handlePointerUp(): void {
    dragging = false;
  }

  function handleBackdropClick(e: MouseEvent): void {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      onClose();
    }
  }

  onMount(() => {
    dialogEl?.focus();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="forge-dialog__backdrop"
  onmousedown={handleBackdropClick}
  onkeydown={handleKeydown}
>
  <div
    bind:this={dialogEl}
    class="forge-dialog"
    style:width={width}
    style:height={height}
    style:left={posX !== null ? `${posX}px` : undefined}
    style:top={posY !== null ? `${posY}px` : undefined}
    style:position={posX !== null ? 'fixed' : undefined}
    style:transform={posX === null ? undefined : 'none'}
    tabindex="-1"
    role="dialog"
    aria-label={title}
  >
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="forge-dialog__titlebar"
      onpointerdown={handleTitleBarPointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
    >
      <span class="forge-dialog__title">{title}</span>
      <button class="forge-dialog__close" onclick={onClose} aria-label="Close">✕</button>
    </div>
    <div class="forge-dialog__body">
      {@render children()}
    </div>
  </div>
</div>

<style>
  .forge-dialog__backdrop {
    position: fixed;
    inset: 0;
    z-index: 9000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.35);
  }

  .forge-dialog {
    display: flex;
    flex-direction: column;
    background: var(--forge-bg-secondary);
    border: 1px solid var(--forge-border-zone);
    border-radius: var(--forge-radius-md);
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    overflow: hidden;
    outline: none;
    max-width: 90vw;
    max-height: 85vh;
  }

  .forge-dialog__titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--forge-space-sm) var(--forge-space-md);
    background: var(--forge-bg-tertiary);
    border-bottom: 1px solid var(--forge-border-zone);
    cursor: grab;
    user-select: none;
    flex-shrink: 0;
  }

  .forge-dialog__titlebar:active {
    cursor: grabbing;
  }

  .forge-dialog__title {
    font-size: var(--forge-font-size-sm);
    font-weight: 600;
    color: var(--forge-text-primary);
  }

  .forge-dialog__close {
    border: none;
    background: transparent;
    color: var(--forge-text-muted);
    cursor: pointer;
    font-size: var(--forge-font-size-sm);
    padding: 2px 6px;
    border-radius: var(--forge-radius-sm);
    line-height: 1;
  }

  .forge-dialog__close:hover {
    color: var(--forge-text-primary);
    background: rgba(255, 255, 255, 0.1);
  }

  .forge-dialog__body {
    flex: 1;
    overflow: hidden;
  }
</style>
