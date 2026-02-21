---
world:
  name: key-puzzle
  start: cell
types:
  Key [portable]:
    name: string
  Door [interactable]:
    locked: bool = true
entities:
  @rusty_key: Key { name: "Rusty Key" }
  @cell_door: Door
---
# Cell

A dim stone cell.

[@rusty_key, @cell_door]

-> north: Corridor
  ? @cell_door.locked == false
  ! The iron door is locked.

== actions

* Use key -> @cell_door
  ? @rusty_key in here
  > @cell_door.locked = false
  > destroy @rusty_key

The cell falls silent.

# Corridor

You made it out.
