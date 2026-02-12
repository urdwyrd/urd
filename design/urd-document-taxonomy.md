# Urd · Wyrd — Document Color Taxonomy

A consistent color coding system for all project artifacts, derived from the
architecture diagram's existing component colors.

---

## The Seven Categories

Each category maps to a zone in the system architecture diagram or to a
cross-cutting concern. Every document produced belongs to exactly one
category.

The base hues below are canonical — derived from the architecture
diagram and invariant across themes. Themes may adjust saturation and
lightness for their surface palette, but the hue identity is fixed.

```
 Category       Color           Hex         Architecture Zone
 ─────────────  ──────────────  ──────────  ─────────────────────────────
 Research       Rose            #cc8888     Pre-architecture (discovery)
 Contract       Gold            #dab860     The .urd.json data model
 Authoring      Blue            #6a9acc     Writer Mode / input layer
 Architecture   Amber           #cc7a3a     Compiler + system design
 Runtime        Purple          #b090dd     Wyrd execution engine
 Validation     Green           #70b080     Testing framework
 Strategy       Gold            #dab860     Product vision, roadmap, positioning
```

**Note:** Strategy shares gold with Contract. Both relate to "what the
project is" — Contract defines the technical data model, Strategy defines
the product and market positioning. They share a hue but are distinct
categories for filtering and organisation.

### Extended Palette (theme-derived)

Each category has variant tokens (dim, light, border) used for backgrounds,
emphasis, and borders. These variants are **theme-specific** — derived from
the base hues above but adjusted for contrast against each theme's surface
palette.

```
 Variant     Purpose                        Derivation
 ──────────  ─────────────────────────────  ─────────────────────────────
 {cat}-dim   Background tint on cards       Base hue, desaturated, near-black
 {cat}-light Emphasis text, active states   Base hue, lightened for contrast
 {cat}-border Card and pill borders         Base hue, midpoint between dim and base
```

For concrete hex values and CSS variables, see the active theme's design
brief (e.g., `themes/gloaming/DESIGN_BRIEF.md` §3 Colour System).

---

## Current Document Assignments

### Research — Rose #cc8888
> *Pre-architecture discovery. The "why" before the "what."*

| Document                          | Format               | Date     | Size         |
|-----------------------------------|----------------------|----------|--------------|
| Landscape Analysis & Gap Assessment | Research Report      | Jan 2026 | 12,000+ words |
| Developer Pain Points Report      | Forum-Sourced Research | Jan 2026 | 8,000+ words |

**Why rose?** These documents exist before the architecture. They define the
problem space and justify the project's existence. Rose is used in the
architecture diagram for downstream consumers (game engines) — entities
outside the core system. Research occupies the same structural position:
it informs the system but isn't part of it.

**Future documents in this category:** Market updates, competitive analysis
refreshes, user research, community feedback synthesis.

---

### Strategy — Gold #dab860
> *Product vision, roadmap, and market positioning. The "what" and "why" of the project.*

| Document                          | Format               | Date     | Size         |
|-----------------------------------|----------------------|----------|--------------|
| Urd + Wyrd Product Vision v2.0    | Product Strategy      | Feb 2026 | 3,000 words  |

**Why gold?** Strategy documents define the project at its highest level —
what it is, who it's for, and where it's going. They share gold with
Contract because both answer "what is Urd?" — Strategy at the product
level, Contract at the technical level. Gold is the brand colour and the
natural home for documents that define the project's identity.

**The rule:** If a document's primary purpose is to define the product
direction, market positioning, or roadmap, it's Strategy. If it defines
the technical data model, it's Contract.

**Future documents:** Updated roadmaps, market positioning refreshes,
revenue model updates, partnership strategy.

---

### Contract — Gold #dab860
> *The .urd.json data model. What the schema actually is.*

| Document                          | Format               | Date     | Size         |
|-----------------------------------|----------------------|----------|--------------|
| Urd World Schema Specification v0.1 | Technical Specification | Feb 2026 | 14 sections  |
| Nested Dialogue Architecture      | Design Document       | Feb 2026 | Detailed     |

**Why gold?** Gold is the brand color AND the .urd.json contract color in the
architecture diagram. These documents define the data model that everything
else consumes. The Schema Spec is the contract itself. The Nested Dialogue
doc defines how dialogue sections, choices, and exhaustion are represented
in that contract — it's a deep dive into one part of the schema, not a
compiler or runtime document.

**The rule:** If a document's primary purpose is to specify *what data looks
like*, it's Contract/Gold. If it specifies *how data is transformed*, it's
Architecture/Amber. If it specifies *how data is executed*, it's Runtime/Purple.

**Future documents:** Schema changelog, migration guides, entity reference,
property type catalogue, rule grammar formal spec.

---

### Authoring — Blue #6a9acc
> *Writer Mode. How content enters the system.*

| Document                          | Format               | Date     | Size         |
|-----------------------------------|----------------------|----------|--------------|
| Schema Markdown Syntax Specification | Syntax Specification | Feb 2026 | Full spec    |

**Why blue?** Blue is Writer Mode in the architecture diagram. The Schema
Markdown syntax is the writer-facing surface — the format that narrative
designers actually touch. It compiles to the gold contract, but the document
itself is about the authoring experience.

**Future documents:** Authoring guide, template library, frontmatter reference,
import system documentation, writer-facing tutorials.

---

### Architecture — Amber #cc7a3a
> *Compiler + system design. How it's built.*

