<script lang="ts">
  /**
   * CornerHotspot — invisible corner drag targets for zone splitting.
   *
   * Four 10x10px hotspots at zone corners. On drag, determines split direction
   * from the dominant drag axis (horizontal = horizontal split, vertical = vertical).
   * 15px minimum threshold before committing the split.
   */

  interface Props {
    onSplit: (direction: 'horizontal' | 'vertical') => void;
  }

  let { onSplit }: Props = $props();

  const THRESHOLD = 15;

  let dragging = $state(false);
  let startX = 0;
  let startY = 0;

  function handlePointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    e.stopPropagation();
    e.preventDefault();
    dragging = true;
    startX = e.clientX;
    startY = e.clientY;
    (e.target as Element).setPointerCapture(e.pointerId);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!dragging) return;
    const dx = Math.abs(e.clientX - startX);
    const dy = Math.abs(e.clientY - startY);

    if (dx >= THRESHOLD || dy >= THRESHOLD) {
      dragging = false;
      (e.target as Element).releasePointerCapture(e.pointerId);
      // Horizontal drag → horizontal split (left/right), vertical drag → vertical split (top/bottom)
      onSplit(dx >= dy ? 'horizontal' : 'vertical');
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!dragging) return;
    dragging = false;
    (e.target as Element).releasePointerCapture(e.pointerId);
  }
</script>

<!-- Four corner hotspots -->
<div
  class="forge-corner-hotspot forge-corner-hotspot--tl"
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  role="separator"
  aria-label="Drag to split zone"
></div>
<div
  class="forge-corner-hotspot forge-corner-hotspot--tr"
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  role="separator"
  aria-label="Drag to split zone"
></div>
<div
  class="forge-corner-hotspot forge-corner-hotspot--bl"
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  role="separator"
  aria-label="Drag to split zone"
></div>
<div
  class="forge-corner-hotspot forge-corner-hotspot--br"
  onpointerdown={handlePointerDown}
  onpointermove={handlePointerMove}
  onpointerup={handlePointerUp}
  role="separator"
  aria-label="Drag to split zone"
></div>

<style>
  .forge-corner-hotspot {
    position: absolute;
    width: 10px;
    height: 10px;
    z-index: 10;
    cursor: crosshair;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .forge-corner-hotspot:hover {
    opacity: 1;
    background: var(--forge-accent-primary, #5b9bd5);
    border-radius: 2px;
  }

  .forge-corner-hotspot--tl { top: 0; left: 0; }
  .forge-corner-hotspot--tr { top: 0; right: 0; }
  .forge-corner-hotspot--bl { bottom: 0; left: 0; }
  .forge-corner-hotspot--br { bottom: 0; right: 0; }
</style>
