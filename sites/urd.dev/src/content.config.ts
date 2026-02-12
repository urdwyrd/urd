import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const designDocs = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../docs' }),
  schema: z.object({
    title: z.string(),
    slug: z.string(),
    description: z.string(),
    category: z.enum([
      'research', 'contract', 'authoring',
      'architecture', 'runtime', 'validation', 'strategy',
    ]),
    format: z.string(),
    date: z.string(),
    status: z.string(),
    order: z.number(),
    tags: z.array(z.string()),
    details: z.array(z.string()),
  }),
});

export const collections = { designDocs };
