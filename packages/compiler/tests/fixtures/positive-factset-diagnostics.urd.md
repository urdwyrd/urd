---
world:
  name: factset-triggers
  start: room

types:
  NPC [interactable]:
    trust: integer = 0
    suspicion: integer = 0
    mood: enum(calm, alert, friendly) = calm
    rank: integer = 0
    loyalty: integer = 0
    power: integer = 0

entities:
  @npc: NPC
---

# Room

[@npc]

== talk

+ Greet
  > @npc.mood = friendly
  > @npc.loyalty = 5

* Ask about suspicion
  ? @npc.suspicion >= 3
  @npc: I'm watching you.

* Check mood (calm only)
  ? @npc.mood == calm
  @npc: All is well.

* Check mood (alert only)
  ? @npc.mood == alert
  @npc: On guard.

* Seek power
  ? @npc.power >= 100
  @npc: You are mighty.

+ Gain some power
  > @npc.power = 5

* Apply for promotion
  ? @npc.rank >= 1
  > @npc.rank + 1
