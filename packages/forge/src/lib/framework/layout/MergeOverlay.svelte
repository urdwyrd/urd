<script lang="ts">
  /**
   * MergeOverlay — semi-transparent overlay shown when a divider is dragged
   * past the merge threshold, indicating the zone that would be absorbed.
   */

  interface Props {
    direction: 'left' | 'right' | 'up' | 'down';
  }

  let { direction }: Props = $props();

  const arrows: Record<string, string> = {
    left: '\u2190',
    right: '\u2192',
    up: '\u2191',
    down: '\u2193',
  };
</script>

<div class="forge-merge-overlay">
  <span class="forge-merge-overlay__arrow">{arrows[direction]}</span>
  <span class="forge-merge-overlay__label">Merge</span>
</div>

<style>
  .forge-merge-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background: rgba(91, 155, 213, 0.15);
    border: 2px dashed var(--forge-accent-primary, #5b9bd5);
    z-index: 20;
    pointer-events: none;
  }

  .forge-merge-overlay__arrow {
    font-size: 32px;
    color: var(--forge-accent-primary, #5b9bd5);
  }

  .forge-merge-overlay__label {
    font-family: var(--forge-font-family-ui);
    font-size: var(--forge-font-size-sm);
    color: var(--forge-accent-primary, #5b9bd5);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
</style>
