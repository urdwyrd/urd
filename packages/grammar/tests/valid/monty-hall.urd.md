---
world: monty-hall
start: stage
entry: game

types:
  Door [interactable]:
    ~prize: enum(goat, car)
    state: enum(closed, open) = closed
    chosen: bool = false

  Host:
    name: string

entities:
  @door_1: Door { prize: car }
  @door_2: Door { prize: goat }
  @door_3: Door { prize: goat }
  @monty: Host { name: "Monty Hall" }
---

# Stage

A game show stage with three closed doors.

[@door_1, @door_2, @door_3, @monty]

## Game

### Choose

Pick a door.

* Pick a door -> any Door

  ? target.state == closed

  ? target.chosen == false

  > target.chosen = true

### Reveal (auto)

@monty opens a door that hides a goat.

rule monty_reveals:
  @monty selects target from [@door_1, @door_2, @door_3]
  where target.prize != car
  where target.chosen == false
  where target.state == closed
  > target.state = open

### Switch or Stay

Monty opened a door with a goat. Switch or stay?

* Switch to the other closed door -> any Door

  ? target.state == closed

  ? target.chosen == false

  > target.chosen = true

* Stay with your current choice

### Resolve (auto)

> reveal @door_1.prize

> reveal @door_2.prize

> reveal @door_3.prize
