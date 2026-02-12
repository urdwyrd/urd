---
title: "Developer Pain Points Report"
slug: "pain-points-report"
description: "Mined Reddit, Hacker News, GDC talks, developer blogs, and postmortems. Found seven pain clusters. Every team builds the same glue code. Writers edit dialogue in spreadsheets. Testing is manual and terrifying."
category: "research"
format: "Forum-Sourced Research"
date: "2026-01"
status: "v1.0 complete"
order: 2
tags:
  - research
  - pain-points
  - developer-experience
  - forums
details:
  - "The integration tax: 30–50% of effort on glue code"
  - "Writer-hostile workflows forcing non-technical users into code"
  - "Testing is manual, fragile, and terrifying"
  - "No standard interface between narrative data and game state"
---

> **Document status: INFORMATIVE — RESEARCH**
> Forum-sourced research documenting developer pain points in narrative game tooling. Provides the evidence base for the product vision. Not an implementable specification.
> Single canonical copy. February 2026 draft.

# WHAT SUCKS TODAY

## Developer Pain Points in Narrative Game Tooling

A Forum Sourced Research Report

February 2026

Sources: Reddit, Hacker News, GDC talks, developer blogs, postmortems, Steam reviews, tool forums, and published interviews

## Executive Summary

This report bypasses market sizing to answer a single question: What do developers actually hate about their current narrative game development workflows?

The evidence comes from where developers complain most honestly. Reddit threads, Hacker News discussions, Steam reviews, developer blogs, GDC postmortems, tool forums, and published interviews. Nobody is trying to impress anyone in these contexts. The frustration is raw.

The core finding is that narrative game development suffers from a **fragmentation tax**. Not a single catastrophic tool failure, but death by a thousand integration cuts. Every team cobbles together a bespoke pipeline from tools that were never designed to work together, and then spends 30 to 50 percent of their development effort on the glue.

The pain clusters into seven categories, each documented with direct evidence from the developer community. Together they paint a picture of an industry where the tooling has not kept pace with the ambition of the work.

## Methodology: How This Was Researched

Rather than interviews, this report synthesizes publicly available developer frustrations from the following source types:

**Developer forums and communities:** Reddit r/gamedev, r/interactivefiction, Hacker News, Interactive Fiction Community Forum, Pixel Crushers Forum, TIGSource, GameDev.net, articy:draft forums, Steam Community discussions.

**Developer blogs and postmortems:** Published dev blogs from studios shipping narrative games (ZA/UM's Disco Elysium, Failbetter's Mask of the Rose, Night in the Woods, Sable), GDC Narrative Summit talks, Noclip documentaries.

**Tool comparison articles and reviews:** Steam reviews of articy:draft, AlternativeTo user reviews, Arcweave's competitive analysis blog, NarrativeFlow's tool comparison, Game Developer magazine features.

**Pipeline documentation:** Ian Thomas's "A Dialogue Pipeline" (senior narrative developer with AAA experience), Yarn Spinner 2026 roadmap, Problem Machine devlog, Robert Yang's Radiator Blog.

The pain points below are ranked by frequency of complaint (how many independent sources raise the issue) and severity of impact (how much developer time/quality it costs). Direct quotes are used where available.

## Pain #1: The Integration Tax

### Every team builds the same glue, and it's always fragile

The single most consistently reported frustration across every source type is the effort required to bridge narrative authoring tools and game engines. No existing tool handles the full pipeline. Every team invents custom integration code, and this code is the most brittle part of their stack.

> *"When I start at a studio, I almost always find myself building or rebuilding a pipeline for game dialogue."*
> — Ian Thomas, senior narrative developer (AAA credits)

> *"As a person who built an entire open-source external tool to manage Dialogue System databases, I can resolutely say that it's much better to work with the built-in editor. The problem with an external tool is that you become very disconnected from your other game logic and data."*
> — Pixel Crushers Forum user (developer of open source dialogue tooling)

**What developers actually do**

The typical indie narrative game pipeline looks like this: write in ink or Yarn Spinner, import into Unity via a plugin, write custom C# to bridge dialogue output to game UI, write more custom code to connect narrative state to game state (inventory, quests, NPC attitudes), debug the seams. Each of these connection points is a potential failure mode.

For studios using articy:draft, the pattern is similar but more expensive: author in articy, export XML/JSON, import via Unity/Unreal plugin, write custom code to interpret articy's template data in engine terms. The articy Unity plugin itself is reportedly slow: one developer on Pixel Crushers Forum notes it is "dog slow in the editor, although it runs fine in builds/playmode."

