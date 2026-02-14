<script lang="ts">
  import { onMount, tick } from 'svelte';

  interface Review {
    model: string;
    company: string;
    date: string;
    rating: number;
    initial: string;
    colour: string;
    icon: string;
    quote: string;
  }

  interface Props {
    slug: string;
    label?: string;
    heading?: string;
    subtitle?: string;
  }

  let {
    slug,
    label = 'Peer Review',
    heading = 'What the Machines Think',
    subtitle = 'AI systems reviewed this document. Here is what they had to say.',
  }: Props = $props();

  let reviews: Review[] = $state([]);
  let loaded = $state(false);

  onMount(async () => {
    try {
      const res = await fetch(`/document-reviews/${slug}.json`);
      if (res.ok) {
        reviews = await res.json();
      }
    } catch {
      // Fetch failed — reviews stays empty, component renders nothing
    }
    loaded = true;

    // If the URL hash targets #reviews, scroll to it after render
    if (reviews.length > 0 && window.location.hash === '#reviews') {
      await tick();
      document.getElementById('reviews')?.scrollIntoView({ behavior: 'smooth' });
    }
  });

  function stars(rating: number): { filled: number; empty: number } {
    return { filled: rating, empty: 5 - rating };
  }
</script>

{#if loaded && reviews.length > 0}
  <section class="doc-reviews" id="reviews">
    <div class="doc-reviews-rule"></div>
    <header class="doc-reviews-header">
      <span class="doc-reviews-label">{label}</span>
      <h2 class="doc-reviews-title">{heading}</h2>
      <p class="doc-reviews-subtitle">{subtitle}</p>
    </header>

    <div class="doc-reviews-grid">
      {#each reviews as review}
        <article class="review-card" style="--accent: {review.colour}">
          <blockquote class="review-quote">
            {review.quote}
          </blockquote>
          <div class="review-footer">
            <div class="review-attribution">
              <img
                class="review-avatar"
                src={review.icon}
                alt={review.model}
                width="32"
                height="32"
              />
              <div class="review-meta">
                <span class="review-model">{review.model}</span>
                <span class="review-company">{review.company} · {review.date}</span>
              </div>
            </div>
            <div class="review-stars">
              {#each { length: stars(review.rating).filled } as _}
                <span class="star filled">★</span>
              {/each}
              {#each { length: stars(review.rating).empty } as _}
                <span class="star empty">★</span>
              {/each}
            </div>
          </div>
        </article>
      {/each}
    </div>
  </section>
{/if}

<style>
  .doc-reviews {
    margin-top: 48px;
  }

  .doc-reviews-rule {
    width: 48px;
    height: 3px;
    background: var(--gold-dim);
    border-radius: 2px;
    margin-bottom: 24px;
    opacity: 0.6;
  }

  .doc-reviews-header {
    margin-bottom: 24px;
  }

  .doc-reviews-label {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--gold-dim);
    display: block;
    margin-bottom: 6px;
  }

  .doc-reviews-title {
    font-family: var(--display);
    font-size: clamp(20px, 3vw, 24px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 6px;
  }

  .doc-reviews-subtitle {
    font-family: var(--body);
    font-size: 16px;
    color: var(--faint);
    line-height: 1.6;
  }

  .doc-reviews-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 10px;
  }

  .review-card {
    background: var(--raise);
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent);
    border-radius: 8px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: var(--card-shadow);
    transition: background 0.2s ease;
  }

  .review-card:hover {
    background: var(--surface);
  }

  .review-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: auto;
  }

  .review-stars {
    display: flex;
    gap: 2px;
  }

  .star {
    font-size: 14px;
    line-height: 1;
  }

  .star.filled {
    color: var(--gold);
  }

  .star.empty {
    color: var(--border-light);
  }

  .review-quote {
    font-family: var(--body);
    font-size: 16px;
    font-style: italic;
    line-height: 1.6;
    color: var(--dim);
    border: none;
    padding: 0;
    margin: 0;
  }

  .review-attribution {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .review-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    object-fit: cover;
    flex-shrink: 0;
  }

  .review-meta {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .review-model {
    font-family: var(--display);
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
  }

  .review-company {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--faint);
  }
</style>
