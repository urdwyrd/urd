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

  interface Era {
    title: string;
    subtitle: string;
    status: 'complete' | 'active' | 'next';
    order: number;
    slug: string;
    description: string;
    phases: Phase[];
  }

  interface Props {
    label?: string;
  }

  let {
    label = 'Project Timeline',
  }: Props = $props();

  let eras: Era[] = $state([]);
  let currentEraIndex = $state(0);
  let loaded = $state(false);

  let currentEra = $derived(eras[currentEraIndex] ?? null);
  let isFirstEra = $derived(currentEraIndex === 0);
  let isLastEra = $derived(currentEraIndex === eras.length - 1);

  function defaultEraIndex(list: Era[]): number {
    const activeIdx = list.findLastIndex((e) => e.status === 'active');
    if (activeIdx >= 0) return activeIdx;
    const completeIdx = list.findLastIndex((e) => e.status === 'complete');
    if (completeIdx >= 0) return completeIdx;
    return Math.max(0, list.length - 1);
  }

  function prevEra() {
    if (currentEraIndex > 0) currentEraIndex--;
  }

  function nextEra() {
    if (currentEraIndex < eras.length - 1) currentEraIndex++;
  }

  function statusIndicator(status: string): string {
    if (status === 'complete') return '✓';
    if (status === 'finalising' || status === 'active') return '◆';
    return '';
  }

  onMount(async () => {
    const res = await fetch('/timeline.json');
    if (res.ok) {
      eras = await res.json();
      currentEraIndex = defaultEraIndex(eras);
    }
    loaded = true;
  });
</script>

<section class="timeline">
  <header class="timeline-header">
    <span class="timeline-label">{label}</span>

    {#if loaded && currentEra}
      <div class="era-nav">
        <button
          class="era-nav-btn"
          onclick={prevEra}
          disabled={isFirstEra}
          aria-label="Previous era"
        >←</button>

        <div class="era-title-group">
          <h2 class="timeline-heading">{currentEra.title}</h2>
          <p class="timeline-intro">{currentEra.subtitle}</p>
        </div>

        <button
          class="era-nav-btn"
          onclick={nextEra}
          disabled={isLastEra}
          aria-label="Next era"
        >→</button>
      </div>

      {#if eras.length > 1}
        <div class="era-pips" aria-label="Era {currentEraIndex + 1} of {eras.length}">
          {#each eras as era, i}
            <button
              class="era-pip"
              class:era-pip-active={i === currentEraIndex}
              class:era-pip-complete={era.status === 'complete'}
              onclick={() => currentEraIndex = i}
              aria-label={era.title}
              aria-current={i === currentEraIndex ? 'step' : undefined}
            ></button>
          {/each}
        </div>
      {/if}
    {/if}
  </header>

  {#if loaded && currentEra}
    {#if currentEra.phases.length > 0}
      {#key currentEra.slug}
        <div class="phases" style="grid-template-columns: repeat({Math.min(currentEra.phases.length, 5)}, 1fr)">
          {#each currentEra.phases as phase}
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
      {/key}
    {:else}
      <div class="era-placeholder">
        <p class="era-placeholder-text">This era has not yet begun.</p>
      </div>
    {/if}

    {#if currentEra.description}
      <aside class="timeline-note">
        <p>{currentEra.description}</p>
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

  /* ── Era navigation ── */
  .era-nav {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 8px;
  }

  .era-title-group {
    flex: 1;
    min-width: 0;
    text-align: center;
  }

  .timeline-heading {
    font-family: var(--display);
    font-size: clamp(22px, 3.5vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 4px;
  }

  .timeline-intro {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    line-height: 1.6;
  }

  .era-nav-btn {
    font-family: var(--display);
    font-size: 18px;
    font-weight: 500;
    color: var(--faint);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    flex-shrink: 0;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .era-nav-btn:hover:not(:disabled) {
    color: var(--gold);
    border-color: color-mix(in srgb, var(--gold) 30%, transparent);
  }

  .era-nav-btn:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  .era-nav-btn:disabled {
    opacity: 0.2;
    cursor: default;
  }

  /* ── Era pips ── */
  .era-pips {
    display: flex;
    justify-content: center;
    gap: 8px;
    margin-bottom: 8px;
  }

  .era-pip {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--border);
    border: none;
    padding: 0;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.15s ease;
  }

  .era-pip:hover {
    background: var(--faint);
  }

  .era-pip-active {
    background: var(--gold);
    transform: scale(1.25);
  }

  .era-pip-complete {
    background: var(--green-dim);
  }

  .era-pip-complete.era-pip-active {
    background: var(--green-light);
  }

  /* ── Phase cards ── */
  .phases {
    display: grid;
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

  /* ── Empty era placeholder ── */
  .era-placeholder {
    border: 1px dashed var(--border);
    border-radius: 8px;
    padding: 48px 24px;
    text-align: center;
  }

  .era-placeholder-text {
    font-family: var(--body);
    font-size: 17px;
    font-style: italic;
    color: var(--faint);
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

  /* ── Responsive ── */
  @media (max-width: 1100px) {
    .phases {
      grid-template-columns: repeat(3, 1fr) !important;
    }
  }

  @media (max-width: 980px) {
    .phases {
      grid-template-columns: repeat(2, 1fr) !important;
    }
  }

  @media (max-width: 640px) {
    .phases {
      grid-template-columns: 1fr !important;
    }

    .era-nav-btn {
      width: 32px;
      height: 32px;
      font-size: 16px;
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .progress-fill {
      transition: none;
    }

    .era-pip {
      transition: none;
    }

    .era-nav-btn {
      transition: none;
    }
  }
</style>
