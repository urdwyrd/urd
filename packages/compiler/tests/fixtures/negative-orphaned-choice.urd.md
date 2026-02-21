---
world:
  name: orphaned-choice-test
  start: room
types:
  Door [interactable]:
    state: enum(closed, open) = closed
entities:
  @door: Door
---
# Room

[@door]

== actions

* Try the lock
  ? @door.state == locked

The room is quiet.
