---
world: two-room-key
start: cell

types:
  Key [portable]:
    name: string

  LockedDoor [interactable]:
    locked: bool = true
    requires: ref(Key)

  Guard [interactable, mobile, container]:
    name: string
    mood: enum(hostile, neutral, helpful) = hostile
    ~hint_given: bool = false

entities:
  @rusty_key: Key { name: "Rusty Key" }
  @cell_door: LockedDoor { requires: @rusty_key }
  @guard: Guard { name: "Halvard" }
---

# Cell

A dim stone cell. A guard watches from the corner.

[@rusty_key, @guard, @cell_door]

* Wait quietly and show respect -> @guard

  ? @guard.mood == hostile

  > @guard.mood = neutral

* Talk to the guard -> @guard

  ? @guard.mood == neutral

  ? @guard.hint_given == false

  @guard glances at the loose stone in the wall.

  > @guard.hint_given = true

* Pick up the rusty key -> @rusty_key

  ? @rusty_key in here

  > move @rusty_key -> player

* Use the key on the door -> @cell_door

  ? @rusty_key in player

  ? @cell_door.locked == true

  The lock clicks. The door swings open.

  > @cell_door.locked = false

  > destroy @rusty_key

-> north: Corridor

  ? @cell_door.locked == false

  ! The iron door is locked.

# Corridor

A long corridor. Daylight leaks from the far end.

-> south: Cell
