import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';

export const GET: APIRoute = async () => {
  const entries = await getCollection('updates');

  const updates = entries
    .sort((a, b) => b.data.date.localeCompare(a.data.date))
    .map((entry) => ({
      title: entry.data.title,
      date: entry.data.date,
      description: (entry.body ?? '').trim(),
    }));

  return new Response(JSON.stringify(updates, null, 2), {
    headers: { 'Content-Type': 'application/json' },
  });
};
