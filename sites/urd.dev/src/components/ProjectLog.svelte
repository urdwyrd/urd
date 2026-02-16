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
  }

  let {
    label = 'Project Log',
    title = 'Updates',
    subtitle = "What's happened, as it happens.",
  }: Props = $props();

  let updates: Update[] = $state([]);
  let loaded = $state(false);

  const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  function formatDate(iso: string): string {
    const parts = iso.split('-');
    const day = parseInt(parts[2], 10);
    const month = MONTHS[parseInt(parts[1], 10) - 1];
    return `${day} ${month}`;
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
    <div class="log-entries">
      {#each updates as update}
        <article class="log-entry">
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
