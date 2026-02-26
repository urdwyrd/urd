<script lang="ts">
  /** Standard loading/spinner state for views awaiting async data. */
</script>

<div class="forge-zone-loading">
  <div class="forge-zone-loading__spinner"></div>
  <p class="forge-zone-loading__text">Loading…</p>
</div>

<style>
  .forge-zone-loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--forge-space-md);
    color: var(--forge-text-muted);
  }

  .forge-zone-loading__spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--forge-border-zone);
    border-top-color: var(--forge-accent-primary);
    border-radius: 50%;
    animation: forge-spin 0.8s linear infinite;
  }

  .forge-zone-loading__text {
    font-size: var(--forge-font-size-sm);
  }

  @keyframes forge-spin {
    to { transform: rotate(360deg); }
  }

  @media (prefers-reduced-motion: reduce) {
    .forge-zone-loading__spinner {
      animation: none;
      border-top-color: var(--forge-text-muted);
    }
  }
</style>
