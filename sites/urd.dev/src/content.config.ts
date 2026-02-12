import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const reviews = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/reviews' }),
  schema: z.object({
    model: z.string(),
    company: z.string(),
    date: z.string(),
    rating: z.number().min(1).max(5),
    initial: z.string().length(1),
    colour: z.string(),
  }),
});

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

export const collections = { designDocs, reviews };
