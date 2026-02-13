# Brief: Theme Switching (Gloaming ↔ Parchment)

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-13
**Status:** Complete

### What was done

- Added full Parchment theme token block in `global.css` under `[data-theme="parchment"]` — 30+ CSS custom properties covering surfaces, text, category colours (darkened for light background), and document taxonomy mappings.
- Introduced 5 new theme-dependent depth tokens (`--card-shadow`, `--card-overlay`, `--toc-shadow`, `--rune-colour`, `--rune-max-opacity`) in `:root` with Parchment overrides.
- Added `color-scheme: light` override for `html[data-theme="parchment"]`.
- Replaced hardcoded TOC panel `box-shadow` in `document.astro` with `var(--toc-shadow)`.
- Replaced 3 hardcoded `rgba(255, 255, 255, ...)` backgrounds in `DocumentExplorer.svelte` with `var(--card-overlay)`.
- Added `box-shadow: var(--card-shadow)` to cards in DocumentExplorer (list + grid views) and PeerReview.
- Rewrote rune canvas in `index.astro` to read `--rune-colour` and `--rune-max-opacity` from computed styles, with a `MutationObserver` on `data-theme` to react to live theme changes.
- Imported both dark and light logo SVGs; rendered both in DOM with CSS visibility toggling via `[data-theme="parchment"]` selector.
- Added theme toggle button to Nav — displays "Light" on dark theme, "Dark" on light theme. Mono font, matches nav link style. Updates `aria-label` and `title` dynamically.
- Added blocking inline `<script>` in `base.astro` `<head>` that reads `localStorage('urd-theme')` before first paint, falling back to `prefers-color-scheme: light` if no stored preference.

### What changed from the plan

- Combined Phases 4 and 5 (toggle UI + `prefers-color-scheme`) into a single blocking script rather than separate implementations.
- Toggle button uses `Light` / `Dark` text labels rather than the `◐` character — clearer for users.
- ProjectLog component needed no changes — it uses a timeline layout with borders, no card backgrounds.

### What was left out or deferred

- Theme-specific OG images (as planned — dark OG image is fine for both themes).
- Theme-specific rune canvas characters (both themes use same Futhark set, as planned).

### Observations

- The CSS custom property architecture made this straightforward — nearly all component styles already used `var(--token)` references. Only 3 locations in DocumentExplorer and 1 in document.astro had hardcoded `rgba()` values.
- The `MutationObserver` approach for the rune canvas is clean — it watches `data-theme` attribute changes and re-reads CSS vars on each toggle, so the canvas transitions instantly.
- Build size impact is minimal: CSS grew by ~1.2KB (the Parchment token block), and the additional logo SVGs are inlined.

---

## Goal

Allow visitors to toggle between the Gloaming (dark) and Parchment (light) themes on urd.dev. Both themes are fully specified in `design/themes/`. The site currently ships only Gloaming with hardcoded values in several places.

---

## Context

### What exists

- **Gloaming design brief** — `design/themes/gloaming/design-brief.md` (canonical, fully specified)
- **Parchment design brief** — `design/themes/parchment/design-brief.md` (canonical, fully specified)
- **Design system HTML** — living reference pages for both themes in `design/themes/*/design-system.html`
- **Logo variants** — dark and light SVGs for both Urd and Wyrd in `sites/urd.dev/src/assets/logos/`
- **CSS custom properties** — all component styles already use `var(--token)` for surfaces, text, and category colours

### What needs to change

The site has **no theme switching infrastructure**. All Gloaming values are baked in at `:root` level with no alternative selector. Several places use hardcoded values that bypass the token system.

---

## Audit of Hardcoded Theme Values

### 1. `global.css` — `:root` block (line 76–136)
All CSS custom properties set to Gloaming hex values. No `[data-theme="parchment"]` block exists.

### 2. `global.css` — `html` rule (line 148)
`color-scheme: dark` — must switch to `light` for Parchment.

### 3. `index.astro` — Rune canvas (lines 64, 119, 147)
- `MAX_OPACITY = 0.05` — Parchment spec says 0.03
- `rgba(218, 184, 96, ...)` (gold) — Parchment spec says `rgba(60, 48, 30, ...)` (dark sepia)

### 4. `index.astro` — Logo imports (line 4–5)
Imports `urd-logo-dark.svg` and `wyrd-logo-dark.svg`. Parchment needs `-light` variants.

### 5. `document.astro` — TOC panel box-shadow (lines 118–121)
```css
box-shadow:
  inset 1px 1px 0 rgba(255, 255, 255, 0.03),
  inset -1px 0 0 rgba(0, 0, 0, 0.3),
  2px 0 8px rgba(0, 0, 0, 0.3);
```
Gloaming raised pattern. Parchment needs the recessed pattern:
```css
box-shadow:
  inset 0 1px 3px rgba(0, 0, 0, 0.06),
  0 1px 0 rgba(255, 255, 255, 0.8);
```

