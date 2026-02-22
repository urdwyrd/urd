import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';

/** Count completed vs total requirements from the gate document body. */
function computeGateProgress(body: string): number {
  let total = 0;
  let done = 0;
  const seen = new Set<string>();

  // C, S, F, E sections have a Status column with ✓ markers.
  // Deduplicate by ID — the audit record table reuses the same IDs.
  for (const match of body.matchAll(/^\| ([CSFE]\d+) \|.+\|(.+)\|$/gm)) {
    const id = match[1];
    if (seen.has(id)) continue;
    seen.add(id);
    total++;
    if (match[2].trim().startsWith('✓')) done++;
  }

  return total > 0 ? Math.round((done / total) * 100) : 0;
}

export const GET: APIRoute = async () => {
  const [entries, docs] = await Promise.all([
    getCollection('timeline'),
    getCollection('designDocs'),
  ]);

  // Compute gate progress from the v1-completion-gate document
  const gateDoc = docs.find((d) => d.data.slug === 'v1-completion-gate');
  const gateProgress = gateDoc ? computeGateProgress(gateDoc.body ?? '') : null;

  const phases = entries
    .sort((a, b) => a.data.order - b.data.order)
    .map((entry) => ({
      title: entry.data.title,
      status: entry.data.status,
      subtitle: entry.data.subtitle,
      order: entry.data.order,
      description: (entry.body ?? '').trim(),
      link: entry.data.link ?? null,
      linkLabel: entry.data.linkLabel ?? null,
      progress: entry.data.title === 'Validation' ? gateProgress : null,
    }));

  return new Response(JSON.stringify(phases, null, 2), {
    headers: { 'Content-Type': 'application/json' },
  });
};
