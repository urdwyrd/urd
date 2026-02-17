<script lang="ts">
  import { onMount } from 'svelte';

  interface Update {
    title: string;
    date: string;
    description: string;
    link: string | null;
  }

  interface Props {
    label?: string;
    title?: string;
    subtitle?: string;
    pageSize?: number;
  }

  let {
    label = 'Project Log',
    title = 'Updates',
    subtitle = "What's happened, as it happens.",
    pageSize = 5,
  }: Props = $props();

  let updates: Update[] = $state([]);
  let loaded = $state(false);
  let page = $state(0);

  let totalPages = $derived(Math.max(1, Math.ceil(updates.length / pageSize)));
  let visibleUpdates = $derived(updates.slice(page * pageSize, (page + 1) * pageSize));

  const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  function formatDate(iso: string): string {
    const parts = iso.split('-');
    const day = parseInt(parts[2], 10);
    const month = MONTHS[parseInt(parts[1], 10) - 1];
    return `${day} ${month}`;
  }

  function prev() {
    if (page > 0) page--;
  }

  function next() {
    if (page < totalPages - 1) page++;
  }

  onMount(async () => {
    const res = await fetch('/updates.json');
    if (res.ok) {
      updates = await res.json();
    }
    loaded = true;
  });
</script>

<section class="project-log">
  <header class="log-header">
    <span class="log-label">{label}</span>
    <h2 class="log-title">{title}</h2>
    <p class="log-subtitle">{subtitle}</p>
  </header>

  {#if loaded}
    {#key page}
    <div class="log-entries">
      {#each visibleUpdates as update, i}
        <article class="log-entry log-entry-appear" style="animation-delay: {i * 60}ms">
          <span class="log-date">{formatDate(update.date)}</span>
          <div class="log-content">
            {#if update.link}
              <h3 class="log-entry-title">
                <a href={update.link} class="log-entry-link">{update.title}</a>
                <span class="log-entry-pill">Article</span>
              </h3>
            {:else}
              <h3 class="log-entry-title">{update.title}</h3>
            {/if}
            <p class="log-entry-description">{update.description}</p>
          </div>
        </article>
      {/each}
    </div>
    {/key}

    {#if totalPages > 1}
      <nav class="log-pagination" aria-label="Update pages">
        <button
          class="log-page-btn"
          onclick={prev}
          disabled={page === 0}
          aria-label="Newer updates"
        >← Newer</button>

        <span class="log-page-indicator">
          {page + 1} of {totalPages}
        </span>

        <button
          class="log-page-btn"
          onclick={next}
          disabled={page === totalPages - 1}
          aria-label="Older updates"
        >Older →</button>
      </nav>
    {/if}
  {/if}
</section>

<style>
  .project-log {
    width: 100%;
  }

  .log-header {
    margin-bottom: 24px;
  }

  .log-label {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--gold-dim);
    display: block;
    margin-bottom: 6px;
  }

  .log-title {
    font-family: var(--display);
    font-size: clamp(22px, 3.5vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 6px;
  }

  .log-subtitle {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    line-height: 1.6;
  }

  .log-entries {
    display: flex;
    flex-direction: column;
  }

  .log-entry {
    display: flex;
    gap: 24px;
    padding: 20px 0;
    border-top: 1px solid var(--border);
  }

  .log-date {
    font-family: var(--mono);
    font-size: 14px;
    color: var(--faint);
    width: 80px;
    flex-shrink: 0;
    text-align: right;
    padding-top: 2px;
  }

  .log-content {
    flex: 1;
    min-width: 0;
  }

  .log-entry-title {
    font-family: var(--display);
    font-size: 17px;
    font-weight: 600;
    color: var(--text);
    line-height: 1.3;
    margin-bottom: 6px;
  }

  .log-entry-link {
    color: var(--text);
    text-decoration: none;
    transition: color 0.15s ease;
  }

  .log-entry-link:hover {
    color: var(--gold);
  }

  .log-entry-pill {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.2px;
    color: var(--gold-dim);
    background: color-mix(in srgb, var(--gold-dim) 10%, transparent);
    border-radius: 20px;
    padding: 2px 10px;
    margin-left: 8px;
    vertical-align: middle;
    line-height: 1.6;
  }

  .log-entry-description {
    font-family: var(--body);
    font-size: 16px;
    color: var(--dim);
    line-height: 1.6;
    margin-bottom: 0;
  }

  /* ── Page transition ── */
  @keyframes entryFadeUp {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .log-entry-appear {
    animation: entryFadeUp 0.3s ease both;
  }

  @media (prefers-reduced-motion: reduce) {
    .log-entry-appear {
      animation: none;
    }
  }

  /* ── Pagination ── */
  .log-pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 20px;
    margin-top: 8px;
    padding-top: 20px;
    border-top: 1px solid var(--border);
  }

  .log-page-btn {
    font-family: var(--display);
    font-size: 13px;
    font-weight: 500;
    color: var(--faint);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 16px;
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .log-page-btn:hover:not(:disabled) {
    color: var(--gold);
    border-color: color-mix(in srgb, var(--gold) 30%, transparent);
  }

  .log-page-btn:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
  }

  .log-page-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .log-page-indicator {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
    letter-spacing: 0.04em;
    min-width: 60px;
    text-align: center;
  }

  @media (max-width: 640px) {
    .log-entry {
      flex-direction: column;
      gap: 4px;
    }

    .log-date {
      width: auto;
      text-align: left;
    }
  }
</style>
