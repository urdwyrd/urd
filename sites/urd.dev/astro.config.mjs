import { defineConfig } from "astro/config";
import { readFileSync } from "node:fs";
import svelte from "@astrojs/svelte";
import sitemap from "@astrojs/sitemap";
import tailwindcss from "@tailwindcss/vite";
import rehypeSlug from "rehype-slug";

// Read compiler version from Cargo.toml for WASM cache-busting.
const cargoToml = readFileSync("../../packages/compiler/Cargo.toml", "utf-8");
const compilerVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? "0";

export default defineConfig({
  site: "https://urd.dev",
  integrations: [svelte(), sitemap()],
  markdown: {
    rehypePlugins: [rehypeSlug],
  },
  vite: {
    plugins: [tailwindcss()],
    define: {
      __WASM_CACHE_BUST__: JSON.stringify(compilerVersion),
    },
  },
});
