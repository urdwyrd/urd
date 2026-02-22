<script lang="ts">
  import { onMount } from 'svelte';

  interface Phase {
    title: string;
    status: 'complete' | 'finalising' | 'active' | 'next';
    subtitle: string;
    order: number;
    description: string;
    link: string | null;
    linkLabel: string | null;
    progress: number | null;
  }

  interface Props {
    label?: string;
    heading?: string;
    intro?: string;
    note?: string;
  }

  let {
    label = 'Project Timeline',
    heading = 'From Spec to System',
    intro = 'The v0.1 and v1 specifications are locked. <a href="https://urd.dev" class="timeline-link">urd.dev</a> is live with a documentation pipeline publishing the spec, reference cards, and architecture briefs. Now we formalise, prove, and build.',
    note = '<strong>Specs are locked</strong> — the v0.1 and v1 specifications are frozen for implementation. Any gaps discovered during formalisation or validation will be tracked as amendments, not inline edits, preserving a stable reference point.',
  }: Props = $props();

  let phases: Phase[] = $state([]);
  let loaded = $state(false);

  onMount(async () => {
    const res = await fetch('/timeline.json');
    if (res.ok) {
      phases = await res.json();
    }
    loaded = true;
  });

  function statusIndicator(status: string): string {
    if (status === 'complete') return '✓';
    if (status === 'finalising') return '◆';
    if (status === 'active') return '◆';
    return '';
  }
</script>

<section class="timeline">
  <header class="timeline-header">
    <span class="timeline-label">{label}</span>
    <h2 class="timeline-heading">{heading}</h2>
    <p class="timeline-intro">{@html intro}</p>
  </header>

  {#if loaded && phases.length > 0}
    <div class="phases">
      {#each phases as phase}
        <div class="phase phase-{phase.status}">
          <p class="phase-status">
            {#if statusIndicator(phase.status)}
              <span class="phase-indicator">{statusIndicator(phase.status)}</span>
            {/if}
            {phase.status.toUpperCase()}
          </p>
          <h3 class="phase-title">{phase.title}</h3>
          <p class="phase-subtitle">{phase.subtitle}</p>
          <p class="phase-desc">
            {phase.description}
            {#if phase.link}
              <a class="phase-link" href={phase.link}>{phase.linkLabel ?? 'Learn more →'}</a>
            {/if}
          </p>
          {#if phase.progress != null}
            <div class="phase-progress">
              <div class="progress-track">
                <div class="progress-fill" style="width: {phase.progress}%"></div>
              </div>
              <span class="progress-label">{phase.progress}%</span>
            </div>
          {/if}
        </div>
      {/each}
    </div>

    {#if note}
      <aside class="timeline-note">
        <p>{@html note}</p>
      </aside>
    {/if}
  {/if}
</section>

<style>
  .timeline {
    width: 100%;
  }

  .timeline-header {
    margin-bottom: 24px;
  }

  .timeline-label {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--gold-dim);
    display: block;
    margin-bottom: 6px;
  }

  .timeline-heading {
    font-family: var(--display);
    font-size: clamp(22px, 3.5vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 6px;
  }

  .timeline-intro {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    line-height: 1.6;
    max-width: 680px;
  }

  .timeline-intro :global(.timeline-link) {
    color: var(--dim);
    text-decoration: underline;
    text-underline-offset: 2px;
    text-decoration-color: var(--border-light);
    transition: color 0.15s ease;
  }

  .timeline-intro :global(.timeline-link:hover) {
    color: var(--text);
  }

  /* ── Phase cards ── */
  .phases {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
  }

  .phase {
    background: var(--raise);
    padding: 22px 24px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .phase-status {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 1.5px;
    text-transform: uppercase;
    display: flex;
    align-items: center;
    gap: 5px;
  }

  .phase-complete .phase-status {
    color: var(--green-light);
  }

  .phase-finalising .phase-status {
    color: var(--gold);
  }

  .phase-active .phase-status {
    color: var(--gold);
  }

  .phase-next .phase-status {
    color: var(--faint);
  }

  .phase-indicator {
    font-size: 8px;
  }

  .phase-finalising .phase-indicator {
    font-size: 9px;
  }

  .phase-active .phase-indicator {
    font-size: 9px;
  }

  .phase-title {
    font-family: var(--display);
    font-size: clamp(18px, 2.2vw, 22px);
    font-weight: 700;
    color: var(--text);
    line-height: 1.2;
  }

  .phase-subtitle {
    font-family: var(--body);
    font-size: 15px;
    font-style: italic;
    color: var(--dim);
    line-height: 1.4;
  }

  .phase-desc {
    font-family: var(--body);
    font-size: 15px;
    color: var(--faint);
    line-height: 1.55;
    margin-top: 4px;
  }

  .phase-progress {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
  }

  .progress-track {
    flex: 1;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background: var(--gold);
    border-radius: 2px;
    transition: width 0.3s ease;
  }

  .phase-complete .progress-fill {
    background: var(--green-light);
  }

  .progress-label {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--gold-dim);
    flex-shrink: 0;
    min-width: 32px;
    text-align: right;
  }

  .phase-complete .progress-label {
    color: var(--green-light);
  }

  .phase-link {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 500;
    color: var(--gold-dim);
    text-decoration: none;
    white-space: nowrap;
    transition: color 0.15s ease;
  }

  .phase-link:hover {
    color: var(--gold);
  }

  .phase-link:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
    border-radius: 2px;
  }

  /* ── Callout note ── */
  .timeline-note {
    margin-top: 24px;
    padding: 20px 24px;
    border-left: 3px solid var(--gold);
    background: var(--raise);
    border-radius: 0 8px 8px 0;
  }

  .timeline-note p {
    font-family: var(--body);
    font-size: 15px;
    color: var(--dim);
    line-height: 1.6;
    margin: 0;
  }

  .timeline-note :global(strong) {
    font-weight: 700;
    color: var(--text);
  }

  /* ── Responsive ── */
  @media (max-width: 980px) {
    .phases {
      grid-template-columns: repeat(2, 1fr);
    }
  }

  @media (max-width: 640px) {
    .phases {
      grid-template-columns: 1fr;
    }
  }
</style>
