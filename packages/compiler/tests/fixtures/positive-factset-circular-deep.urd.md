---
world:
  name: factset-circular
  start: room

types:
  NPC [interactable]:
    clearance: integer = 0

entities:
  @npc: NPC
---

# Room

[@npc]

== lobby

* Request level 1 access
  ? @npc.clearance >= 1
  > @npc.clearance = 2

* Request level 2 access
  ? @npc.clearance >= 2
  > @npc.clearance = 3
