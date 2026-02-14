import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { categoryColours } from '../lib/colours';

function stripMarkdown(text: string): string {
  return text
    .replace(/^#{1,6}\s+/gm, '')       // headings
    .replace(/\*{1,3}([^*]+)\*{1,3}/g, '$1') // bold/italic
    .replace(/`{1,3}[^`]*`{1,3}/g, '') // inline code / code blocks
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1') // links
    .replace(/^\s*[-*+]\s+/gm, '')     // list markers
    .replace(/^\s*>\s+/gm, '')         // blockquotes
    .replace(/\|[^|]*\|/g, '')         // table rows
    .replace(/---+/g, '')              // horizontal rules
    .replace(/\n{2,}/g, ' ')           // collapse blank lines
    .replace(/\s+/g, ' ')             // normalise whitespace
    .trim();
}

export const GET: APIRoute = async () => {
  const entries = await getCollection('designDocs');
  const allReviews = await getCollection('documentReviews');

  // Build a map of slug → average rating
  const ratingsBySlug = new Map<string, number[]>();
  for (const review of allReviews) {
    const slug = review.id.split('/')[0];
    if (!ratingsBySlug.has(slug)) ratingsBySlug.set(slug, []);
    ratingsBySlug.get(slug)!.push(review.data.rating);
  }

  const documents = entries
    .sort((a, b) => b.data.date.localeCompare(a.data.date))
    .map((entry) => {
      const body = entry.body ?? '';
      const wordCount = body.split(/\s+/).filter(Boolean).length;
      const readingTime = Math.ceil(wordCount / 250);
      const plain = stripMarkdown(body);
      const excerpt = plain.length > 300 ? plain.slice(0, 300) + '…' : plain;

      return {
        title: entry.data.title,
        slug: entry.data.slug,
        description: entry.data.description,
        category: entry.data.category,
        format: entry.data.format,
        date: entry.data.date,
        status: entry.data.status,
        order: entry.data.order,
        tags: entry.data.tags,
        details: entry.data.details,
        wordCount,
        readingTime,
        excerpt,
        colour: categoryColours[entry.data.category] ?? '#888888',
        rating: (() => {
          const ratings = ratingsBySlug.get(entry.data.slug);
          if (!ratings || ratings.length === 0) return null;
          return Math.round((ratings.reduce((a, b) => a + b, 0) / ratings.length) * 10) / 10;
        })(),
        reviewCount: ratingsBySlug.get(entry.data.slug)?.length ?? 0,
        url: `/documents/${entry.data.slug}`,
        githubUrl: `https://github.com/urdwyrd/urd/blob/main/docs/${entry.id}.md`,
        downloadUrl: `https://raw.githubusercontent.com/urdwyrd/urd/main/docs/${entry.id}.md`,
      };
    });

  return new Response(JSON.stringify(documents, null, 2), {
    headers: { 'Content-Type': 'application/json' },
  });
};
