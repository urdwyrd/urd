<script lang="ts">
  import { onMount } from 'svelte';

  interface Document {
    title: string;
    slug: string;
    description: string;
    category: string;
    format: string;
    date: string;
    status: string;
    order: number;
    tags: string[];
    details: string[];
    wordCount: number;
    readingTime: number;
    excerpt: string;
    colour: string;
    url: string;
  }

  interface Props {
    label?: string;
    title?: string;
    subtitle?: string;
  }

  const {
    label = 'Artifacts',
    title = 'The work so far',
    subtitle = 'Every document produced. Click to expand.',
  }: Props = $props();

  const MONTHS: Record<string, string> = {
    '01': 'Jan', '02': 'Feb', '03': 'Mar', '04': 'Apr',
    '05': 'May', '06': 'Jun', '07': 'Jul', '08': 'Aug',
    '09': 'Sep', '10': 'Oct', '11': 'Nov', '12': 'Dec',
  };

  let documents = $state<Document[]>([]);
  let activeFilter = $state<string>('all');
  let expandedSlugs = $state<Set<string>>(new Set());
  let loaded = $state(false);
  let reducedMotion = $state(false);

  const categories = $derived(
    [...new Set(documents.map((d) => d.category))].sort()
  );

  const filtered = $derived(
    activeFilter === 'all'
      ? documents
      : documents.filter((d) => d.category === activeFilter)
  );

  function formatDate(date: string): string {
    const [year, month] = date.split('-');
    return `${MONTHS[month] ?? month} ${year}`;
  }

  function formatWordCount(count: number): string {
    return count.toLocaleString('en-GB') + ' words';
  }

  function titleCase(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }

  function toggleExpanded(slug: string): void {
    const next = new Set(expandedSlugs);
    if (next.has(slug)) {
      next.delete(slug);
    } else {
      next.add(slug);
    }
    expandedSlugs = next;
  }

  function setFilter(category: string): void {
    activeFilter = category;
  }

  onMount(async () => {
    reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    try {
      const res = await fetch('/documents.json');
      documents = await res.json();
    } catch {
      documents = [];
    }
    loaded = true;
  });
</script>

