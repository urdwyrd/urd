---
world:
  name: the-locked-garden
  start: gatehouse

types:
  Character [interactable]:
    mood: enum(wary, neutral, friendly) = wary
    trust: integer = 0
    ~role: string
  Item [portable]:
    name: string
    used: bool = false
  Lock [interactable]:
    locked: bool = true

entities:
  @warden: Character { role: "Gatekeeper", mood: "neutral" }
  @iron_key: Item { name: "Iron Key" }
  @garden_gate: Lock
  @lantern: Item { name: "Lantern" }
---

# Gatehouse

A stone archway choked with ivy.

[@warden, @iron_key, @lantern]

-> garden: The Walled Garden
  ? @garden_gate.locked == false
  ? @warden.mood == friendly
  ! The gate is sealed with old iron.

== greet

@warden: Nobody passes without reason.

+ State your purpose
  @warden: Hmm. Lots of people say that.
  > @warden.trust + 1
  -> greet

* Ask about the garden
  ? @warden.trust >= 3
  @warden: The garden remembers what we buried there.
  > @garden_gate.locked = false
  > @warden.mood = friendly

# The Walled Garden

Overgrown paths wind between crumbling statues.

[@garden_gate]

-> north: Gatehouse

== explore

@warden: You shouldn't have come here.

* Leave quietly
  @warden: The garden never forgets.
