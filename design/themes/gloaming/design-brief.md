# urd.dev — Design Brief

> The canonical reference for visual identity, tone, and design decisions.
> This document is the source of truth when building, modifying, or
> extending the site. Feed it to any AI working on the project.

---

## 1. What This Site Is

urd.dev is the public engineering journal for Urd + Wyrd, a declarative
schema system for interactive worlds. It exists to:

- Show proof of serious, rigorous engineering work
- Make the architecture legible to technical visitors
- Let people play runnable demos powered by Wyrd
- Document the full history of decisions and artifacts
- Demonstrate what one developer + AI collaboration can produce

The audience is game developers, narrative designers, engine programmers,
and technically curious people who've been burned by vaporware. They are
skeptical by default. Everything on this site must be verifiable.

---

## 2. Aesthetic Identity

### The Register: "Drawn, Not Printed"

The Norse naming (Urd = fate/origin, Wyrd = destiny/becoming) is
deliberate and carries through the visual language. The aesthetic
references geometry and precision — the clean curves of the Well,
the angular lattice of the Web. Modern, warm, confident.

**Key qualities:**
- Warm and legible. Dark backgrounds with high-contrast warm text.
  The warmth is tonal, not a filter — text must always be effortlessly
  readable at length. Legibility wins every trade-off against atmosphere.
- Precise, not playful. This is an engineering tool, not a game.
  The visual tone says "rigorous" and "considered."
- Quiet confidence. The work speaks for itself. No exclamation marks
  in the design — no glows competing for attention, no animation for
  animation's sake.
- Layered depth. Subtle surface elevation, rune canvas in backgrounds,
  gentle glow on Wyrd elements, raised panel navigation with embossed
  controls. The site rewards attention without demanding it.

### What This Site Is NOT

These are hard constraints. If a design choice moves toward any of
these, it's wrong:

- **Not a SaaS landing page.** No hero gradients, no "Get Started Free"
  buttons, no social proof carousels, no pricing tables.
- **Not a dashboard.** No dense grids of metrics, no excessive badges or
  status indicators competing for attention.
- **Not fantasy cosplay.** The Norse register is etymological and tonal,
  not decorative. No swords, shields, Viking helmets, dragon motifs, or
  medieval textures. The rune canvas is the maximum allowable ornament.
- **Not generic developer docs.** No Nextra/Docusaurus/GitBook aesthetic.
  No breadcrumbs. Sidebar navigation, where used (design system, future
  documentation), follows the raised panel pattern — not flat link trees.
  See Section 5 for the navigation panel specification.
- **Not a startup pitch.** No "Join 10,000 developers" counters, no
  testimonial cards, no "As featured in" logo bars.

---

## 3. Colour System

### Foundation Palette

Derived from the system architecture diagram. Every colour has a
semantic meaning tied to a system component. The six **category
colours** (gold, purple, blue, green, amber, rose) are the primary
palette. The light/dim/dark variants are utility tokens for gradients,
backgrounds, and subdued labels — not standalone semantic colours.

```
 Token         Hex       Role
 ────────────  ────────  ──────────────────────────────────────────
 gold          #dab860   Brand primary. Urd. The schema. The contract.
 gold-light    #f0d78c   Emphasis on gold elements.
 gold-dim      #a08a58   Section labels, subtle gold accents.
 gold-dark     #5a4e35   Borders on gold surfaces, footer links.

 purple        #b090dd   Wyrd. Runtime. Execution. Gets the glow.
 purple-light  #ccb0f0   Wyrd in display/hero context.
 purple-dim    #1f1530   Background on Wyrd-related surfaces.

 blue          #6a9acc   Writer Mode. Authoring. Input layer.
 blue-light    #90bbdd   Emphasis in writer contexts.
 blue-dim      #1a2535   Background on authoring surfaces.

 green         #70b080   Testing. Validation. Engineer Mode.
 green-light   #a0d8aa   Emphasis in testing contexts.
 green-dim     #1a2a1f   Background on validation surfaces.

 amber         #cc7a3a   Compiler. Architecture. Build system.
 amber-light   #e8a060   Emphasis on architecture elements.
 amber-dim     #2a1a10   Background on architecture surfaces.

 rose          #cc8888   Research. Discovery. External/pre-system.
 rose-dim      #2a1515   Background on research surfaces.
```