<section class="doc-explorer">
  <header class="doc-explorer-header">
    <div class="doc-explorer-header-text">
      <p class="doc-explorer-label">{label}</p>
      <h2 class="doc-explorer-title">{title}</h2>
      <p class="doc-explorer-subtitle">{subtitle}</p>
    </div>

    {#if loaded && categories.length > 0}
      <nav class="doc-explorer-filters" aria-label="Filter documents by category">
        <button
          class="doc-filter-btn"
          class:active={activeFilter === 'all'}
          onclick={() => setFilter('all')}
        >
          All
        </button>
        {#each categories as cat}
          <button
            class="doc-filter-btn"
            class:active={activeFilter === cat}
            style:--filter-colour="var(--doc-{cat})"
            onclick={() => setFilter(cat)}
          >
            {titleCase(cat)}
          </button>
        {/each}
      </nav>
    {/if}
  </header>

  {#if loaded}
    {#if filtered.length === 0}
      <p class="doc-explorer-empty">No documents in this category yet.</p>
    {:else}
      <div class="doc-card-list">
        {#each filtered as doc (doc.slug)}
          {@const isExpanded = expandedSlugs.has(doc.slug)}
          <article
            class="doc-card"
            style:--card-colour="var(--doc-{doc.category})"
            style:--card-dim="var(--doc-{doc.category}-dim)"
          >
            <button
              class="doc-card-toggle"
              onclick={() => toggleExpanded(doc.slug)}
              aria-expanded={isExpanded}
              aria-label="{isExpanded ? 'Collapse' : 'Expand'} {doc.title}"
            >
              <div class="doc-card-meta">
                <span class="doc-card-pill">{doc.format}</span>
                <span class="doc-card-date">{formatDate(doc.date)}</span>
                <span class="doc-card-words">{formatWordCount(doc.wordCount)}</span>
                <span class="doc-card-chevron" class:expanded={isExpanded}>▸</span>
              </div>

              <h3 class="doc-card-title">{doc.title}</h3>
              <p class="doc-card-description">{doc.description}</p>
            </button>

            {#if isExpanded}
              <div class="doc-card-expanded" class:no-motion={reducedMotion}>
                <p class="doc-card-section-label">Key content</p>
                <ul class="doc-card-details">
                  {#each doc.details as detail}
                    <li>{detail}</li>
                  {/each}
                </ul>

                {#if doc.tags.length > 0}
                  <p class="doc-card-tags">
                    {doc.tags.join(' · ')}
                  </p>
                {/if}

                <a class="doc-card-link" href={doc.url}>
                  Read full document →
                </a>
              </div>
            {/if}
          </article>
        {/each}
      </div>
    {/if}
  {/if}
</section>

<style>
  /* ── Container ── */
  .doc-explorer {
    max-width: 860px;
    margin: 0 auto;
    padding: 0 32px;
  }

  /* ── Header ── */
  .doc-explorer-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 24px;
    margin-bottom: 28px;
    flex-wrap: wrap;
  }

  .doc-explorer-label {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--gold-dim);
    margin-bottom: 6px;
  }

  .doc-explorer-title {
    font-family: var(--display);
    font-size: clamp(22px, 3.5vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 6px;
  }

  .doc-explorer-subtitle {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    line-height: 1.6;
  }

  /* ── Filters ── */
  .doc-explorer-filters {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
    align-items: center;
  }

  .doc-filter-btn {
    font-family: var(--display);
    font-size: 11px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--faint);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 5px 12px;
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease, background 0.15s ease;
    white-space: nowrap;
  }

  .doc-filter-btn:hover {
    color: var(--dim);
    border-color: var(--border-light);
  }

  .doc-filter-btn.active {
    color: var(--filter-colour, var(--gold));
    border-color: var(--filter-colour, var(--gold));
    background: rgba(255, 255, 255, 0.03);
  }

  /* ── Card List ── */
  .doc-card-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  /* ── Card ── */
  .doc-card {
    background: var(--raise);
    border: 1px solid var(--border);
    border-left: 2px solid var(--card-colour);
    border-radius: 8px;
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .doc-card:hover {
    background: var(--surface);
    border-color: var(--border-light);
    border-left-color: var(--card-colour);
  }

  /* ── Card Toggle (clickable area) ── */
  .doc-card-toggle {
    display: block;
    width: 100%;
    padding: 18px 20px;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    color: inherit;
    font: inherit;
  }

  .doc-card-toggle:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: -2px;
    border-radius: 7px;
  }

  /* ── Card Meta Row ── */
  .doc-card-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
    flex-wrap: wrap;
  }

  .doc-card-pill {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--card-colour);
    background: var(--card-dim);
    border: 1px solid color-mix(in srgb, var(--card-colour) 20%, transparent);
    border-radius: 4px;
    padding: 2px 8px;
    line-height: 1.6;
  }

  .doc-card-date,
  .doc-card-words {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
    letter-spacing: 0.06em;
  }

  .doc-card-chevron {
    margin-left: auto;
    font-size: 14px;
    color: var(--faint);
    transition: transform 0.15s ease;
    display: inline-block;
  }

  .doc-card-chevron.expanded {
    transform: rotate(90deg);
  }

  /* ── Card Content ── */
  .doc-card-title {
    font-family: var(--display);
    font-size: clamp(16px, 2.5vw, 18px);
    font-weight: 600;
    color: var(--text);
    line-height: 1.3;
    margin-bottom: 6px;
  }

  .doc-card-description {
    font-family: var(--body);
    font-size: 17px;
    font-weight: 400;
    color: var(--dim);
    line-height: 1.6;
  }

  /* ── Expanded Section ── */
  .doc-card-expanded {
    padding: 0 20px 20px;
    border-top: 1px solid var(--border);
    margin: 0 20px;
    padding-top: 16px;
    animation: fadeIn 0.25s ease-out;
  }

  .doc-card-expanded.no-motion {
    animation: none;
  }

  .doc-card-section-label {
    font-family: var(--display);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--faint);
    margin-bottom: 10px;
  }

  .doc-card-details {
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 14px;
  }

  .doc-card-details li {
    font-family: var(--body);
    font-size: 16px;
    color: var(--dim);
    line-height: 1.6;
    padding-left: 18px;
    position: relative;
  }

  .doc-card-details li::before {
    content: '◆';
    position: absolute;
    left: 0;
    font-size: 7px;
    top: 0.55em;
    color: var(--card-colour);
  }

  .doc-card-tags {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--card-colour);
    opacity: 0.6;
    letter-spacing: 0.04em;
    margin-bottom: 14px;
  }

  .doc-card-link {
    font-family: var(--display);
    font-size: 13px;
    font-weight: 500;
    color: var(--card-colour);
    text-decoration: none;
    display: inline-block;
    padding: 6px 14px;
    border: 1px solid color-mix(in srgb, var(--card-colour) 30%, transparent);
    border-radius: 4px;
    transition: border-color 0.15s ease, background 0.15s ease;
  }

  .doc-card-link:hover {
    border-color: var(--card-colour);
    background: rgba(255, 255, 255, 0.02);
  }

  .doc-card-link:focus-visible {
    outline: 2px solid var(--card-colour);
    outline-offset: 2px;
  }

  /* ── Empty State ── */
  .doc-explorer-empty {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    text-align: center;
    padding: 48px 0;
  }

  /* ── Responsive ── */
  @media (max-width: 980px) {
    .doc-explorer-header {
      flex-direction: column;
      align-items: flex-start;
    }
  }

  @media (max-width: 520px) {
    .doc-explorer {
      padding: 0 18px;
    }

    .doc-card-toggle {
      padding: 16px;
    }

    .doc-card-expanded {
      margin: 0 16px;
      padding: 14px 16px 16px;
    }

    .doc-card-description {
      font-size: 16px;
    }
  }
</style>
