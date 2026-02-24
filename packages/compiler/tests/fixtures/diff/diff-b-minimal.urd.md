---
world:
  name: minimal-world
  start: plaza

types:
  NPC [interactable]:
    greeting: string = "Hello"

entities:
  @guide: NPC { greeting: "Welcome, traveller" }
---

# Plaza

A simple town square.

[@guide]

-> east: Market
  ! The path to the market is clear.

== welcome

+ Say hello
  Welcome to the plaza.

+ Look around
  There's not much here.

# Market

A bustling market square.

== browse

+ Browse the stalls
  Colourful wares line the stalls.
