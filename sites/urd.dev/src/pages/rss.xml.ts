import rss from '@astrojs/rss';
import type { APIContext } from 'astro';
import { getCollection } from 'astro:content';

export async function GET(context: APIContext) {
  const entries = await getCollection('updates');

  const updates = entries.sort(
    (a, b) => b.data.date.localeCompare(a.data.date) || b.id.localeCompare(a.id),
  );

  return rss({
    title: 'Urd — Project Updates',
    description:
      'Development journal for Urd, a declarative schema system for interactive worlds.',
    site: context.site!,
    items: updates.map((entry) => ({
      title: entry.data.title,
      pubDate: new Date(entry.data.date),
      description: (entry.body ?? '').trim(),
      link: entry.data.link ?? '/',
    })),
  });
}
