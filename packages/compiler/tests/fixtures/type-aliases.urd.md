---
world:
  name: alias-test
  start: room
types:
  Character [interactable]:
    trust: int(0, 100) = 30
    active: bool = true
    weight: num(0.0, 10.0) = 5.5
    label: str = "default"
    mood: enum(happy, sad) = happy
entities:
  @hero: Character
---
# Room

A test room.

[@hero]

== greet

@hero: Hello.
