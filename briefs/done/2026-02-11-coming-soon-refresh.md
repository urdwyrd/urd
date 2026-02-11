# Brief: Coming Soon Page Refresh

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-11
**Status:** Complete

### What was done
- Updated `sites/urd.dev/src/layouts/base.astro`: replaced old `favicon.svg` link with `favicon-32.png`, added `apple-touch-icon`, `site.webmanifest` link, full Open Graph meta tags (`og:type`, `og:title`, `og:description`, `og:image`, `og:url`, `og:site_name`), and Twitter/X card meta tags (`summary_large_image`)
- Created `sites/urd.dev/public/site.webmanifest` with PWA manifest pointing to `icon-192.png` and `icon-512.png`, using Gloaming background/theme colour (`#0e0f16`)
- Imported Urd and Wyrd dark SVG logos into `index.astro` using `?raw` imports for Astro inlining
- Added a `.logo-pair` container above the wordmark displaying both logos side by side (Urd left, Wyrd right) at 56px with 2.5rem gap, `opacity: 0.8` at rest with hover to `1.0`
- Logos are `aria-hidden="true"` — the wordmark text remains the accessible label
- Added responsive breakpoint at 480px reducing logos to 44px with tighter gap
- Staggered `fadeUp` animation on the logo pair (0.5s) precedes the hero content (0.6s) for a layered entrance
- Deleted old `favicon.svg` reference (file was already removed by user)
- `pnpm build` succeeds — logos inlined in output HTML, all meta tags present

### Deviations from brief
- **Logo opacity uses 0.8 instead of 0.85** — tested both and 0.8 felt more balanced against the dark background, keeping the logos clearly subordinate to the wordmark
- **Added hover transition on logos** — not specified in the brief, but a natural touch that matches the existing `github-link` hover pattern. Uses `transition: opacity 0.3s ease` — no new animation types introduced

### Issues encountered
- None

### Notes for next brief
- OG image URL is hardcoded to `https://urd.dev/og/urd-og-dark.png` — if the site moves to a different domain or the image changes, the URL needs updating in `base.astro`
- The `og:url` is also hardcoded to `https://urd.dev` — works for now but won't scale to multiple pages without being made dynamic via `Astro.url`
- The Svelte runtime chunk (`client.svelte.*.js`, ~22KB / 8.7KB gzipped) is still being shipped despite no Svelte islands being used — consider deferring the Svelte integration until the first island is actually needed
- The old `favicon.svg` file has been deleted but no proper SVG favicon exists — if high-DPI favicon support is desired, an SVG export from the logo would be needed in a future brief

---

## Context

The Coming Soon page was built during the scaffold brief as a proof-of-concept — it established the Gloaming theme, fonts, and rune canvas, but used no actual logo assets. Now that the logo brief is complete, we have proper SVGs for both Urd and Wyrd, plus favicons, OG images, and PWA icons ready to wire in.

### Current state

- **Wordmark:** Text-only gradient (`URD · WYRD`) in Georgia
- **Favicon:** Inline SVG (`/favicon.svg`) — a gold "U" on dark background, hand-coded during scaffold
- **OG tags:** Only `<meta name="description">` — no `og:image`, `og:title`, `og:type`, or Twitter card tags
- **PWA:** No `site.webmanifest`, so `icon-192.png` and `icon-512.png` are unused
- **Logos:** Not referenced anywhere on the page

### Available assets

| Asset | Path |
|-------|------|
| Urd logo (dark SVG) | `src/assets/logos/urd-logo-dark.svg` |
| Wyrd logo (dark SVG) | `src/assets/logos/wyrd-logo-dark.svg` |
| Favicon PNG | `public/favicon-32.png` |
| Apple touch icon | `public/apple-touch-icon.png` |
| PWA icons | `public/icon-192.png`, `public/icon-512.png` |
| OG image (dark) | `public/og/urd-og-dark.png` |

The Urd logo is three nested U-shaped arcs in gold (`#dab860`, progressively fading opacity). The Wyrd logo is a runic lattice of intersecting lines with purple radial gradients (`#8c68c6` → `#630ab1`).

---

## Implementation Steps

### Step 1: Update `base.astro` head — favicon, OG, and PWA

Replace the single favicon line and add meta tags:

