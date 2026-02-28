<script lang="ts">
  /**
   * InsightSection — collapsible section wrapper with count badges and filter slot.
   *
   * More feature-rich than InspectorSection: supports inline badge pills,
   * optional filter controls snippet, and external state management.
   */

  import type { Snippet } from 'svelte';

  interface Badge {
    label: string;
    count: number;
    colour?: string;
  }

  interface Props {
    sectionId: string;
    title: string;
    collapsed: boolean;
    onToggle: (sectionId: string) => void;
    badges?: Badge[];
    filterControls?: Snippet;
    children: Snippet;
  }

  let { sectionId, title, collapsed, onToggle, badges = [], filterControls, children }: Props = $props();

  function toggle(): void {
    onToggle(sectionId);
  }

  function handleKeydown(e: KeyboardEvent): void {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      toggle();
    }
  }
</script>

<div class="forge-insight-section" data-section-id={sectionId}>
  <div
    class="forge-insight-section__header"
    role="button"
    tabindex="0"
    aria-expanded={!collapsed}
    onclick={toggle}
    onkeydown={handleKeydown}
  >
    <span class="forge-insight-section__toggle">{collapsed ? '\u25B8' : '\u25BE'}</span>
    <span class="forge-insight-section__title">{title}</span>
    {#each badges as badge}
      <span
        class="forge-insight-section__badge"
        style:background-color={badge.colour ? `color-mix(in srgb, ${badge.colour} 15%, transparent)` : undefined}
        style:color={badge.colour}
      >
        {badge.label ? `${badge.label} ${badge.count}` : badge.count}
      </span>
    {/each}
    {#if filterControls && !collapsed}
      <span class="forge-insight-section__filter-slot">
        {@render filterControls()}
      </span>
    {/if}
  </div>
  {#if !collapsed}
    <div class="forge-insight-section__body">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .forge-insight-section {
    margin-bottom: 0;
  }

  .forge-insight-section__header {
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
    height: 28px;
    padding: 0 var(--forge-space-sm);
    background-color: var(--forge-bg-secondary);
    cursor: pointer;
    user-select: none;
    border-bottom: 1px solid var(--forge-border-zone);
  }

  .forge-insight-section__header:hover {
    background-color: var(--forge-table-row-hover);
  }

  .forge-insight-section__toggle {
    flex-shrink: 0;
    width: 10px;
    font-size: 10px;
    color: var(--forge-text-muted);
  }

  .forge-insight-section__title {
    font-family: var(--forge-font-family-ui);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--forge-text-muted);
  }

  .forge-insight-section__badge {
    font-family: var(--forge-font-family-mono);
    font-size: 10px;
    padding: 0 var(--forge-space-xs);
    border-radius: var(--forge-radius-sm);
    background-color: color-mix(in srgb, var(--forge-text-muted) 15%, transparent);
    color: var(--forge-text-secondary);
    line-height: 18px;
  }

  .forge-insight-section__filter-slot {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--forge-space-xs);
  }

  .forge-insight-section__body {
    overflow: hidden;
  }
</style>