### Surface Palette

```
 Token         Hex       Usage
 ────────────  ────────  ──────────────────────────────────────────
 bg            #0e0f16   Page background. Dark navy, slightly lifted.
 raise         #14151e   Card and panel backgrounds.
 deep          #0a0a12   Inset elements, code blocks, recessed areas.
 surface       #1a1b25   Hover states on raised elements.

 border        #28273a   Default border. Subtle, purple-tinted.
 border-light  #353348   Active/hover borders.
```

### Text Palette

```
 Token         Hex       Usage
 ────────────  ────────  ──────────────────────────────────────────
 text          #f2ece0   Primary text. Warm near-white, high contrast.
 dim           #d4cbb8   Body text, descriptions. Must be effortlessly
                         readable at 18px for sustained reading.
 faint         #9e9585   Tertiary text, metadata, timestamps. Still
                         legible — never decorative-only.
```

### Usage Rules

1. **Gold means Urd / Contract / Schema.** Use it for the .urd.json bar,
   schema spec references, brand marks, section labels, and "by the
   numbers" statistics.

2. **Purple means Wyrd / Runtime / Execution.** Use it for anything
   related to how worlds run. The Wyrd card gets a subtle glow animation;
   nothing else does.

3. **Blue means Writer / Authoring.** Use it for Schema Markdown syntax
   references, writer mode badges, and authoring-related documentation.

4. **Green means Testing / Validation.** Use it for proof point status
   indicators, test results, and the "building in public" live badge.

5. **Amber means Compiler / Architecture.** Use it for compiler pipeline
   stages, architecture documentation, and build system references.

6. **Rose means Research / Discovery.** Use it only for pre-architecture
   documents: landscape analysis, pain points, market research.

7. **Never use colour decoratively.** Every colour application should be
   traceable to a system component. If you can't say "this is purple
   because it relates to Wyrd," don't make it purple.

8. **The background is navy, not black.** `#0e0f16` has a deliberate
   blue-purple undertone. Pure black (`#000`) is never used. The
   background is slightly lifted from the original `#0b0b12` to
   improve text legibility without losing depth.

---

## 4. Typography

### Font Stack

```
 Role      Family          Fallback              Weight Range
 ────────  ──────────────  ────────────────────  ────────────
 Wordmark  Georgia         Times New Roman, serif 400
 Display   Outfit          Helvetica Neue, sans  300–700
 Body      Source Serif 4  Georgia, serif        300–600
 Mono      JetBrains Mono  Consolas, mono        300, 400, 500
```

### Why These Fonts

- **Georgia** — a classic serif with strong, authoritative capitals and
  excellent screen rendering at large sizes. Used exclusively for the
  brand wordmark (URD · WYRD) in uppercase with wide letter-spacing
  (0.08em). Georgia's serifs give the wordmark gravitas and distinguish
  it from the geometric sans used everywhere else. No Google Fonts import
  needed — it ships with every OS.

- **Outfit** — a geometric sans-serif with clean curves and precise lines
  that match the logo artwork. Modern, warm, and highly legible at all
  sizes. Its weight range (300–700) provides clear hierarchy from light
  secondary text to bold hero titles. Used for headings, section labels,
  component names, and badges. Uppercase for labels. Never for the
  brand wordmark.

- **Source Serif 4** — a variable serif designed specifically for on-screen
  reading. Superior legibility to traditional book serifs (Garamond,
  Palatino) at body sizes on dark backgrounds. The sans/serif pairing
  with Outfit creates natural hierarchy — geometric precision for
  structure, warm serif for sustained reading. Used for all body text,
  descriptions, and prose.

