import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";
import sitemap from "@astrojs/sitemap";
import tailwindcss from "@tailwindcss/vite";
import rehypeSlug from "rehype-slug";

export default defineConfig({
  site: "https://urd.dev",
  integrations: [svelte(), sitemap()],
  markdown: {
    rehypePlugins: [rehypeSlug],
  },
  vite: {
    plugins: [tailwindcss()],
  },
});
