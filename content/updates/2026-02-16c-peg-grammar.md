---
title: "PEG grammar for compiler input"
date: "2026-02-16"
link: "/articles/peg-grammar"
---

The input-side formalisation artifact is complete: a PEG grammar (Rust/pest) that validates `.urd.md` source files syntactically. Covers the full Schema Markdown syntax — headings, choices, conditions, effects, rule blocks, and five resolved ambiguities. Tested against 12 fixtures — 7 valid, 5 invalid — all passing.