- **JetBrains Mono** — a developer-oriented monospace with clear
  letterform distinction. Used for code snippets, metadata, file
  paths, technical badges, and status indicators.

### Sizing Rules

```
 Element                  Font           Size          Weight  Extras
 ───────────────────────  ─────────────  ────────────  ──────  ──────────────
 Brand wordmark (URD)     Georgia        clamp(42,5vw,72)px  400   uppercase, tracking 0.08em
 Section heading          Outfit         26–28px       600     tracking -0.01em
 Component name           Outfit         15–18px       600     uppercase, tracking
 Section label            Outfit         12–13px       600     uppercase, 1.5px tracking
 Badge / pill             Outfit         10–11px       500     uppercase, 1px tracking
 Body text                Source Serif 4 20–21px       400     line-height: 1.7
 Card description         Source Serif 4 17–18px       400     line-height: 1.6
 Metadata / date          Mono           12–13px       400     tracking: 0.06em
 Code / path              Mono           13–14px       400     —
 Pipeline stage           Mono           10–11px       400     —
```

### Rules

1. **Georgia is exclusively for the brand wordmark.** The URD · WYRD
   wordmark uses Georgia 400 in uppercase with 0.08em letter-spacing
   and gradient text fills (gold for URD, purple for WYRD). Georgia is
   never used anywhere else in the UI.

2. **Outfit is for structure, never for prose or the wordmark.** Headings,
   labels, component names, badges. Never for paragraphs, descriptions,
   or the brand wordmark. Use negative tracking (-0.01 to -0.03em) on
   large display sizes.

3. **Source Serif 4 is for reading.** Any text longer than a label gets
   Source Serif 4. Minimum size 16px, preferred 20px for body prose.

4. **Mono is for precision.** Code, file paths, dates, version numbers,
   technical metadata. Never for decorative purposes.

5. **Line-height is generous.** Body text: 1.7. Card text: 1.6. Code: 1.7.
   The dark background needs breathing room for sustained readability.

6. **Letter-spacing on Outfit.** Labels and badges: 1–1.5px. Section
   headings: -0.01em. Hero: -0.03em. Outfit's geometric forms can
   tighten at display sizes for visual weight.

---

## 5. Layout and Spacing

### Page Structure

```
 Max width:    1160px (content), centered
 Side padding: 32px (desktop), 18px (mobile)
 Section gap:  48px vertical, separated by 1px border (var(--border))
```

### Spacing Scale

```
 4px   — Inline gaps (between badge and text)
 6px   — Tight element gaps (pill clusters)
 8px   — Related items within a group
 10px  — Between cards in a list
 12px  — Standard card internal gap
 14px  — Between section header and content
 16px  — Card padding (tight)
 18px  — Card padding (standard), grid gaps
 20px  — Component card internal padding
 24px  — Panel padding
 28px  — Between major subsections
 32px  — Page side padding
 48px  — Between sections
```

### Grid Patterns

- **Hero:** Single column, max-width ~640px for text, ~860px for
  component cards.
- **Proof points + Latest Build:** Two columns, 1.2fr / 0.8fr.
- **Writer / Engineer:** Two columns, 1fr / 1fr.
- **Deep Dives:** Four columns, equal width. Collapses to 2 then 1.
- **History timeline:** Two columns, 160px date / 1fr content.

### Card Anatomy

All cards follow a consistent structure:

```
 ┌─ border: 1px solid var(--border) ─────────────────────┐
 │  border-radius: 8–10px                                │
 │  background: rgba(255,255,255, 0.025–0.04)            │
 │  padding: 18–24px                                     │
 │                                                       │
 │  [Optional: pill badge, top-right]                    │
 │  SECTION LABEL (Outfit, 13px, uppercase, gold-dim)     │
 │  Heading (Outfit, 17–26px, text colour)                │
 │  Description (Source Serif 4, 17–18px, dim colour)     │
 │                                                       │
 │  [Optional: details, action links]                    │
 └───────────────────────────────────────────────────────┘
```

