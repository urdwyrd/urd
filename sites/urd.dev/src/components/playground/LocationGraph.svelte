<script lang="ts">
  import type { FactSet, Diagnostic } from './compiler-bridge';
  import GraphRenderer from './GraphRenderer.svelte';
  import { buildLocationGraph } from './transform-location';

  interface Props {
    facts: FactSet;
    worldJson: string | null;
    diagnostics: Diagnostic[];
  }

  let { facts, worldJson, diagnostics }: Props = $props();

  let graph = $derived(buildLocationGraph(facts, worldJson, diagnostics));
</script>

{#if facts.exits.length === 0}
  <div class="graph-empty-state">No exits in this world.</div>
{:else}
  <GraphRenderer {graph} direction="LR" />
{/if}

<style>
  .graph-empty-state {
    padding: 24px 12px;
    text-align: center;
    color: var(--faint);
    font-family: var(--body);
    font-size: 13px;
  }
</style>
