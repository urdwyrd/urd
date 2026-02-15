<script lang="ts">
  import { onMount, tick } from 'svelte';

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
    rating: number | null;
    reviewCount: number;
    url: string;
    githubUrl: string;
    downloadUrl: string;
  }

  interface Props {
    label?: string;
    title?: string;
    subtitle?: string;
  }

  const {
    label = 'Artifacts',
    title = 'The work so far',
    subtitle = 'Every document produced.',
  }: Props = $props();

  const MONTHS: Record<string, string> = {
    '01': 'Jan', '02': 'Feb', '03': 'Mar', '04': 'Apr',
    '05': 'May', '06': 'Jun', '07': 'Jul', '08': 'Aug',
    '09': 'Sep', '10': 'Oct', '11': 'Nov', '12': 'Dec',
  };

  let documents = $state<Document[]>([]);
  let activeFilter = $state<string>('all');
  let activeFormat = $state<string>('all');
  const VALID_VIEWS = new Set(['list', 'grid', 'compact']);
  const storedView = typeof localStorage !== 'undefined' ? localStorage.getItem('urd-view') : null;
  let viewMode = $state<'list' | 'grid' | 'compact'>(
    storedView && VALID_VIEWS.has(storedView) ? storedView as 'list' | 'grid' | 'compact' : 'grid'
  );
  let expandedSlugs = $state<Set<string>>(new Set());
  let loaded = $state(false);
  let reducedMotion = $state(false);

  const categories = $derived(
    [...new Set(documents.map((d) => d.category))].sort()
  );

  const categoryFiltered = $derived(
    activeFilter === 'all'
      ? documents
      : documents.filter((d) => d.category === activeFilter)
  );

  const availableFormats = $derived(
    [...new Set(categoryFiltered.map((d) => d.format))].sort()
  );

  const filtered = $derived(
    activeFormat === 'all'
      ? categoryFiltered
      : categoryFiltered.filter((d) => d.format === activeFormat)
  );

  function formatDate(date: string): string {
    const parts = date.split('-');
    if (parts.length === 3) {
      const [year, month, day] = parts;
      return `${parseInt(day, 10)} ${MONTHS[month] ?? month} ${year}`;
    }
    const [year, month] = parts;
    return `${MONTHS[month] ?? month} ${year}`;
  }

  function formatReadingTime(minutes: number, words: number): string {
    const time = minutes < 1 ? '< 1' : String(minutes);
    return `${time} min read (${words.toLocaleString('en-GB')} words)`;
  }

  function formatReadingTimeShort(minutes: number): string {
    return minutes < 1 ? '< 1 min' : `${minutes} min`;
  }

  function titleCase(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }

  function starsFromRating(rating: number): { filled: number; empty: number } {
    const rounded = Math.round(rating);
    return { filled: rounded, empty: 5 - rounded };
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
    activeFormat = 'all';
    if (fmtScrollEl) fmtScrollEl.scrollLeft = 0;
  }

  function setFormatFilter(format: string): void {
    activeFormat = format;
  }

  // Scroll arrow state
  let catScrollEl = $state<HTMLElement | null>(null);
  let fmtScrollEl = $state<HTMLElement | null>(null);
  let catCanScrollLeft = $state(false);
  let catCanScrollRight = $state(false);
  let fmtCanScrollLeft = $state(false);
  let fmtCanScrollRight = $state(false);

  function updateScrollState(el: HTMLElement | null, setLeft: (v: boolean) => void, setRight: (v: boolean) => void): void {
    if (!el) return;
    setLeft(el.scrollLeft > 2);
    setRight(el.scrollLeft + el.clientWidth < el.scrollWidth - 2);
  }

  function scrollBy(el: HTMLElement | null, delta: number): void {
    el?.scrollBy({ left: delta, behavior: 'smooth' });
  }

  $effect(() => {
    if (catScrollEl) {
      const update = () => updateScrollState(catScrollEl, (v) => catCanScrollLeft = v, (v) => catCanScrollRight = v);
      catScrollEl.addEventListener('scroll', update, { passive: true });
      const ro = new ResizeObserver(update);
      ro.observe(catScrollEl);
      update();
      return () => { catScrollEl?.removeEventListener('scroll', update); ro.disconnect(); };
    }
  });

  $effect(() => {
    if (fmtScrollEl) {
      const update = () => updateScrollState(fmtScrollEl, (v) => fmtCanScrollLeft = v, (v) => fmtCanScrollRight = v);
      fmtScrollEl.addEventListener('scroll', update, { passive: true });
      const ro = new ResizeObserver(update);
      ro.observe(fmtScrollEl);
      update();
      return () => { fmtScrollEl?.removeEventListener('scroll', update); ro.disconnect(); };
    }
  });

  // Re-check scroll arrows when filter content changes (buttons added/removed)
  $effect(() => {
    activeFilter;
    availableFormats;
    categories;
    tick().then(() => {
      updateScrollState(catScrollEl, (v) => catCanScrollLeft = v, (v) => catCanScrollRight = v);
      updateScrollState(fmtScrollEl, (v) => fmtCanScrollLeft = v, (v) => fmtCanScrollRight = v);
    });
  });

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
      <div class="doc-explorer-subtitle-row">
        <p class="doc-explorer-subtitle">{subtitle}</p>
        {#if loaded}
          <div class="doc-view-toggle" role="group" aria-label="View mode">
            <button
              class="doc-view-btn"
              class:active={viewMode === 'compact'}
              onclick={() => { viewMode = 'compact'; localStorage.setItem('urd-view', 'compact'); }}
              aria-label="Compact view"
            ><span class="doc-view-icon">—</span> Compact</button>
            <button
              class="doc-view-btn"
              class:active={viewMode === 'list'}
              onclick={() => { viewMode = 'list'; localStorage.setItem('urd-view', 'list'); }}
              aria-label="List view"
            ><span class="doc-view-icon">≡</span> List</button>
            <button
              class="doc-view-btn"
              class:active={viewMode === 'grid'}
              onclick={() => { viewMode = 'grid'; localStorage.setItem('urd-view', 'grid'); }}
              aria-label="Card view"
            ><span class="doc-view-icon">⊞</span> Cards</button>
          </div>
        {/if}
      </div>
    </div>

    {#if loaded && categories.length > 0}
      <div class="scroll-row">
        {#if catCanScrollLeft}
          <button class="scroll-arrow scroll-arrow-left" aria-label="Scroll left" onclick={() => scrollBy(catScrollEl, -120)}>‹ more</button>
        {/if}
        <nav class="doc-explorer-filters" bind:this={catScrollEl} aria-label="Filter documents by category">
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
        {#if catCanScrollRight}
          <button class="scroll-arrow scroll-arrow-right" aria-label="Scroll right" onclick={() => scrollBy(catScrollEl, 120)}>more ›</button>
        {/if}
      </div>
    {/if}
  </header>

  {#if loaded}
    <div class="scroll-row">
      {#if fmtCanScrollLeft}
        <button class="scroll-arrow scroll-arrow-left" aria-label="Scroll left" onclick={() => scrollBy(fmtScrollEl, -120)}>‹ more</button>
      {/if}
      <nav
        class="doc-explorer-format-filters"
        bind:this={fmtScrollEl}
        aria-label="Filter documents by format"
        style={activeFilter !== 'all' ? `--format-accent: var(--doc-${activeFilter})` : ''}
      >
        <button
          class="doc-format-btn"
          class:active={activeFormat === 'all'}
          onclick={() => setFormatFilter('all')}
        >
          All formats
        </button>
        {#each availableFormats as fmt}
          <button
            class="doc-format-btn"
            class:active={activeFormat === fmt}
            onclick={() => setFormatFilter(fmt)}
          >
            {fmt}
          </button>
        {/each}
      </nav>
      {#if fmtCanScrollRight}
        <button class="scroll-arrow scroll-arrow-right" aria-label="Scroll right" onclick={() => scrollBy(fmtScrollEl, 120)}>more ›</button>
      {/if}
    </div>
  {/if}

  {#if loaded}
    {#if filtered.length === 0}
      <p class="doc-explorer-empty">No documents match these filters.</p>
    {:else if viewMode === 'list'}
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
                <span class="doc-card-separator" aria-hidden="true">◷</span>
                <span class="doc-card-words">{formatReadingTime(doc.readingTime, doc.wordCount)}</span>
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

                <div class="doc-card-actions-row">
                  <div class="doc-card-actions">
                    <a class="doc-card-link" href={doc.url}>
                      Read full document →
                    </a>
                    <a class="doc-card-link doc-card-link-secondary" href={doc.githubUrl} target="_blank" rel="noopener noreferrer">
                      View on GitHub <span aria-hidden="true">↗</span>
                    </a>
                    <button
                      class="doc-card-link doc-card-link-secondary"
                      onclick={(e) => {
                        const btn = e.currentTarget;
                        btn.disabled = true;
                        fetch(doc.downloadUrl)
                          .then((r) => r.blob())
                          .then((blob) => {
                            const url = URL.createObjectURL(blob);
                            const a = document.createElement('a');
                            a.href = url;
                            a.download = `${doc.slug}.md`;
                            a.click();
                            URL.revokeObjectURL(url);
                          })
                          .finally(() => { btn.disabled = false; });
                      }}
                    >
                      Download .md
                    </button>
                  </div>
                  {#if doc.rating !== null}
                    <a class="doc-card-rating" href="{doc.url}#reviews" title="{doc.rating} / 5 from {doc.reviewCount} reviews — click for details">
                      {#each { length: starsFromRating(doc.rating).filled } as _}
                        <span class="star-inline filled">★</span>
                      {/each}
                      {#each { length: starsFromRating(doc.rating).empty } as _}
                        <span class="star-inline empty">★</span>
                      {/each}
                    </a>
                  {/if}
                </div>
              </div>
            {/if}
          </article>
        {/each}
      </div>
    {:else if viewMode === 'grid'}
      <div class="doc-grid">
        {#each filtered as doc (doc.slug)}
          <article
            class="doc-grid-card"
            style:--card-colour="var(--doc-{doc.category})"
            style:--card-dim="var(--doc-{doc.category}-dim)"
          >
            <div class="doc-grid-card-header">
              <span class="doc-card-pill">{doc.format}</span>
              <span class="doc-grid-card-time">{formatReadingTimeShort(doc.readingTime)}</span>
            </div>
            <a class="doc-grid-card-title" href={doc.url}>{doc.title}</a>
            <p class="doc-grid-card-desc">{doc.description}</p>
            <div class="doc-grid-card-footer">
              <span class="doc-grid-card-date">
                {formatDate(doc.date)}
                {#if doc.rating !== null}
                  <a class="doc-grid-rating" href="{doc.url}#reviews" title="{doc.rating} / 5 from {doc.reviewCount} reviews — click for details">
                    <span class="star-inline filled">★</span> {doc.rating}
                  </a>
                {/if}
              </span>
              <div class="doc-grid-card-actions">
                <a class="doc-grid-action" href={doc.url} title="Read document">→</a>
                <a class="doc-grid-action" href={doc.githubUrl} target="_blank" rel="noopener noreferrer" title="View on GitHub">↗</a>
                <button
                  class="doc-grid-action"
                  title="Download .md"
                  onclick={(e) => {
                    const btn = e.currentTarget;
                    btn.disabled = true;
                    fetch(doc.downloadUrl)
                      .then((r) => r.blob())
                      .then((blob) => {
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = `${doc.slug}.md`;
                        a.click();
                        URL.revokeObjectURL(url);
                      })
                      .finally(() => { btn.disabled = false; });
                  }}
                >⤓</button>
              </div>
            </div>
          </article>
        {/each}
      </div>
    {:else}
      <div class="doc-compact-list">
        {#each filtered as doc (doc.slug)}
          <a
            class="doc-compact-row"
            href={doc.url}
            style:--card-colour="var(--doc-{doc.category})"
          >
            <span class="doc-compact-indicator"></span>
            <span class="doc-compact-title">{doc.title}</span>
            <span class="doc-compact-format">{doc.format}</span>
            <span class="doc-compact-time">{formatReadingTimeShort(doc.readingTime)}</span>
            <span class="doc-compact-date">{formatDate(doc.date)}</span>
            {#if doc.rating !== null}
              <span
                class="doc-compact-rating"
                title="{doc.rating} / 5 from {doc.reviewCount} reviews — click for details"
                role="link"
                onclick={(e) => { e.preventDefault(); e.stopPropagation(); window.location.href = `${doc.url}#reviews`; }}
                onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); e.stopPropagation(); window.location.href = `${doc.url}#reviews`; } }}
                tabindex="0"
              >
                <span class="star-inline filled">★</span> {doc.rating}
              </span>
            {/if}
          </a>
        {/each}
      </div>
    {/if}
  {/if}
</section>

<style>
  /* ── Container ── */
  .doc-explorer {
    width: 100%;
  }

  /* ── Header ── */
  .doc-explorer-header {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 6px;
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

  .doc-explorer-subtitle-row {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .doc-explorer-subtitle {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    line-height: 1.6;
  }

  /* ── Scroll Row ── */
  .scroll-row {
    position: relative;
  }

  .scroll-arrow {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    display: flex;
    align-items: center;
    gap: 2px;
    font-family: var(--display);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: var(--dim);
    background: var(--raise);
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 3px 8px;
    cursor: pointer;
    z-index: 2;
    transition: color 0.15s ease, border-color 0.15s ease, background 0.15s ease;
    white-space: nowrap;
  }

  .scroll-arrow:hover {
    color: var(--gold);
    border-color: var(--gold-dim);
    background: var(--surface);
  }

  .scroll-arrow-left {
    left: 0;
  }

  .scroll-arrow-right {
    right: 0;
  }

  /* ── Category Filters ── */
  .doc-explorer-filters {
    display: flex;
    gap: 6px;
    align-items: center;
    overflow-x: auto;
    scrollbar-width: none;
    -webkit-overflow-scrolling: touch;
  }

  .doc-explorer-filters::-webkit-scrollbar {
    display: none;
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
    background: var(--card-overlay);
  }

  /* ── Format Filters ── */
  .doc-explorer-format-filters {
    display: flex;
    gap: 5px;
    align-items: center;
    margin-bottom: 8px;
    overflow-x: auto;
    scrollbar-width: none;
    -webkit-overflow-scrolling: touch;
  }

  .doc-explorer-format-filters::-webkit-scrollbar {
    display: none;
  }

  .doc-format-btn {
    font-family: var(--display);
    font-size: 10px;
    font-weight: 400;
    letter-spacing: 0.02em;
    color: var(--faint);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 3px 9px;
    cursor: pointer;
    transition: color 0.15s ease, border-color 0.15s ease;
    white-space: nowrap;
  }

  .doc-format-btn:hover {
    color: var(--dim);
    border-color: var(--border-light);
  }

  .doc-format-btn.active {
    color: var(--format-accent, var(--text));
    border-color: var(--format-accent, var(--border-light));
  }

  /* ── View Toggle ── */
  .doc-view-toggle {
    display: flex;
    gap: 3px;
    flex-shrink: 0;
    margin-left: auto;
    background: var(--raise);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 3px;
  }

  .doc-view-btn {
    display: flex;
    align-items: center;
    gap: 5px;
    font-family: var(--display);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.02em;
    line-height: 1;
    color: var(--faint);
    background: transparent;
    border: none;
    border-radius: 4px;
    padding: 5px 10px;
    cursor: pointer;
    transition: color 0.2s ease, background 0.2s ease, box-shadow 0.2s ease;
  }

  .doc-view-icon {
    font-size: 14px;
    line-height: 1;
  }

  .doc-view-btn.active {
    color: var(--gold);
    background: var(--surface);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .doc-view-btn:hover:not(.active) {
    color: var(--dim);
    background: var(--surface);
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
    box-shadow: var(--card-shadow);
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

  .doc-card-separator {
    font-size: 10px;
    color: var(--border-light);
    margin: 0 -3px;
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

  /* ── Action Links ── */
  .doc-card-actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
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
    background: var(--card-overlay);
  }

  .doc-card-link:focus-visible {
    outline: 2px solid var(--card-colour);
    outline-offset: 2px;
  }

  .doc-card-link-secondary {
    font-weight: 400;
    font-size: 12px;
    color: var(--faint);
    border-color: var(--border);
  }

  .doc-card-link-secondary:hover {
    color: var(--dim);
    border-color: var(--border-light);
    background: var(--card-overlay);
  }

  /* ── Grid View ── */
  .doc-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  .doc-grid-card {
    display: flex;
    flex-direction: column;
    background: var(--raise);
    border: 1px solid var(--border);
    border-top: 2px solid var(--card-colour);
    border-radius: 8px;
    padding: 16px 18px;
    box-shadow: var(--card-shadow);
    transition: background 0.15s ease, border-color 0.15s ease;
  }

  .doc-grid-card:hover {
    background: var(--surface);
    border-color: var(--border-light);
    border-top-color: var(--card-colour);
  }

  .doc-grid-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 10px;
  }

  .doc-grid-card-time {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    letter-spacing: 0.06em;
    white-space: nowrap;
  }

  .doc-grid-card-title {
    font-family: var(--display);
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
    text-decoration: none;
    line-height: 1.3;
    margin-bottom: 8px;
  }

  .doc-grid-card-title:hover {
    color: var(--card-colour);
  }

  .doc-grid-card-title:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 2px;
    border-radius: 2px;
  }

  .doc-grid-card-desc {
    font-family: var(--body);
    font-size: 14px;
    color: var(--dim);
    line-height: 1.5;
    flex: 1;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .doc-grid-card-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 12px;
  }

  .doc-grid-card-date {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    letter-spacing: 0.06em;
  }

  .doc-grid-card-actions {
    display: flex;
    gap: 4px;
  }

  .doc-grid-action {
    font-size: 13px;
    line-height: 1;
    color: var(--faint);
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 3px;
    padding: 4px 7px;
    cursor: pointer;
    text-decoration: none;
    transition: color 0.15s ease, border-color 0.15s ease;
  }

  .doc-grid-action:hover {
    color: var(--card-colour);
    border-color: var(--border-light);
  }

  .doc-grid-action:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: 1px;
  }

  /* ── Compact View ── */
  .doc-compact-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .doc-compact-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    text-decoration: none;
    color: inherit;
    border-radius: 4px;
    transition: background 0.1s ease;
  }

  .doc-compact-row:hover {
    background: var(--raise);
  }

  .doc-compact-row:focus-visible {
    outline: 2px solid var(--gold);
    outline-offset: -2px;
  }

  .doc-compact-indicator {
    width: 3px;
    height: 16px;
    border-radius: 1px;
    background: var(--card-colour);
    flex-shrink: 0;
  }

  .doc-compact-title {
    font-family: var(--display);
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .doc-compact-format {
    font-family: var(--display);
    font-size: 10px;
    color: var(--faint);
    white-space: nowrap;
    width: 150px;
    text-align: right;
  }

  .doc-compact-time {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    letter-spacing: 0.06em;
    white-space: nowrap;
    width: 52px;
    text-align: right;
  }

  .doc-compact-date {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    letter-spacing: 0.06em;
    white-space: nowrap;
    width: 100px;
    text-align: right;
  }

  /* ── Star Ratings ── */
  .star-inline {
    font-size: 11px;
    line-height: 1;
  }

  .star-inline.filled {
    color: var(--gold);
  }

  .star-inline.empty {
    color: var(--border-light);
  }

  .doc-card-actions-row {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 12px;
  }

  .doc-card-rating {
    display: inline-flex;
    align-items: center;
    gap: 1px;
    flex-shrink: 0;
    text-decoration: none;
    transition: opacity 0.15s ease;
  }

  .doc-card-rating:hover {
    opacity: 0.8;
  }

  .doc-compact-rating {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-family: var(--mono);
    font-size: 11px;
    color: var(--faint);
    letter-spacing: 0.06em;
    white-space: nowrap;
    width: 42px;
    text-align: right;
    cursor: pointer;
    transition: color 0.15s ease;
  }

  .doc-compact-rating:hover {
    color: var(--gold);
  }

  .doc-grid-rating {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    margin-left: 8px;
    text-decoration: none;
    color: inherit;
    transition: opacity 0.15s ease;
  }

  .doc-grid-rating:hover {
    opacity: 0.8;
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
  @media (max-width: 640px) {
    .doc-grid {
      grid-template-columns: 1fr;
    }

    .doc-compact-format,
    .doc-compact-date {
      display: none;
    }
  }

  @media (max-width: 520px) {

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

    .doc-card-actions {
      flex-direction: column;
    }
  }
</style>