### 6. `DocumentExplorer.svelte` — `rgba(255, 255, 255, ...)` backgrounds (lines 427, 689, 707)
White-alpha overlays for card backgrounds. Parchment uses `var(--raise)` with subtle drop shadow instead.

### 7. Open Graph meta (base.astro, lines 28, 36)
`og:image` points to `urd-og-dark.png`. If a Parchment OG image exists or is created, this should be dynamic. Low priority — can remain dark-branded initially.

---

## Implementation Plan

### Phase 1 — CSS token layer

**File: `global.css`**

1. Keep the existing `:root` block as Gloaming (dark is the default).

2. Add a `[data-theme="parchment"]` selector block with all Parchment values from the design brief:

```css
[data-theme="parchment"] {
  /* Surface */
  --bg: #f4ede0;
  --raise: #faf6ef;
  --deep: #e8e0d0;
  --surface: #ede5d6;
  --border: #d4c8b0;
  --border-light: #c4b89e;

  /* Text */
  --text: #2c2418;
  --dim: #5c5040;
  --faint: #8a7e6c;

  /* Category colours (darkened for light surface) */
  --gold: #8a6d18;
  --gold-light: #dab860;
  --gold-dim: #c4a84c;
  --gold-dark: #6a5510;

  --purple: #6b4fa0;
  --purple-light: #b090dd;
  --purple-dim: #f0ecf5;

  --blue: #3a6a98;
  --blue-light: #6a9acc;
  --blue-dim: #ecf2f8;

  --green: #3a7a4a;
  --green-light: #70b080;
  --green-dim: #ecf5ee;

  --amber: #a05a1a;
  --amber-light: #cc7a3a;
  --amber-dim: #f5ede5;

  --rose: #a05050;
  --rose-light: #cc8888;
  --rose-dim: #f5ece8;

  /* Document taxonomy (same mappings, new values inherited) */
  --doc-research: var(--rose);
  --doc-research-dim: var(--rose-dim);
  --doc-contract: var(--gold);
  --doc-contract-dim: #f5f0e0;
  --doc-authoring: var(--blue);
  --doc-authoring-dim: var(--blue-dim);
  --doc-architecture: var(--amber);
  --doc-architecture-dim: var(--amber-dim);
  --doc-runtime: var(--purple);
  --doc-runtime-dim: var(--purple-dim);
  --doc-validation: var(--green);
  --doc-validation-dim: var(--green-dim);
  --doc-strategy: var(--gold);
  --doc-strategy-dim: var(--doc-contract-dim);
}
```

3. Add theme-aware `color-scheme`:

```css
html { color-scheme: dark; }
html[data-theme="parchment"] { color-scheme: light; }
```

4. Add new CSS custom properties for theme-dependent shadows and overlays (used by components):

```css
:root {
  /* Theme-dependent depth tokens */
  --card-shadow: none;
  --card-overlay: rgba(255, 255, 255, 0.03);
  --toc-shadow: inset 1px 1px 0 rgba(255, 255, 255, 0.03),
                inset -1px 0 0 rgba(0, 0, 0, 0.3),
                2px 0 8px rgba(0, 0, 0, 0.3);
  --rune-colour: 218, 184, 96;
  --rune-max-opacity: 0.05;
}

[data-theme="parchment"] {
  --card-shadow: 0 1px 3px rgba(0, 0, 0, 0.04);
  --card-overlay: transparent;
  --toc-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.06),
                0 1px 0 rgba(255, 255, 255, 0.8);
  --rune-colour: 60, 48, 30;
  --rune-max-opacity: 0.03;
}
```

### Phase 2 — Replace hardcoded values in components

**File: `document.astro`**
- Replace the TOC panel's hardcoded `box-shadow` with `var(--toc-shadow)`.

**File: `DocumentExplorer.svelte`**
- Replace `rgba(255, 255, 255, 0.03)` and `rgba(255, 255, 255, 0.02)` backgrounds with `var(--card-overlay)`.
- Add `box-shadow: var(--card-shadow)` to card containers.

**File: `index.astro` — Rune canvas script**
- Read `--rune-colour` and `--rune-max-opacity` from computed styles instead of hardcoded constants:
```js
const style = getComputedStyle(document.documentElement);
const RUNE_COLOUR = style.getPropertyValue('--rune-colour').trim();
const MAX_OPACITY = parseFloat(style.getPropertyValue('--rune-max-opacity')) || 0.05;
```
- Use `rgba(${RUNE_COLOUR}, ${opacity})` for fill style.
- Re-read values when theme changes (listen for a custom event or observe `data-theme` attribute).

### Phase 3 — Logo switching

**File: `index.astro`**

The logos need to swap between dark and light variants. Two approaches:

**Approach A — CSS visibility (recommended):** Import both logo variants. Render both sets in the DOM, use CSS to show/hide based on `[data-theme]`:

