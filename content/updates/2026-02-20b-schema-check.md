---
title: "Schema Check — closing the validation loop"
date: "2026-02-20"
link: "/articles/schema-check"
---

Compiler output now validates against the JSON Schema at three levels: CI test suite (7 compiler fixtures), browser playground (lazy-loaded AJV with toggle and traffic light), and the emit phase itself (world names auto-slugified). Compiler 0.1.4. Schema test corpus at 39 fixtures.