Component-specific cards (compiler, Wyrd, testing) use the component
border colour instead of the default border.

**Accent borders.** Component cards have a 2px left border using
the category gradient. Gold gradients flow vertically (schema/contract).
Purple gradients flow diagonally at 135° (runtime). This adds
dimensionality without weight:

```
 accent-gold    2px left   linear-gradient(to bottom, gold-light → gold-dark)
 accent-purple  2px left   linear-gradient(135deg, purple-light → #6050a0)
 accent-blue    2px left   linear-gradient(to bottom, blue-light → blue-dim)
 accent-green   2px left   linear-gradient(to bottom, green-light → green-dim)
 accent-amber   2px left   linear-gradient(to bottom, amber-light → amber-dim)
 accent-rose    2px left   linear-gradient(to bottom, rose-light → rose-dim)
```

### Navigation Panel

Sidebar and table-of-contents panels use a Zachtronics-inspired raised
panel pattern. Each interactive element has a pixel of depth — opposing
inset highlights and outer shadows that create the feel of embossed
interface controls on a physical instrument panel.

```
 ┌─ box-shadow: inset 1px 1px 0 rgba(255,255,255,0.03) ──┐
 │  box-shadow: inset -1px 0 0 rgba(0,0,0,0.3)           │
 │  box-shadow: 2px 0 8px rgba(0,0,0,0.3)                │
 │  background: var(--raise)                              │
 │  width: 242px                                          │
 │                                                        │
 │  ┌─ HEADER ──────────────────────────────────────────┐ │
 │  │  background: gold gradient wash (0.04 opacity)    │ │
 │  │  box-shadow: inset 0 1px 0 rgba(gold, 0.1)       │ │
 │  │  Title: Outfit 14px 600 uppercase gold            │ │
 │  │  Subtitle: Mono 10px gold-dark                    │ │
 │  └───────────────────────────────────────────────────┘ │
 │                                                        │
 │  ┌─ SECTION LINK (raised micro-button) ──────────────┐ │
 │  │  Mono 13.5px 500 --dim                            │ │
 │  │  background: gradient(white 0.015 → 0.005)        │ │
 │  │  box-shadow: inset 0 1px 0 white, 0 1px 0 black  │ │
 │  │  border-left: 2px solid transparent               │ │
 │  │  :hover → gold tint + gold left border            │ │
 │  └───────────────────────────────────────────────────┘ │
 │                                                        │
 │    Sub-item: Mono 12.5px, --faint, 26px left indent    │
 │    Divider: gradient(transparent → white 0.04 → trans) │
 │                                                        │
 └────────────────────────────────────────────────────────┘
```

**Key depth technique:** Each section link gets a 1px inset highlight on
top (rgba white) and a 1px outer shadow on bottom (rgba black). This is
the minimum amount of relief needed to create a sense of physicality —
the control looks raised from the panel surface without looking 3D or
skeuomorphic.

---

## 6. Animation and Motion

### Philosophy

Animation is atmospheric, not performative. It creates a sense of
quiet aliveness — the site breathes, it doesn't dance.

### Allowed Animations

```
 Animation       Duration   Easing          Usage
 ──────────────  ─────────  ──────────────  ──────────────────────
 fadeUp          0.5–0.6s   ease-out        Page load, staggered by 0.06–0.08s
 fadeIn          0.25s      ease-out        Expanding card content
 pulse           2.5s       ease-in-out     Status dots (building, in-progress)
 pulseRing       2.5s       ease-in-out     Ring around status dots
 wyrdGlow        4s         infinite        ONLY on Wyrd component card
 float           3s         ease-in-out     Scroll indicator only
 runeCanvas      continuous  —              Full-page background rune drift, opacity < 0.05
```

### Rules

