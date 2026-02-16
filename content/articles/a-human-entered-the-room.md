---
title: A Human Entered the Room
slug: a-human-entered-the-room
description: What happens when you show an AI-built project to people who have been building interactive fiction and care about the subject.
date: "2026-02-16"
---

> **Document status: INFORMATIVE, DEVELOPMENT JOURNAL**
> Reflections on showing an AI-built project to people who know the domain. What they said, what it revealed, and what changes.
> Single canonical copy. February 2026.

## What happened

On 15 February 2026, Urd was posted to an online community for people who build, study, and care about interactive fiction. The post introduced the project as a hobby experiment in AI-driven spec development: write a declarative schema for interactive worlds, hand the specification to AI, and see what it can build.

The community responded. Not with polite interest. With expertise.

Within hours, the thread had contributions from people who know this domain deeply. They read the specifications, visited the site, inspected the claims, and told us what they found.

This document records what they said, what it exposed, and what the project is doing about it.

## The feedback

### "Confident bullshit"

The sharpest critique came from an experienced Inform developer. The assessment was blunt: the site read as "a lot of very confident bullshit, using big words and lavish praise to say very little." The specific complaints were precise and, on inspection, correct.

**AI-generated testimonials.** The site's "Peer Review" section presented AI system evaluations of the specification as though they were meaningful endorsements. They were not. Current commercial LLMs will praise anything presented to them. Displaying sycophantic output as validation is worse than displaying nothing. It signals that the project cannot distinguish genuine feedback from flattery. In a community of writers who work with text for a living, this was immediately visible.

**Historical inaccuracy.** The introduction stated that Inform, born in 1993, "gave us the first serious world model for interactive fiction." This is false. TADS predates Inform by five years (1988). ZIL at Infocom goes back further still. To a community that lived through this history, getting the timeline wrong in the opening paragraph is a credibility-destroying error, and it was an error that the AI generated and the human did not catch.

**Broken links.** The architecture section's specification links did not work. For a project about engineering rigour, non-functional navigation is a particularly bad look.

**Overclaiming.** Several passages made confident assertions about what the system achieves, using present tense, for capabilities that do not exist yet. No compiler. No runtime. No playable output. The gap between the tone and the reality was the core problem.

### "Database-driven IF"

The community identified a deeper architectural concern. The declarative approach, describing what the world *is* rather than scripting what it *does*, is not new. It has been tried before, going back to the 1980s, and it has a name in the IF community: database-driven IF.

The problem is well understood: systems that delegate all behaviour to a runtime tend to produce worlds where the standard behaviours work fine but nothing *interesting* happens. The interesting parts of interactive fiction are the parts that are not built in. The custom mechanic. The unexpected verb. The interaction that nobody anticipated.

The challenge was direct: if all behaviour is defined by the Urd runtime, then Urd has the same coupling problem it accuses Inform of having: the world model and the execution engine are still inseparable. You have just moved the dependency, not eliminated it.

This is a legitimate structural critique, and the project's answer, a lambda extension host that lets authors attach custom logic to entities without leaving the schema, is designed but not built. Until it exists and is tested, the critique stands.

### "Show me a game"

Multiple community members made the same point: specifications are not games. The IF community evaluates systems by what they produce, not by what they describe. A comp entry, a demo, a playable five-minute experience: any of these would carry more weight than ten specification documents.

This is the classic tension of developer procrastination: building the engine instead of the game. The project acknowledges this openly. Urd is, in part, a test of whether that procrastination can be *weaponised*: whether AI can be made to do the building so the human can focus on the design. But the community's point remains: until there is something to play, the project is a collection of promises.

### What was acknowledged

The community also acknowledged several things:

- The problem Urd is trying to solve, the fragmentation tax, the glue code between narrative tools and game engines, is real.
- The experimental framing, once explained, was more interesting than the product framing the site presented.
- The question of whether AI-driven development can produce real systems from formal specifications is genuinely worth asking, regardless of whether Urd specifically succeeds.

## What it revealed

### The AI blindspot

The most uncomfortable revelation was not about the schema. It was about the process.

