---
title: "JSON Schema for compiler output"
date: "2026-02-16"
link: "/articles/json-schema"
---

The output-side formalisation artifact is complete: a JSON Schema (draft 2020-12) that validates compiled `.urd.json` files structurally. Covers all eight top-level blocks, nine reusable sub-schemas, conditional type validation, and recursive dialogue choices. Tested against 32 fixtures — 7 positive, 25 negative — all passing.
