import type { GetStaticPaths } from 'astro';
import { getCollection } from 'astro:content';

export const getStaticPaths: GetStaticPaths = async () => {
  const docs = await getCollection('designDocs');
  return docs.map((doc) => ({ params: { slug: doc.data.slug } }));
};

export async function GET({ params }: { params: { slug: string } }) {
  const allReviews = await getCollection('documentReviews');
  const slug = params.slug;

  const reviews = allReviews
    .filter((entry) => entry.id.split('/')[0] === slug)
    .map((entry) => {
      const filename = entry.id.split('/').pop()!;
      return {
        model: entry.data.model,
        company: entry.data.company,
        date: entry.data.date,
        rating: entry.data.rating,
        initial: entry.data.initial,
        colour: entry.data.colour,
        icon: `/images/reviews/${filename}.png`,
        quote: (entry.body ?? '').trim(),
      };
    });

  return new Response(JSON.stringify(reviews, null, 2), {
    headers: { 'Content-Type': 'application/json' },
  });
}
