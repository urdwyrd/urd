# Brief: TOC Scroll Highlighting and Heading Cleanup

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-12
**Status:** Complete

### What was done

- Removed `rehype-autolink-headings` from `sites/urd.dev/astro.config.mjs` — headings are now plain text with `id` attributes (from `rehype-slug`), no longer wrapped in `<a>` tags
- Added `.doc-toc-link.active` and `.depth-3 .doc-toc-link.active` styles to `sites/urd.dev/src/layouts/document.astro` — gold left-border and text colour matching the hover state
- Added IntersectionObserver `<script>` to `sites/urd.dev/src/layouts/document.astro` — observes all `.prose h2[id]` and `.prose h3[id]` elements, toggles `active` class on the matching TOC link

### Deviations from brief

- None — implementation followed the brief exactly

### Issues encountered

- None

### Notes for next brief

- The `rehype-autolink-headings` package is still installed as a dependency but no longer imported — can be removed with `pnpm remove` in a future cleanup pass
- The observer uses `rootMargin: '0px 0px -80% 0px'` which triggers when a heading enters the top 20% of the viewport — this may need tuning based on user feedback

---

## Context

The document pages have a sticky TOC sidebar with anchor links to each H2/H3 heading. Two issues:

1. **No active state** — the TOC doesn't indicate which section the reader is currently viewing. As you scroll through a long document (40+ sections in `architecture.md`), there's no visual feedback about your position.

2. **Headings are underlined and clickable** — `rehype-autolink-headings` wraps every heading in an `<a>` tag with `behavior: "wrap"`. These anchor links then inherit `.prose a` styles (gold colour, underline), making headings look like hyperlinks. This is confusing and adds no value when the TOC sidebar already provides navigation. The heading IDs (from `rehype-slug`) are still needed for the TOC — only the `<a>` wrapping should go.

### Prerequisites

- Document pages working at `/documents/{slug}` with TOC sidebar
- `rehype-slug` and `rehype-autolink-headings` registered in `astro.config.mjs`
- Heading IDs generated correctly and matching TOC link hrefs

### Design References

- `design/themes/gloaming/design-brief.md` — colour and typography tokens
- TOC sidebar uses the raised panel pattern (§5 of design brief)

---

## Implementation Steps

### Step 1: Remove rehype-autolink-headings

Remove `rehype-autolink-headings` from `astro.config.mjs`. Keep `rehype-slug` — it generates the `id` attributes that the TOC links target.

**File:** `sites/urd.dev/astro.config.mjs`

Before:
```js
rehypePlugins: [
  rehypeSlug,
  [rehypeAutolinkHeadings, { behavior: "wrap" }],
],
```

After:
```js
rehypePlugins: [rehypeSlug],
```

Remove the import line for `rehype-autolink-headings` too. Optionally uninstall the package (`pnpm remove rehype-autolink-headings --filter urd.dev`), but leaving it as an unused dependency is harmless for now.

**Result:** Headings render as plain text with their `id` attributes. No more gold underlines or clickable heading text.

### Step 2: Add scroll-aware TOC highlighting

Add a small vanilla JS `<script>` to `sites/urd.dev/src/layouts/document.astro` that observes which heading is currently in view and highlights the corresponding TOC link.

**Approach:** Use `IntersectionObserver` on all heading elements that have IDs matching TOC entries. When a heading enters the viewport, add an `active` class to the corresponding TOC link. This is zero-dependency, performant, and works with static HTML.

```html
<script>
  const tocLinks = document.querySelectorAll('.doc-toc-link');
  const headings = document.querySelectorAll('.prose h2[id], .prose h3[id]');

  if (tocLinks.length > 0 && headings.length > 0) {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const id = entry.target.id;
            tocLinks.forEach((link) => {
              link.classList.toggle(
                'active',
                link.getAttribute('href') === `#${id}`
              );
            });
          }
        }
      },
      { rootMargin: '0px 0px -80% 0px' }
    );

    for (const heading of headings) {
      observer.observe(heading);
    }
  }
</script>
```

**`rootMargin: '0px 0px -80% 0px'`** — triggers when a heading enters the top 20% of the viewport, which feels natural for "you're now reading this section."

**Respect `prefers-reduced-motion`:** The highlight itself is a colour change, not an animation, so no motion concern. The existing `transition: color 0.15s ease` on `.doc-toc-link` already respects the reduced-motion media query in `global.css`.

### Step 3: Add active TOC link styles

Add CSS for the `.doc-toc-link.active` state in `sites/urd.dev/src/layouts/document.astro`:

```css
.doc-toc-link.active {
  color: var(--gold);
  border-left-color: var(--gold);
}

.depth-3 .doc-toc-link.active {
  color: var(--gold-dim);
  border-left-color: var(--gold-dim);
}
```

This matches the existing hover state but persists while the section is in view. H3 entries get a subtler gold to maintain the visual hierarchy.

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `sites/urd.dev/astro.config.mjs` | Modify — remove rehype-autolink-headings |
| `sites/urd.dev/src/layouts/document.astro` | Modify — add IntersectionObserver script + active TOC styles |

---

## Verification

1. `pnpm build` succeeds
2. Document headings are plain text (no underline, no gold colour, not clickable)
3. Heading `id` attributes still present (TOC links still work)
4. Scrolling through a document highlights the current section in the TOC
5. Only one TOC link is highlighted at a time
6. Highlighting works on long documents (`architecture.md` with 40+ sections)
7. Mobile: no JS errors (TOC may be inline above content, highlighting still works)

---

## What NOT To Do

- Do not remove `rehype-slug` — the heading IDs are required for TOC navigation
- Do not add a scroll-spy library — vanilla `IntersectionObserver` is sufficient
- Do not add smooth-scroll behaviour — the browser default is fine and respects user preferences
- Do not add a "back to top" button — out of scope
- Do not highlight TOC links on page load before scrolling — let the observer handle initial state naturally
