<script lang="ts">
  /**
   * PlaceholderColour — solid background colour assigned by zone ID hash.
   * Makes it immediately obvious which zone is which during layout testing.
   */

  interface Props {
    zoneId: string;
    zoneTypeId: string;
    state: unknown;
    onStateChange: (state: unknown) => void;
  }

  let { zoneId }: Props = $props();

  // Deterministic colour from zone ID
  function hashColour(id: string): string {
    let hash = 0;
    for (let i = 0; i < id.length; i++) {
      hash = ((hash << 5) - hash + id.charCodeAt(i)) | 0;
    }
    const hue = Math.abs(hash) % 360;
    return `hsl(${hue}, 40%, 25%)`;
  }

  let colour = $derived(hashColour(zoneId));
</script>

<div class="forge-placeholder-colour" style:background-color={colour}>
  <span class="forge-placeholder-colour__id forge-selectable">{zoneId}</span>
</div>

<style>
  .forge-placeholder-colour {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
  }

  .forge-placeholder-colour__id {
    font-family: var(--forge-font-family-mono);
    font-size: var(--forge-font-size-sm);
    color: rgba(255, 255, 255, 0.5);
  }
</style>
