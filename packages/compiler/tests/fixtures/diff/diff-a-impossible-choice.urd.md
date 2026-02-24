---
world:
  name: impossible-choice-test
  start: hall

types:
  Door [interactable]:
    state: enum(locked, unlocked, open) = locked

entities:
  @heavy_door: Door
---

# Hall

A grand entrance hall.

[@heavy_door]

== enter

A heavy door stands before you.

* Try the door
  ? @heavy_door.state == unlocked
  The door swings open.

* Knock loudly
  Your knocks echo through the hall.