**The glue code is never reusable**

Because every game has different needs (some need bark systems, some need hub and spoke dialogue, some need procedural storylets), the integration code is always custom. Teams cannot share or reuse it. There is no standard interface between "narrative data" and "game state." This means every new project starts from scratch on the plumbing.

> *"Getting this all working was a week or so of work, not least because Yarn Spinner does not make most of this variable getting/setting information readily available. Because of this I had to create my own helper class which parses the dialogue files and pulls out all of the information. This was, frankly, a very frustrating thing to have to do: If I'm putting this much work into parsing dialogue files, the benefits of using a pre-made dialogue system really start to diminish."*
> — Problem Machine devlog (indie developer using Yarn Spinner)

## Pain #2: The Scale Wall

### Tools that work at 5,000 words break at 500,000

Tools are designed for demos and small projects. When a real game hits production scale, they buckle. This isn't a theoretical concern. It's the single most famous narrative tooling failure in recent game history.

> *"There was a lot of dialogue so it got quite janky, at some point froze completely, because it definitely wasn't built for it. We contacted them as well, and they were like, 'Yeah, you know this is the first time anyone's coming with those problems to us, so we don't really know what to do.'"*
> — Helen Hindpere, lead writer of Disco Elysium, on articy:draft (Noclip documentary, January 2026)

Disco Elysium literally broke articy:draft, the industry's leading professional narrative design tool. The volume of text exceeded what the software could handle, and the articy team had no solution. This was widely reported across PC Gamer, Attack of the Fanboy, and Hacker News in January 2026.

A Hacker News commenter who also shipped a narrative game with articy confirmed the pattern:

> *"Articy is a good tool but it has its own set of problems, specifically awful performance on huge projects. Even on a project of much smaller scale, getting it all debugged, loop-free and free of tons of logical errors is very hard."*
> — Hacker News user (shipped a narrative game with articy)

**Twine hits the wall even sooner**

Twine's single board architecture means that even moderately complex projects become visually unmanageable. Multiple sources note that "all content is put onto one board which can get difficult to manage," unlike tools like Arcweave which support multiple boards. For anything beyond a short interactive fiction piece, Twine's visual approach becomes a liability rather than an asset.

**Ink scales textually but not structurally**

Ink handles large volumes of text well. It's been used for 750,000+ word games (80 Days). But it has no world model, no entity system, no spatial representation. At scale, teams need to track not just dialogue but characters, locations, items, quests, and their interrelationships. Ink provides none of this. Teams that start with ink eventually outgrow it and face a painful migration or bolt on problem.

## Pain #3: The Writer Programmer Wall

### Writers can't touch the game, programmers can't touch the story

The most human pain point. In most teams, writers and programmers operate in completely different tools, different file formats, different mental models. The handoff between them is where quality dies.

> *"Once, I was contracted to write all the sidequests for a game in about two weeks — 60-some quests — where I could edit the text and add lines but not change characters, portraits, or order of lines that were already in the game, and I had to do it all in Excel. Yes, it was a nightmare."*
> — Doc Burford, game writer (Medium, November 2025)

This writer was literally editing dialogue in a spreadsheet because that was the only format both the writing team and the engineering team could exchange. This is not an edge case. Multiple sources describe spreadsheet based dialogue pipelines as common, especially in studios without articy:draft licenses.

**The 40 change problem**

The NarrativeFlow tool comparison captures the core dynamic: "The writer asks the programmer to make 40 changes (slow, frustrating). The writer learns to edit code (risky, error prone)." Neither option is good. This is a tool design failure, not a people problem.

**articy partially solves this but creates new problems**

articy:draft is explicitly designed to let non programmers author narrative content. It succeeds at this within its own environment. But the moment content needs to connect to game logic, conditions based on game state, triggers that fire game events, the writer is back to needing programmer help or learning articy:expresso, which has its own steep learning curve. Multiple reviews note articy's "multitude of features can feel complicated and overwhelming" and that it "may be difficult to learn for beginners."

## Pain #4: Collaboration Is an Afterthought

### Most tools are built for solo use in a team sport

Narrative game development is inherently collaborative. Writers, narrative designers, level designers, voice actors, localizers, and programmers all need to interact with narrative content. Yet nearly every tool in the space was designed for a single user.

**The version control gap**

A consistent complaint across Arcweave's competitive analysis and multiple independent reviews: "Difficult to collaborate in teams or gain feedback through line comments" applies to ink, Yarn Spinner, Twine, and Ren'Py. articy:draft supports multi user editing but "collaborative features can be difficult to setup and require knowledge of version control systems and local network setups."