```html
<!-- Favicon -->
<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32.png" />
<link rel="apple-touch-icon" href="/apple-touch-icon.png" />
<link rel="manifest" href="/site.webmanifest" />

<!-- Open Graph -->
<meta property="og:type" content="website" />
<meta property="og:title" content={title} />
<meta property="og:description" content={description} />
<meta property="og:image" content="https://urd.dev/og/urd-og-dark.png" />
<meta property="og:url" content="https://urd.dev" />
<meta property="og:site_name" content="Urd" />

<!-- Twitter / X -->
<meta name="twitter:card" content="summary_large_image" />
<meta name="twitter:title" content={title} />
<meta name="twitter:description" content={description} />
<meta name="twitter:image" content="https://urd.dev/og/urd-og-dark.png" />
```

The old `favicon.svg` has been deleted — `favicon-32.png` is the sole favicon.

### Step 2: Create `site.webmanifest`

Create `sites/urd.dev/public/site.webmanifest`:

```json
{
  "name": "Urd",
  "short_name": "Urd",
  "description": "Define in Urd. Test in Wyrd. Ship Anywhere.",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#0e0f16",
  "theme_color": "#0e0f16",
  "icons": [
    {
      "src": "/icon-192.png",
      "sizes": "192x192",
      "type": "image/png"
    },
    {
      "src": "/icon-512.png",
      "sizes": "512x512",
      "type": "image/png"
    }
  ]
}
```

### Step 3: Incorporate logo SVGs into the page

Place both logos above the wordmark as a visual centrepiece. The logos should flank the wordmark or sit above it — Urd on the left, Wyrd on the right — as a pair that reinforces the duality.

**Approach:** Import the SVGs from `src/assets/logos/` so Astro can inline them. Display them at a restrained size (roughly 48–64px height) with a subtle entrance animation (staggered `fadeUp`, matching the existing hero content timing).

```
┌─────────────────────────────┐
│                             │
│      [Urd]     [Wyrd]       │  ← logo pair, ~48–64px, subtle
│                             │
│       URD  ·  WYRD          │  ← existing wordmark
│                             │
│  Define in Urd. Test in...  │  ← tagline
│         ...                 │
└─────────────────────────────┘
```

**Design constraints:**
- Logos must not overpower the wordmark — they are accents, not heroes
- Use `opacity: 0.85` or similar to keep them from being too bright against the dark background
- Respect the gold/purple brand split — the Urd logo is already gold, the Wyrd logo is already purple, so no additional colour treatment is needed
- Gap between the two logos: `2–3rem`
- Respect `prefers-reduced-motion` — no animation if the user has opted out
- The logos should be `aria-hidden="true"` with the wordmark text remaining as the accessible label

### Step 4: Remove old favicon reference

The hand-coded `favicon.svg` has already been deleted. Update `base.astro` to remove the old `<link rel="icon" type="image/svg+xml" href="/favicon.svg" />` line — the PNG favicon added in Step 1 is the sole favicon now.

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `sites/urd.dev/src/layouts/base.astro` | Modify — add OG/Twitter meta, update favicon links, add manifest link |
| `sites/urd.dev/src/pages/index.astro` | Modify — import and display logo SVGs above wordmark |
| `sites/urd.dev/public/site.webmanifest` | Create |
| `sites/urd.dev/public/favicon.svg` | Already deleted — remove reference from `base.astro` |

---

## Verification

1. **`pnpm build` succeeds** — no broken imports or missing assets
2. **Favicon shows in browser tab** — both PNG and SVG variants work
3. **OG preview:** Paste `https://urd.dev` into [opengraph.xyz](https://opengraph.xyz) or a social media composer — should show the OG image, title, and description
4. **PWA:** Open Chrome DevTools → Application → Manifest — should show the manifest with both icons
5. **Logos render correctly** — Urd in gold on the left, Wyrd in purple on the right, at a tasteful size
6. **Reduced motion:** Enable `prefers-reduced-motion` in DevTools — logos should appear without animation
7. **Mobile responsive:** Logos should scale down or stack gracefully on narrow viewports

---

## What NOT To Do

- Do not change the rune canvas — it's working and within spec
- Do not change the typography system, colour tokens, or any global CSS variables
- Do not add a theme toggle or any interactive elements beyond what's described
- Do not add analytics or tracking scripts
- Do not overdesign — this is still a Coming Soon page, not a landing page
