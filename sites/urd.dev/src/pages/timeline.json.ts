import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';

export const GET: APIRoute = async () => {
  const entries = await getCollection('timeline');

  const phases = entries
    .sort((a, b) => a.data.order - b.data.order)
    .map((entry) => ({
      title: entry.data.title,
      status: entry.data.status,
      subtitle: entry.data.subtitle,
      order: entry.data.order,
      description: (entry.body ?? '').trim(),
    }));

  return new Response(JSON.stringify(phases, null, 2), {
    headers: { 'Content-Type': 'application/json' },
  });
};