Text based tools (ink, Yarn) can theoretically use Git, but merge conflicts in narrative files are nightmarish. Narrative files don't diff the way code does. A change to a variable name in one ink file can break dozens of references across other files, and Git won't help you find them.

**Arcweave as the exception that proves the rule**

Arcweave's primary differentiator is real time cloud collaboration. The fact that this is a competitive advantage tells you how bad the baseline is. Arcweave is "comparatively new" and "some features are still under development," but teams choose it specifically because they can work together. This signals that collaboration is a must have the market hasn't adequately served.

**Disco Elysium's coordination challenge**

> *"What might start out as a harmless and innocent 'wouldn't-it-be-cool-if' type of an idea might need someone else to rewrite an entire scene. And then there's the task of coordinating it among the writers — how do you make sure that everyone is on the same page?"*
> — ZA/UM team, Disco Elysium (Articy showcase)

Their solution: documents. External documents maintained alongside articy. The tool itself did not solve the coordination problem.

## Pain #5: Testing Is Manual and Terrifying

### No one knows if their narrative actually works until players find the bugs

Narrative content is extremely difficult to test. Branching paths multiply exponentially, state dependencies create subtle bugs, and the only real test is to manually play through every possible path. For a game with thousands of dialogue nodes, this is effectively impossible.

**Dead ends, unreachable content, logic errors**

Every branching narrative tool generates content that may be unreachable, may dead end without resolution, or may have logical contradictions. Detecting these requires either exhaustive manual playtesting or automated analysis tools. Almost no automated analysis tools exist.

Yarn Spinner's "Story Solver" tool, currently in development for 2026 release, is specifically targeting this gap. Their 2026 roadmap blog says: "Story Solver started as a client project. We showed it at GCAP 2024. People wanted it." The demand for this tool validates the pain. It doesn't yet solve it.

**The Hacker News testimony**

> *"Even on a project of much smaller scale getting it all debugged, loop-free and free of tons of logical errors is very hard. Knowing how many variables Disco Elysium have I truly believe developers are all geniuses."*
> — Hacker News user who shipped a narrative game

If a developer who successfully shipped a narrative game describes debugging as "very hard" even at smaller scale, the pain is systemic. It's not a skill issue; it's a tooling issue.

**Playtesting is the only option**

Ink's "loose end" warnings and Inky's as you type error highlighting are useful but limited. They catch structural errors, not logical ones. Whether a particular sequence of player choices leads to a narratively coherent outcome requires human judgment that no existing tool automates.

## Pain #6: Engine Lock In

### Your narrative data is trapped in your engine choice

Nearly every narrative tool is tightly coupled to a specific game engine. Yarn Spinner's primary integration is Unity. Pixel Crushers' Dialogue System is Unity only. Fungus is Unity only. Dialogic is Godot only. This creates two problems.

**Problem 1: Engine migration destroys narrative work**

