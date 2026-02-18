import type { APIRoute } from 'astro';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

export const GET: APIRoute = async () => {
  try {
    const reportPath = resolve(process.cwd(), 'src/data/compiler-test-report.json');
    const raw = readFileSync(reportPath, 'utf-8');
    return new Response(raw, {
      headers: { 'Content-Type': 'application/json' },
    });
  } catch {
    return new Response('null', {
      headers: { 'Content-Type': 'application/json' },
    });
  }
};
