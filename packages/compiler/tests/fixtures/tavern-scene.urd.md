---
world:
  name: the-rusty-anchor
  start: the-rusty-anchor
types:
  Character [interactable]:
    mood: enum(hostile, neutral, friendly) = neutral
    trust: integer = 0
entities:
  @arina: Character { mood: "friendly" }
---
# The Rusty Anchor

[@arina]

-> harbor: The Harbor

== topics

@arina: What'll it be?

* Ask about the ship
  @arina: She's seen better days.
  > @arina.trust + 1
+ Order a drink
  @arina: Coming right up.

@arina leans back and sighs.

# The Harbor

The harbor stretches out before you.

-> south: The Rusty Anchor
