import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';

export const GET: APIRoute = async () => {
  const entries = await getCollection('reviews');

  const reviews = entries.map((entry) => ({
    model: entry.data.model,
    company: entry.data.company,
    date: entry.data.date,
    rating: entry.data.rating,
    initial: entry.data.initial,
    colour: entry.data.colour,
    quote: (entry.body ?? '').trim(),
  }));

  return new Response(JSON.stringify(reviews, null, 2), {
    headers: { 'Content-Type': 'application/json' },
  });
};
