<script lang="ts">
  import { onMount } from 'svelte';

  interface Review {
    model: string;
    company: string;
    date: string;
    rating: number;
    initial: string;
    colour: string;
    quote: string;
  }

  interface Props {
    label?: string;
    title?: string;
    subtitle?: string;
  }

  let {
    label = 'Peer Review',
    title = 'What the Machines Think',
    subtitle = 'We asked leading AI systems to review the specification. They had opinions.',
  }: Props = $props();

  let reviews: Review[] = $state([]);
  let loaded = $state(false);

  onMount(async () => {
    const res = await fetch('/reviews.json');
    if (res.ok) {
      reviews = await res.json();
    }
    loaded = true;
  });

  function stars(rating: number): { filled: number; empty: number } {
    return { filled: rating, empty: 5 - rating };
  }
</script>

<section class="peer-review">
  <header class="peer-review-header">
    <span class="peer-review-label">{label}</span>
    <h2 class="peer-review-title">{title}</h2>
    <p class="peer-review-subtitle">{subtitle}</p>
  </header>

  {#if loaded}
    <div class="peer-review-grid">
      {#each reviews as review}
        <article class="review-card" style="--accent: {review.colour}">
          <div class="review-stars">
            {#each { length: stars(review.rating).filled } as _}
              <span class="star filled">★</span>
            {/each}
            {#each { length: stars(review.rating).empty } as _}
              <span class="star empty">★</span>
            {/each}
          </div>
          <blockquote class="review-quote">
            {review.quote}
          </blockquote>
          <div class="review-attribution">
            <div class="review-avatar" style="background-color: {review.colour}">
              {review.initial}
            </div>
            <div class="review-meta">
              <span class="review-model">{review.model}</span>
              <span class="review-company">{review.company} · {review.date}</span>
            </div>
          </div>
        </article>
      {/each}
    </div>
    <p class="peer-review-cta">Don't take their word for it — feed the specs to your favourite model and see what it thinks.</p>
  {/if}
</section>

<style>
  .peer-review {
    width: 100%;
  }

  .peer-review-header {
    margin-bottom: 24px;
  }

  .peer-review-label {
    font-family: var(--display);
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 1.5px;
    color: var(--gold-dim);
    display: block;
    margin-bottom: 6px;
  }

  .peer-review-title {
    font-family: var(--display);
    font-size: clamp(22px, 3.5vw, 28px);
    font-weight: 600;
    color: var(--text);
    letter-spacing: -0.01em;
    line-height: 1.2;
    margin-bottom: 6px;
  }

  .peer-review-subtitle {
    font-family: var(--body);
    font-size: 17px;
    color: var(--faint);
    line-height: 1.6;
  }

  .peer-review-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }

  @media (max-width: 640px) {
    .peer-review-grid {
      grid-template-columns: 1fr;
    }
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
    transition: background 0.2s ease;
  }

  .review-card:hover {
    background: var(--surface);
  }

  .review-stars {
    display: flex;
    gap: 2px;
    align-self: flex-end;
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
    margin-top: auto;
  }

  .review-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-family: var(--display);
    font-size: 14px;
    font-weight: 600;
    color: var(--bg);
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

  .peer-review-cta {
    font-family: var(--body);
    font-size: 15px;
    color: var(--faint);
    text-align: center;
    margin-top: 16px;
  }
</style>