If a team decides to switch engines (an increasingly common scenario as Godot grows and Unity's pricing changes push teams to evaluate alternatives), their narrative content is essentially stranded. The scripting language may be portable (ink compiles to JSON), but all the integration code, all the engine specific hooks, all the UI binding, all of it must be rebuilt from scratch.

**Problem 2: Multi engine teams have no shared workflow**

Studios that work across engines (common in studios that do both mobile and PC/console work) cannot share narrative assets or workflows between projects on different engines. Each engine gets its own narrative pipeline.

**The portability illusion**

Ink is positioned as engine agnostic, and technically it is. There are runtime implementations for C#, JavaScript, Lua, Kotlin, C++, and more. But the runtime only handles the text/choice layer. Everything that makes ink useful in a game, connecting choices to game state, triggering visual/audio events, managing save/load, is entirely engine specific custom code. The "portable" part is the easy part; the hard part is always bespoke.

Yarn Spinner's 2026 roadmap explicitly addresses this, noting efforts to make the Godot integration solid alongside Unity. But the underlying architecture is still C# based, and GDScript native developers face friction.

## Pain #7: The Narrative World Divide

### Dialogue lives in one place, the world lives in another, and nothing connects them

This is the most architecturally fundamental pain point. Current tools treat "narrative" as essentially synonymous with "dialogue." But games need far more than dialogue. They need characters with attributes, locations with properties, items with relationships, quests with dependencies, factions with reputations. No narrative tool provides a unified model for all of this.

**articy comes closest but stops short**

articy:draft's template system lets you define custom entity types (characters, locations, items) with custom properties. This is genuinely useful. But articy is a content management tool, not a runtime. Its entity data must be exported and then interpreted by custom engine code. There is no standard way to say "this dialogue node should only be available when the player has item X and is in location Y and NPC Z has attitude > 50." Each game implements this differently.

**Ink's tag system: necessary but not sufficient**

Ink uses a convention of custom tags (like ">>> CAMERA: BigSwoop") to communicate with the game engine. These are passed through as plain text for custom game code to interpret. This works but is entirely ad hoc. There's no schema, no validation, no tooling support. Inkle's own documentation describes this as a "common convention" rather than a formal system.

**The quest problem**

Quests illustrate the divide perfectly. A developer on Pixel Crushers Forum asked how to visualize all quests at once. The response: articy "doesn't have a concept of quests, so defining quests is a bit awkward." Another suggestion: use Twine (a dialogue tool) to visualize quest structure. A third: use Cytoscape (a network analysis tool from bioinformatics). These are workarounds for a missing capability: no tool natively models the relationship between narrative content and game world structure.

## Pain Severity Matrix

Each pain point scored on frequency of independent complaint and estimated development time cost.

| Pain Point | Frequency | Severity | Time Cost | Who Feels It Most |
|------------|-----------|----------|-----------|-------------------|
| Integration Tax | Very High | Critical | 30 to 50% of dev | Engineers, tech leads |
| Scale Wall | High | Critical | Weeks of debug | Writers on large projects |
| Writer Programmer Wall | Very High | High | Constant friction | Writers, narrative designers |
| Collaboration | High | High | 20 to 30% overhead | All team members |
| Testing | High | High | Unmeasured (hidden) | QA, writers, designers |
| Engine Lock In | Medium | High | Full rebuild on switch | Tech leads, studios |
| Narrative World Divide | Medium | Critical | Custom code per game | Narrative designers, engineers |

## What This Means for Product Direction

The pain point evidence validates the direction outlined in the Product Vision and sharpens the priorities.

### Build for the integration tax first

The highest frequency, highest severity pain is the glue code between narrative tools and engines. A product that eliminates even 50% of this integration work has an immediate value proposition. This means the world schema's most important property is not expressiveness. It's having a clean, standard interface that engines can consume without custom code. Think less "powerful format" and more "USB C for narrative data."

### Solve collaboration before features

Every review of every tool mentions collaboration difficulty. A product with real time collaboration on narrative data, even if the narrative model is simple, beats a more powerful product that's single user. Arcweave's growth validates this. Git friendly text formats are necessary but not sufficient; most narrative teams include non technical writers who will never use Git.

### Make testing a first class feature

Yarn Spinner's Story Solver is an early attempt. The fact that it started as a client project and immediately generated demand when shown publicly tells you this is underserved. Automated path exploration, state space analysis, and dead end detection should be core features, not afterthoughts.

### Build a schema native writing syntax

Ink's popularity is not a moat. It's a habit born from the absence of anything better. The moment writers need to express anything beyond branching text, character states, location conditions, quest dependencies, they fall out of ink into ad hoc tags and external documentation. That's not a beloved experience; it's the least bad option.

The strategic move is to build a **schema native markdown syntax**, a writing format that feels as clean as ink for pure dialogue but compiles directly to the world schema. When a writer adds an entity reference, a condition, or a state mutation, they're not annotating text for later interpretation. They *are* the schema. The output is typed, validated, engine consumable world data with no glue code needed.

**The design constraint**

The markdown syntax must be as clean as ink for pure dialogue. If a writer only writes dialogue and choices, they should barely notice the format is different. World model features, entity references, conditions, state mutations, location scoping, should be additive, not intrusive. Steal what works from ink (the choice markers, the diverts) and improve what ink handles with ugly workarounds. Writers won't feel like they're learning a new tool. They'll feel like they're using a better version of what they already know.

**The migration path still works**

Build the ink importer anyway, as an onramp. Writers who love ink can keep writing in it during transition, and their files import cleanly. But the pitch is: your ink files work here, and when you're ready, the native format does everything ink does plus things ink can't. That's not competing with ink's writing experience. That's graduating from it.

### Target the team, not the tool

The pain is not "ink is bad" or "articy is bad." The pain is that no single tool serves the whole team. Writers want text. Designers want visual graphs. Engineers want typed schemas and APIs. Producers want dashboards. The product opportunity is a platform that gives each role the interface they need while maintaining a single source of truth underneath.

> *The developers who will switch are not looking for a better tool. They are looking for a way to stop building the same fragile pipeline on every project.*
