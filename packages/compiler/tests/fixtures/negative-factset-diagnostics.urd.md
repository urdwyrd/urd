---
world:
  name: factset-clean
  start: room

types:
  NPC [interactable]:
    trust: integer = 0
    mood: enum(calm, alert) = calm
    seen: bool = false

entities:
  @npc: NPC
---

# Room

[@npc]

== talk

+ Greet
  > @npc.trust + 1
  > @npc.mood = alert
  > @npc.seen = true

* Ask for help
  ? @npc.trust >= 3
  @npc: Sure.

* Check mood
  ? @npc.mood == alert
  @npc: I'm on edge.

* Check if seen
  ? @npc.seen == true
  @npc: We've met before.
