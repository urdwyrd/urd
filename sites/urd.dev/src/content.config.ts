import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

const updates = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/updates' }),
  schema: z.object({
    title: z.string(),
    date: z.string(),
    link: z.string().optional(),
    milestone: z.boolean().optional(),
  }),
});

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
  loader: glob({ pattern: '**/*.md', base: '../../content/documents' }),
  schema: z.object({
    title: z.string(),
    slug: z.string(),
    description: z.string(),
    category: z.enum([
      'research', 'contract', 'authoring',
      'architecture', 'runtime', 'validation', 'strategy', 'brief',
    ]),
    format: z.string(),
    date: z.string(),
    status: z.string(),
    order: z.number(),
    tags: z.array(z.string()),
    details: z.array(z.string()),
  }),
});

const timeline = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/timeline' }),
  schema: z.object({
    title: z.string(),
    status: z.enum(['complete', 'active', 'next']),
    subtitle: z.string(),
    order: z.number(),
  }),
});

const articles = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/articles' }),
  schema: z.object({
    title: z.string(),
    slug: z.string(),
    description: z.string(),
    date: z.string(),
  }),
});

const documentReviews = defineCollection({
  loader: glob({ pattern: '**/*.md', base: '../../content/document-reviews' }),
  schema: z.object({
    model: z.string(),
    company: z.string(),
    date: z.string(),
    rating: z.number().min(1).max(5),
    initial: z.string().length(1),
    colour: z.string(),
  }),
});

export const collections = { articles, designDocs, reviews, updates, timeline, documentReviews };
