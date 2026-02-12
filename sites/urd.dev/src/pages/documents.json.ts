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

  const documents = entries
    .sort((a, b) => a.data.order - b.data.order)
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
        url: `/documents/${entry.data.slug}`,
      };
    });

  return new Response(JSON.stringify(documents, null, 2), {
    headers: { 'Content-Type': 'application/json' },
  });
};