1. **The Wyrd glow is exclusive.** No other element gets a box-shadow
   glow animation. This is how the runtime card signals its special
   status as the canonical oracle.

2. **Entry animations are staggered.** Elements in a group (hero title,
   tagline, description, component cards) use increasing animation-delay
   (0.06–0.08s increments) for a composed reveal.

3. **Hover transitions are fast and subtle.** 0.15s ease on background
   and border colour. No transforms except on navigation chips (1px
   translateY is acceptable).

4. **The rune canvas is nearly invisible.** Maximum opacity 0.05. Fixed
   position, full-page, z-index 0 behind all content and panels. It
   should be noticeable only when you're not looking directly at it.
   If someone asks "is the background moving?" — correct calibration.

5. **No animation on text.** Text never slides, bounces, types out, or
   transitions colour. It appears and stays.

---

## 7. Document Taxonomy

Six categories, each mapped to a system architecture component.
Full specification in `urd-document-taxonomy.md`.

```
 Category       Colour    Meaning
 ─────────────  ────────  ─────────────────────────────────
 Research       Rose      Pre-architecture discovery
 Contract       Gold      The .urd.json data model
 Authoring      Blue      Writer Mode / input layer
 Architecture   Amber     Compiler + system design
 Runtime        Purple    Wyrd execution engine
 Validation     Green     Testing framework
```

Every document on the site gets a pill badge in its category colour.
Filter buttons in the Deep Dives section correspond to these categories.
History timeline entries use small colour-coded dots.

See `urd-document-taxonomy.md` for full assignment rules and the
decision flowchart for categorising future documents.

---

## 8. Copy Voice

### Tone: Precise, Understated, Confident

The site speaks like a senior engineer explaining their work to
a peer — no hype, no hedging, no filler. Every sentence should
either inform or provoke curiosity.

### Examples of On-Brand Copy

```
 ✓  "If it can't be reproduced, it doesn't go on the front page."
 ✓  "Not 'no code.' A compiler, a runtime, and a contract between them."
 ✓  "Define in Urd. Test in Wyrd. Ship Anywhere."
 ✓  "The judgment is human. The throughput is superhuman."
 ✓  "No trailers. Runnable worlds."
 ✓  "The canonical behaviour oracle."
 ✓  "Writers never see JSON."
```

### Examples of Off-Brand Copy

```
 ✗  "Unleash the power of declarative world-building!"
 ✗  "Getting started is easy — just three simple steps!"
 ✗  "Join thousands of developers who trust Urd."
 ✗  "Revolutionary AI-powered narrative engine."
 ✗  "Build amazing interactive experiences in minutes!"
 ✗  "The future of storytelling is here."
```

### Rules

1. **No superlatives.** Not "the best," "the most powerful," "the fastest."
   State what it does. Let the reader judge.

2. **No urgency.** No "Don't miss out," "Limited time," "Get started now."
   The site is patient. It will still be here.

3. **Technical precision over marketing abstraction.** Say "5-phase
   compiler pipeline" not "advanced processing engine." Say "Monte Carlo
   simulation with 10,000 runs" not "rigorous testing."

4. **Short sentences for impact.** The tagline is seven words. Section
   headlines are 4–10 words. Let the prose in body text do the explaining.

5. **Active voice, present tense.** "The compiler validates" not "validation
   is performed by the compiler."

6. **British English spelling.** Behaviour, colour, visualisation, defence.
   Consistent with the project author's usage.

---

## 9. Iconography and Visual Elements

### What's Allowed

- **Rune canvas** — Futhark characters, drifting slowly upward, opacity
  < 0.05. Fixed full-page background at z-index 0, visible behind all
  content. The canvas sits beneath both the sidebar panel (z-index 100)
  and the main content area (z-index 1).
- **Status dots** — Pulsing circles with ring animation. Used for
  "building in public" badge and proof point status.
- **Coloured dots** — Small, static, 7px circles in taxonomy colours.
  Used in the history timeline.