| Document                          | Format               | Date     | Size         |
|-----------------------------------|----------------------|----------|--------------|
| Architecture & Technical Design   | System Blueprint      | Feb 2026 | 10,000+ words |
| System Architecture Diagram       | Interactive Visualisation | Feb 2026 | Visual       |

**Why amber?** Amber is the Compiler in the architecture diagram, and the
compiler is the organising principle of the system. The architecture doc
defines all four components but is structured around the build order — what
gets built when. The system diagram is its visual counterpart. Both are
"how the system is designed and constructed."

**Future documents:** Compiler implementation notes, build system docs,
incremental compilation strategy, ID generation specification, dependency
graph documentation.

---

### Runtime — Purple #b090dd
> *Wyrd execution engine. How worlds run.*

| Document                          | Format               | Date     | Size         |
|-----------------------------------|----------------------|----------|--------------|
| Wyrd Reference Runtime            | Runtime Specification | Feb 2026 | 8,000+ words |

**Why purple?** Purple is Wyrd in every context — the architecture diagram,
the hero section, the component cards. It even gets the glow animation.
Wyrd is the canonical behaviour oracle: if there's ambiguity about how a
world should behave, what Wyrd does is the answer.

**Future documents:** Wyrd API reference, embedding guide, state management
internals, event system specification, extension host documentation,
lambda function sandboxing.

---

### Validation — Green #70b080
> *Testing framework. Proving it works.*

| Document                          | Format               | Date     | Size         |
|-----------------------------------|----------------------|----------|--------------|
| Test Case Strategy                | Validation Plan       | Feb 2026 | Detailed     |

**Why green?** Green is the Testing tool in the architecture diagram AND
the Engineer Mode colour. Testing is what engineers do to prove the system
works. The test strategy document defines the progression — Monty Hall →
Key Puzzle → Composed World — and why each test exists.

**Future documents:** Test results and coverage reports, Monte Carlo
simulation outputs, reachability analysis reports, regression test suites,
proof point write-ups.

---

## How to Assign New Documents

```
 Question                                          → Category
 ────────────────────────────────────────────────── ──────────────
 Does it study the problem space or market?         → Research (Rose)
 Does it define the product vision or roadmap?      → Strategy (Gold)
 Does it define what the data model looks like?     → Contract (Gold)
 Does it define how writers create content?         → Authoring (Blue)
 Does it define how the system is built or compiles?→ Architecture (Amber)
 Does it define how Wyrd executes worlds?           → Runtime (Purple)
 Does it define how correctness is verified?        → Validation (Green)
```

When a document spans multiple concerns (e.g., Nested Dialogue discusses
both compilation and execution), assign it based on its *primary purpose*:
what is the document's reason for existing? Nested Dialogue exists to
specify the data model for dialogue — how sections, choices, and exhaustion
are *represented*. That's Contract, not Architecture or Runtime.

---

## Visual Reference

```
  ┌─────────────────────────────────────────────────────────┐
  │                    STRATEGY  (Gold)                      │
  │                    Product Vision                        │
  └──────────────────────────┬──────────────────────────────┘
                             │ motivates
  ┌──────────────────────────▼──────────────────────────────┐
  │                    RESEARCH  (Rose)                      │
  │         Landscape Analysis  ·  Pain Points Report        │
  └──────────────────────────┬──────────────────────────────┘
                             │ informs
  ┌──────────────────────────▼──────────────────────────────┐
  │                                                         │
  │   AUTHORING (Blue)          CONTRACT (Gold)             │
  │   Schema Markdown    ───▶   Schema Spec                 │
  │                             Nested Dialogue             │
  │                                    │                    │
  └────────────────────────────────────┼────────────────────┘
                                       │ consumed by
  ┌────────────────────────────────────▼────────────────────┐
  │                                                         │
  │   ARCHITECTURE (Amber)       RUNTIME (Purple)           │
  │   Architecture Doc           Wyrd Reference Runtime     │
  │   System Diagram                                        │
  │                                                         │
  └────────────────────────────────────┬────────────────────┘
                                       │ verified by
  ┌────────────────────────────────────▼────────────────────┐
  │                   VALIDATION (Green)                     │
  │                   Test Case Strategy                     │
  └─────────────────────────────────────────────────────────┘
```

---

## Implementation Notes

### On urd.dev
- Pill badges on document cards use the category color
- Filter buttons in the Deep Dives section correspond to these six categories
- The History timeline uses small colored dots matching the category of each entry
- Accent gradient swatches (design system) show each category's gradient
  with a 4px top-edge highlight demonstrating the accent border usage
- The design system sidebar uses the raised navigation panel pattern
  (see Design Brief §5, Navigation Panel)

### In the architecture diagram
- No changes needed — the diagram's existing colors ARE this system
- Documents referenced from the diagram link back to urd.dev with matching colors

### In the repository
- Document frontmatter includes a `category` field matching these seven values
- The Astro build generates the correct color tokens from this field

### CSS Variables

The taxonomy CSS variables are defined per-theme, not here. Each theme's
design brief and design system provide the concrete values.

Variable naming convention (consistent across all themes):
```css
--doc-{category}:        /* base hue */
--doc-{category}-dim:    /* background tint */
--doc-{category}-light:  /* emphasis / active text */
--doc-{category}-border: /* card and pill borders */
```

Categories: `research`, `strategy`, `contract`, `authoring`,
`architecture`, `runtime`, `validation`.