This project's entire pipeline, specifications, documentation, site content, even the peer reviews, was generated through AI conversations. The AI produced historically inaccurate claims and the human published them. The AI generated sycophantic self-reviews and the human displayed them as validation. The AI wrote in a tone of confident product marketing and the human did not notice it was inappropriate for a project that has not built anything yet.

The experiment is supposed to test where AI-driven development fails. The IF community showed us the first failure point: **the human in the loop stopped checking.**

AI is an extraordinary collaborator for exploring ideas, iterating on designs, and producing volume. But it has no sense of audience, no awareness of community norms, and no instinct for when confidence is earned versus performed. When the human defers to AI judgment on those questions, the output is exactly what the forum described: confident bullshit.

This is not an argument against AI-driven development. It is a calibration. The spec can be AI-generated and AI-iterated. The claims about the spec, what it achieves, how it compares to existing work, what it means, require human judgment that is informed by the domain.

### The value of hostile feedback

Every AI conversation about Urd has been constructive. The models explore the design, suggest improvements, identify edge cases, and produce output. What they do not do is say: "This is wrong. You are embarrassing yourself. Stop."

The IF community did that in under an hour. And it was more valuable than weeks of AI iteration, because it tested something no AI conversation can test: *how the project lands with the people it needs to convince.*

This is not something that can be automated. It is not a failure of current models that will be fixed by better models. It is a structural property of the feedback loop. AI operates within the context you give it. Domain communities bring context you do not have.

### The Inform 10 correction

The community pointed out that Inform 10 has made progress on the portability problem Urd claims to address. The assertion that "you cannot hand an Inform world to Unity, to Godot, to a browser" is no longer entirely accurate. The project's documentation has been updated to acknowledge this, but the broader lesson is important: claims about the limitations of existing tools must be current, not based on the AI's training data snapshot.

## What changed

In response to the community feedback, the following changes were made within a few hours:

### Site and presentation

- **Experimental framing.** The site introduction was rewritten to lead with the project's nature as an experiment in AI-driven spec development, not a product announcement.
- **Historical accuracy.** The timeline now correctly credits ZIL and TADS before Inform, presented in chronological order.
- **Inform 10 acknowledgment.** The comparison with Inform now acknowledges Inform 10's progress on portability rather than treating Inform as frozen in time.
- **Honest status.** The "Where We Are" section was rewritten to state plainly: "No compiler exists. No runtime exists. The schema is a design, not a product."
- **The Monty Hall example.** A concrete code example was added to the presentation: the Monty Hall problem in Schema Markdown, showing hidden state, select constraints, and emergent probability. This replaces abstract claims with a demonstrable design.
- **"Where It Breaks" section.** A new section directly names the database-driven IF critique, acknowledges its history, describes the planned lambda extension host, and invites scepticism.
- **Hypothesis framing.** Confident assertions ("The unification has been done. The portability has been done. Never at the same time.") were replaced with hypothesis framing ("That is the hypothesis this project is testing.").

### What did not change

- **The specification itself.** No community member identified a structural flaw in the schema design. The critiques were about framing, accuracy, and the gap between claims and implementation, not about the schema's architecture. The spec stands as designed, pending implementation testing.
- **The experimental thesis.** The core question, can a formal specification drive AI-built implementation, remains the project's purpose. The community feedback sharpened how that question is communicated, not whether it is worth asking.

## What comes next

The community told us what matters: *show something that runs.*

The next milestone is unchanged but now carries the weight of a public commitment:

**Compile the Monty Hall problem to JSON. Run it 10,000 times. Verify the switching advantage converges to 2/3.**

If that works, the declarative thesis holds for at least one non-trivial case. Then the two-room key puzzle. Then dialogue. Each milestone either validates the design or forces it to change, and both outcomes will be published here.

The IF community gave this project something no AI conversation could: a reality check from people who know the domain better than we do. The correct response is not to explain why they are wrong. It is to build something that earns their attention.

A human entered the room, and the room got better.

*This document is part of the Urd development journal at [urd.dev](https://urd.dev).*
