---
world:
  name: monty-hall
  start: stage
types:
  Door [interactable]:
    ~prize: enum(goat, car)
    revealed: bool = false
entities:
  @door_1: Door { prize: "goat" }
  @door_2: Door { prize: "goat" }
  @door_3: Door { prize: "car" }
  @host: Door
---

# Stage

[@door_1, @door_2, @door_3]

## The Game

### Choose

* Pick a door -> any Door

### Reveal (auto)

rule monty_reveals:
  actor: @host action reveal
  selects door from [@door_1, @door_2, @door_3]
    where door.prize == goat
  > reveal door.prize

### Switch

== switch

* Switch doors -> any Door
  ? @door_1.revealed == false
* Stay with your choice

The host opens the final door.