```css
[data-theme="parchment"] .logo-dark { display: none; }
[data-theme="parchment"] .logo-light { display: block; }
:root .logo-light { display: none; }
```

This avoids JS dependency and works with static HTML.

**Approach B — JS swap:** Use a script to swap the `innerHTML` of the logo containers. More complex, less reliable with static builds.

### Phase 4 — Theme toggle UI

**File: `Nav.astro`**

Add a theme toggle button to the navigation bar, right side, before or after the GitHub link.

**Button spec:**
- Text characters only (no icons): `◐` (half-moon, U+25D0) or similar. Alternatively, just the words `Dark` / `Light` in mono font.
- Mono font, 12px, `var(--faint)` colour, transitions to `var(--text)` on hover.
- No background, no border — just a clickable text element matching the nav link style.
- `aria-label="Switch to light theme"` / `"Switch to dark theme"` (dynamic).
- `title` attribute matching the aria-label.

**Behaviour:**
1. On click, toggle `data-theme` between absent (Gloaming default) and `"parchment"` on `<html>`.
2. Persist choice in `localStorage` under key `urd-theme`.
3. On page load, apply the stored preference before first paint (inline `<script>` in `<head>` to prevent flash).

**File: `base.astro`**

Add a blocking inline script in `<head>` to apply the stored theme before the page renders:

```html
<script is:inline>
  (function() {
    const stored = localStorage.getItem('urd-theme');
    if (stored === 'parchment') {
      document.documentElement.setAttribute('data-theme', 'parchment');
    }
  })();
</script>
```

This must run synchronously before CSS is applied to prevent FOUC (flash of unstyled content).

### Phase 5 — Respect `prefers-color-scheme`

If no localStorage preference is set, respect the OS-level preference:

```html
<script is:inline>
  (function() {
    const stored = localStorage.getItem('urd-theme');
    if (stored) {
      if (stored === 'parchment') {
        document.documentElement.setAttribute('data-theme', 'parchment');
      }
    } else if (window.matchMedia('(prefers-color-scheme: light)').matches) {
      document.documentElement.setAttribute('data-theme', 'parchment');
    }
  })();
</script>
```

Once the user manually toggles, their explicit choice overrides the OS preference until they clear it.

---

## Files Changed

| File | Action |
|------|--------|
| `sites/urd.dev/src/styles/global.css` | Add `[data-theme="parchment"]` block, theme-dependent depth tokens |
| `sites/urd.dev/src/layouts/base.astro` | Add blocking theme script in `<head>` |
| `sites/urd.dev/src/components/Nav.astro` | Add theme toggle button with JS |
| `sites/urd.dev/src/pages/index.astro` | Logo switching, rune canvas reads CSS vars |
| `sites/urd.dev/src/layouts/document.astro` | Replace hardcoded TOC shadow with `var(--toc-shadow)` |
| `sites/urd.dev/src/components/DocumentExplorer.svelte` | Replace `rgba(255,255,255,...)` with `var(--card-overlay)` |
| `sites/urd.dev/src/components/PeerReview.svelte` | Add `box-shadow: var(--card-shadow)` if cards lack it |
| `sites/urd.dev/src/components/ProjectLog.svelte` | Same — verify cards use tokens |

---

## Verification

1. **Default load (no localStorage):** Site renders in Gloaming (dark). All existing visuals unchanged.
2. **OS prefers light:** If no stored preference and OS is light mode, site renders in Parchment.
3. **Toggle click:** Clicking the toggle switches theme instantly. No flash, no layout shift.
4. **Persistence:** Refresh the page — theme choice persists via localStorage.
5. **Rune canvas:** Runes are gold on Gloaming (opacity ≤ 0.05), dark sepia on Parchment (opacity ≤ 0.03).
6. **Logos:** Urd and Wyrd logos swap to light variants on Parchment.
7. **TOC panel:** Gloaming uses raised/embossed shadow. Parchment uses recessed shadow.
8. **Document cards:** Gloaming uses white-alpha overlays. Parchment uses solid `var(--raise)` with subtle drop shadow.
9. **Category colours:** All pills, badges, and document taxonomy colours use the correct darkened Parchment variants.
10. **Text contrast:** Primary text ≥ 7:1 (AAA), dim text ≥ 4.5:1 (AA) on both themes.
11. **`prefers-reduced-motion`:** Still respected — animations disabled regardless of theme.
12. **Build:** `pnpm build` succeeds. No hydration errors. 13 pages + 3 endpoints.
13. **Document pages:** Every `/documents/{slug}` page renders correctly in both themes.

---

## Out of Scope

- **Theme-specific OG images** — the dark OG image is fine for both themes initially.
- **Theme-specific rune canvas characters** — both themes use the same Futhark set.
- **Theme editor or third themes** — the architecture supports it but not needed now.
- **Server-side theme detection** — this is a static site; everything is client-side.

---

