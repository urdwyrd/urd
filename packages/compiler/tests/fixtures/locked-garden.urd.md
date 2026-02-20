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
  @ghost: Character { role: "The Forgotten", trust: 3 }
  @iron_key: Item { name: "Iron Key" }
  @journal: Item { name: "Warden's Journal" }
  @garden_gate: Lock
---

# Gatehouse

A stone archway choked with ivy. Lantern light flickers.

[@warden, @iron_key]

-> garden: The Walled Garden
  ? @garden_gate.locked == false
  ! The gate is sealed with old iron.

== greet

@warden: Nobody passes without reason.

+ State your purpose
  @warden: Hmm. Lots of people say that.
  > @warden.trust + 1
  -> greet

* Offer the journal
  ? @journal in player
  @warden: Where did you find this? This changes things.
  > @warden.trust + 5
  > @warden.mood = friendly

* Ask about the garden
  ? @warden.trust >= 3
  @warden: The garden remembers what we buried there.
  > @garden_gate.locked = false

* Force the gate -> @garden_gate
  ? @warden.mood != friendly
  @warden: I wouldn't try that.
  > @warden.trust - 2

# The Walled Garden

Overgrown paths wind between crumbling statues.

[@ghost, @journal]

-> north: Gatehouse

== explore

@ghost: You shouldn't have come here.

? any:
  @ghost.trust >= 5
  @journal in player

* Listen to the ghost
  @ghost: They locked this place to forget. But I remember everything.
  > @ghost.trust + 2

  * Press for the truth
    ? @ghost.trust >= 5
    @ghost: The key opens more than gates. Take it to the fountain.
    -> revelation

* Take the journal
  ? @journal in here
  @ghost: That belongs to the living. Perhaps it still matters.

* Leave quietly
  @ghost: The garden never forgets.

== revelation

@ghost: Now you know.

> destroy @iron_key
