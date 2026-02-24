import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';

const SITE = 'https://urd.dev';

/** Categories to include, in display order. */
const DOC_SECTIONS: Record<string, { heading: string; categories: string[] }> = {
  specs: {
    heading: 'Specifications',
    categories: ['contract', 'authoring'],
  },
  architecture: {
    heading: 'Architecture',
    categories: ['architecture'],
  },
  runtime: {
    heading: 'Runtime & Reference Cards',
    categories: ['runtime'],
  },
  research: {
    heading: 'Research & Strategy',
    categories: ['research', 'strategy'],
  },
};

export const GET: APIRoute = async () => {
  const [docs, articles] = await Promise.all([
    getCollection('designDocs'),
    getCollection('articles'),
  ]);

  const lines: string[] = [];

  // ── H1 + blockquote ──
  lines.push('# Urd');
  lines.push('');
  lines.push(
    '> Urd is a declarative schema system for interactive worlds. Writers author in Schema Markdown (.urd.md), a compiler produces .urd.json and a queryable semantic graph (the FactSet), and runtimes execute worlds anywhere. The compiler includes an LSP server and an MCP server for AI agents.',
  );
  lines.push('');

  // ── Document sections ──
  for (const section of Object.values(DOC_SECTIONS)) {
    const matching = docs
      .filter((d) => section.categories.includes(d.data.category))
      .sort((a, b) => a.data.order - b.data.order);

    if (matching.length === 0) continue;

    lines.push(`## ${section.heading}`);
    lines.push('');
    for (const doc of matching) {
      lines.push(`- [${doc.data.title}](${SITE}/documents/${doc.data.slug}): ${doc.data.description}`);
    }
    lines.push('');
  }

  // ── Key articles ──
  const recentArticles = articles
    .sort((a, b) => b.data.date.localeCompare(a.data.date))
    .slice(0, 8);

  lines.push('## Development Journal');
  lines.push('');
  for (const article of recentArticles) {
    lines.push(`- [${article.data.title}](${SITE}/articles/${article.data.slug}): ${article.data.description}`);
  }
  lines.push('');

  // ── Optional section ──
  const briefs = docs
    .filter((d) => d.data.category === 'brief')
    .sort((a, b) => a.data.order - b.data.order);

  if (briefs.length > 0) {
    lines.push('## Optional');
    lines.push('');
    for (const brief of briefs) {
      lines.push(`- [${brief.data.title}](${SITE}/documents/${brief.data.slug}): ${brief.data.description}`);
    }
    lines.push('');
  }

  return new Response(lines.join('\n'), {
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
  });
};