- **Diamond markers** — ◆ at 7px, in taxonomy colours, for list items
  inside expanded cards.
- **Arrows** — → character in mono font for pipeline stages and
  navigational hints. Never SVG arrows.
- **Emoji** — 📄 for the .urd.json contract bar only. No other emoji
  anywhere.

### What's NOT Allowed

- SVG icons or icon libraries (Lucide, Heroicons, etc.)
- Custom illustrations or graphics
- Photographs or screenshots (until there's a working product)
- Decorative borders, dividers, or ornaments beyond simple 1px lines
- Gradients on text (exception: the brand wordmark uses gradient fills)
- Background images (the rune canvas is a programmatic element, not
  an image)

---

## 10. Responsive Behaviour

### Breakpoints

```
 Width     Behaviour
 ────────  ──────────────────────────────────────────────
 > 980px   Full layout. All grids at specified columns.
 520–980   Two-column grids collapse to single column.
            Deep Dives grid: 4 → 2 columns.
            History timeline: 2-col → stacked.
 < 520px   Single column everything.
            Deep Dives grid: 2 → 1 column.
            Nav chips wrap to second line.
            Hero title clamps down to 48px.
            Side padding: 32px → 18px.
```

### Rules

1. **Content order doesn't change.** The narrative flow is the same on
   mobile — hero, proof points, pipeline, play, deep dives, history.

2. **Cards don't lose content.** Nothing is hidden on mobile. Cards may
   stack but never truncate.

3. **Component cards stack vertically on mobile.** Compiler, Wyrd, Testing
   become a single column with the same internal structure.

---

## 11. Theme Extensibility

This design brief describes the "Nightfall" theme — the default dark
theme for urd.dev. Future themes (e.g., a light "Parchment" variant,
a high-contrast "Terminal" variant) should:

1. **Preserve the colour taxonomy.** Gold still means Contract. Purple
   still means Wyrd. The semantic mapping is invariant across themes.

2. **Preserve the font stack.** Georgia / Outfit / Source Serif 4 /
   JetBrains Mono. Alternative themes may adjust weights and sizes but
   not families. Georgia remains the wordmark font across all themes.

3. **Preserve the copy voice.** Tone and language rules are independent
   of visual theme.

4. **Preserve the component vocabulary.** Cards, pills, status dots,
   pipeline visualisations. The patterns remain; colours and surfaces
   may change.

5. **Define their own surface palette.** A light theme needs its own
   bg / raise / deep / surface / border values. The semantic colours
   (gold, purple, etc.) may need adjusted dim/light variants for
   legibility on light backgrounds.

6. **Name their aesthetic register.** "Nightfall" is geometric, warm,
   confident. A "Parchment" theme might be "written, warm, scholarly."
   A "Terminal" theme might be "typed, cold, functional." The register
   guides all micro-decisions.

---

## 12. Validation Checklist

Before shipping any page or component, verify:

- [ ] Every colour used maps to a semantic token (no raw hex in components)
- [ ] Georgia is only used for the brand wordmark (URD · WYRD, uppercase)
- [ ] Outfit is only used for structure (headings, labels, badges) — never the wordmark
- [ ] Source Serif 4 is used for all prose at ≥16px with line-height ≥1.6
- [ ] No element other than the Wyrd card has a glow animation
- [ ] Status dots are used only for live/active states
- [ ] Document pills use the correct taxonomy colour
- [ ] Primary text contrast ≥ 7:1, dim text ≥ 4.5:1 against bg (WCAG AAA / AA)
- [ ] No marketing superlatives in copy
- [ ] British English spelling throughout
- [ ] Responsive layout tested at 520px and 980px breakpoints
- [ ] Rune canvas opacity ≤ 0.05, fixed full-page at z-index 0
- [ ] No icon libraries or decorative SVGs
- [ ] Sidebar panels use raised panel pattern with depth shadows (not flat link trees)
- [ ] Gradient text fills only on the brand wordmark
