<script lang="ts">
  import type { FactSet } from './compiler-bridge';

  interface Props {
    facts: FactSet;
    onDiagnosticClick?: (line: number, col: number) => void;
  }

  let { facts, onDiagnosticClick }: Props = $props();

  // --- Collapsed state per section ---
  let collapsed: Record<string, boolean> = $state({});

  function toggle(section: string) {
    collapsed[section] = !collapsed[section];
  }

  function isOpen(section: string): boolean {
    return !collapsed[section];
  }

  // --- Helpers ---

  function siteLabel(site: { kind: string; id: string }): string {
    return `${site.kind}: ${site.id}`;
  }

  function clickSpan(span: { start_line: number; start_col: number }) {
    onDiagnosticClick?.(span.start_line, span.start_col);
  }

  // --- Counts ---
  let total = $derived(
    facts.exits.length + facts.choices.length + facts.reads.length +
    facts.writes.length + facts.jumps.length + facts.rules.length
  );
</script>

<div class="factset-view">
  <div class="fact-summary">
    <span class="fact-total">{total} facts</span>
    <span class="fact-counts">
      {facts.exits.length} exits
      · {facts.choices.length} choices
      · {facts.reads.length} reads
      · {facts.writes.length} writes
      · {facts.jumps.length} jumps
      · {facts.rules.length} rules
    </span>
  </div>

  <!-- Exits -->
  {#if facts.exits.length > 0}
    <button class="section-header" onclick={() => toggle('exits')}>
      <span class="section-toggle">{isOpen('exits') ? '▾' : '▸'}</span>
      <span class="section-label">Exits</span>
      <span class="section-count">{facts.exits.length}</span>
    </button>
    {#if isOpen('exits')}
      <div class="section-body">
        {#each facts.exits as exit}
          <button class="fact-row" onclick={() => clickSpan(exit.span)}>
            <span class="fact-from">{exit.from_location}</span>
            <span class="fact-arrow">→</span>
            <span class="fact-to">{exit.to_location}</span>
            <span class="fact-via">via {exit.exit_name}</span>
            {#if exit.is_conditional}
              <span class="fact-tag conditional">conditional</span>
              {#if exit.guard_reads.length > 0}
                <span class="fact-detail">{exit.guard_reads.length} guard{exit.guard_reads.length !== 1 ? 's' : ''}</span>
              {/if}
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Choices -->
  {#if facts.choices.length > 0}
    <button class="section-header" onclick={() => toggle('choices')}>
      <span class="section-toggle">{isOpen('choices') ? '▾' : '▸'}</span>
      <span class="section-label">Choices</span>
      <span class="section-count">{facts.choices.length}</span>
    </button>
    {#if isOpen('choices')}
      <div class="section-body">
        {#each facts.choices as choice}
          <button class="fact-row" onclick={() => clickSpan(choice.span)}>
            <span class="fact-choice-marker">{choice.sticky ? '+' : '*'}</span>
            <span class="fact-choice-label">{choice.label}</span>
            <span class="fact-section">in {choice.section}</span>
            {#if choice.condition_reads.length > 0}
              <span class="fact-detail">{choice.condition_reads.length} cond</span>
            {/if}
            {#if choice.effect_writes.length > 0}
              <span class="fact-detail">{choice.effect_writes.length} effect{choice.effect_writes.length !== 1 ? 's' : ''}</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Reads -->
  {#if facts.reads.length > 0}
    <button class="section-header" onclick={() => toggle('reads')}>
      <span class="section-toggle">{isOpen('reads') ? '▾' : '▸'}</span>
      <span class="section-label">Property Reads</span>
      <span class="section-count">{facts.reads.length}</span>
    </button>
    {#if isOpen('reads')}
      <div class="section-body">
        {#each facts.reads as read}
          <button class="fact-row" onclick={() => clickSpan(read.span)}>
            <span class="fact-prop">{read.entity_type}.{read.property}</span>
            <span class="fact-op">{read.operator}</span>
            <span class="fact-value">{read.value_literal}</span>
            <span class="fact-site-label">{siteLabel(read.site)}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Writes -->
  {#if facts.writes.length > 0}
    <button class="section-header" onclick={() => toggle('writes')}>
      <span class="section-toggle">{isOpen('writes') ? '▾' : '▸'}</span>
      <span class="section-label">Property Writes</span>
      <span class="section-count">{facts.writes.length}</span>
    </button>
    {#if isOpen('writes')}
      <div class="section-body">
        {#each facts.writes as write}
          <button class="fact-row" onclick={() => clickSpan(write.span)}>
            <span class="fact-prop">{write.entity_type}.{write.property}</span>
            <span class="fact-op">{write.operator}</span>
            <span class="fact-value">{write.value_expr}</span>
            <span class="fact-site-label">{siteLabel(write.site)}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Jumps -->
  {#if facts.jumps.length > 0}
    <button class="section-header" onclick={() => toggle('jumps')}>
      <span class="section-toggle">{isOpen('jumps') ? '▾' : '▸'}</span>
      <span class="section-label">Jumps</span>
      <span class="section-count">{facts.jumps.length}</span>
    </button>
    {#if isOpen('jumps')}
      <div class="section-body">
        {#each facts.jumps as jump}
          <button class="fact-row" onclick={() => clickSpan(jump.span)}>
            <span class="fact-from">{jump.from_section}</span>
            <span class="fact-arrow">→</span>
            {#if jump.target.kind === 'end'}
              <span class="fact-to end-marker">end</span>
            {:else}
              <span class="fact-to">{jump.target.id}</span>
            {/if}
            <span class="fact-tag">{jump.target.kind}</span>
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Rules -->
  {#if facts.rules.length > 0}
    <button class="section-header" onclick={() => toggle('rules')}>
      <span class="section-toggle">{isOpen('rules') ? '▾' : '▸'}</span>
      <span class="section-label">Rules</span>
      <span class="section-count">{facts.rules.length}</span>
    </button>
    {#if isOpen('rules')}
      <div class="section-body">
        {#each facts.rules as rule}
          <button class="fact-row" onclick={() => clickSpan(rule.span)}>
            <span class="fact-rule-id">{rule.rule_id}</span>
            {#if rule.condition_reads.length > 0}
              <span class="fact-detail">{rule.condition_reads.length} cond</span>
            {/if}
            {#if rule.effect_writes.length > 0}
              <span class="fact-detail">{rule.effect_writes.length} effect{rule.effect_writes.length !== 1 ? 's' : ''}</span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  <!-- Empty state -->
  {#if total === 0}
    <div class="fact-empty">No facts extracted.</div>
  {/if}
</div>

<style>
  .factset-view {
    flex: 1;
    overflow: auto;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--dim);
  }

  /* --- Summary --- */

  .fact-summary {
    padding: 8px 12px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: baseline;
    gap: 10px;
  }

  .fact-total {
    font-family: var(--display);
    font-weight: 600;
    font-size: 12px;
    color: var(--text);
  }

  .fact-counts {
    font-size: 11px;
    color: var(--faint);
  }

  /* --- Section headers --- */

  .section-header {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    padding: 5px 12px;
    border: none;
    border-bottom: 1px solid var(--border);
    background: var(--raise);
    color: var(--text);
    font-family: var(--display);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.03em;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .section-header:hover {
    background: var(--surface);
  }

  .section-toggle {
    color: var(--faint);
    width: 10px;
    font-size: 10px;
  }

  .section-label {
    flex: 1;
    text-transform: uppercase;
  }

  .section-count {
    color: var(--faint);
    font-family: var(--mono);
    font-size: 11px;
    font-weight: 400;
  }

  /* --- Fact rows --- */

  .section-body {
    border-bottom: 1px solid var(--border);
  }

  .fact-row {
    display: flex;
    align-items: baseline;
    gap: 6px;
    width: 100%;
    padding: 4px 12px 4px 28px;
    border: none;
    background: none;
    color: var(--dim);
    font-family: var(--mono);
    font-size: 12px;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
    flex-wrap: wrap;
  }

  .fact-row:hover {
    background: var(--surface);
  }

  /* --- Tokens --- */

  .fact-from {
    color: var(--amber-light);
  }

  .fact-arrow {
    color: var(--faint);
  }

  .fact-to {
    color: var(--green-light);
  }

  .fact-via {
    color: var(--faint);
    font-size: 11px;
  }

  .fact-tag {
    font-size: 10px;
    padding: 0 4px;
    border-radius: 3px;
    background: color-mix(in srgb, var(--border) 50%, transparent);
    color: var(--faint);
  }

  .fact-tag.conditional {
    background: color-mix(in srgb, var(--amber-light) 15%, transparent);
    color: var(--amber-light);
  }

  .fact-detail {
    font-size: 10px;
    color: var(--faint);
  }

  .fact-choice-marker {
    color: var(--gold);
    font-weight: 700;
    width: 10px;
  }

  .fact-choice-label {
    color: var(--text);
  }

  .fact-section {
    color: var(--faint);
    font-size: 11px;
  }

  .fact-prop {
    color: var(--amber-light);
  }

  .fact-op {
    color: var(--faint);
  }

  .fact-value {
    color: var(--green-light);
  }

  .fact-site-label {
    font-size: 10px;
    color: var(--faint);
    margin-left: auto;
  }

  .fact-rule-id {
    color: var(--purple);
    font-weight: 500;
  }

  .end-marker {
    color: var(--rose);
    font-style: italic;
  }

  .fact-empty {
    padding: 24px 12px;
    text-align: center;
    color: var(--faint);
    font-family: var(--body);
    font-size: 13px;
  }
</style>
