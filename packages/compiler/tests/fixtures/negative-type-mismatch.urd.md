---
world:
  name: Broken
  start: room
types:
  Lock:
    locked: bool = true
entities:
  @lock: Lock
---
# Room

[@lock]

== actions

* Toggle
  > @lock.locked = "not a boolean"
