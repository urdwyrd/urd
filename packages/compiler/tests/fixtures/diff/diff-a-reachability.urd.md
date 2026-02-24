---
world:
  name: reachability-test
  start: village

types:
  Gate [interactable]:
    open: bool = true
---

# Village

The village centre.

-> east: Forest
  ! A path leads into the forest.

== talk

+ Chat with locals
  @narrator: The villagers wave.

# Forest

A dense forest clearing.

-> west: Village
  ! The path leads back.

== wander

+ Explore deeper
  @narrator: The trees grow thicker.
