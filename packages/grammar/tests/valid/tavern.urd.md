---
world: tavern-talk
start: tavern

types:
  Barkeep [interactable]:
    name: string
    trust: int(0, 100) = 30
    ~knows_secret: bool = true

entities:
  @arina: Barkeep { name: "Arina" }
---

# The Rusty Anchor

A low-ceilinged tavern thick with pipe smoke and the smell of salt.

[@arina]

== topics

@arina: What'll it be, stranger?

+ Ask about the harbor

  @arina: Quiet today. Too quiet, if you ask me.

  > @arina.trust + 5

  -> topics

* Ask about the missing ship

  ? @arina.trust > 50

  @arina leans in close.

  @arina: The Selene didn't sink. She was taken.

  > @arina.trust + 10

  -> topics

  ? @arina.trust <= 50

  @arina: I don't know what you're talking about.

  She turns away and starts wiping the counter.

  -> topics

+ Buy her a drink

  @arina smiles.

  @arina: Well aren't you a gentleman.

  > @arina.trust + 20

  -> topics

* Leave -> harbor

@arina: Suit yourself. I've got glasses to clean.
