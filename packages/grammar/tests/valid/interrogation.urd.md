---
import: ./world.urd.md

types:
  Guard [interactable, mobile, container]:
    name: string
    mood: enum(hostile, suspicious, neutral, nervous) = hostile

entities:
  @halvard: Guard { name: "Halvard" }
  @coin_purse: Item { name: "Coin Purse" }
---

== interrogation

@halvard: You've got questions. Make them quick.

* Ask about the prisoner

  @halvard: What prisoner?

  @halvard stares at you, unblinking.

  * Press him

    ? @halvard.mood == neutral

    @halvard sighs.

    @halvard: Cell three. But you didn't hear it from me.

    > @halvard.mood = nervous

    > player.knows_cell = true

    -> interrogation

    ? @halvard.mood == hostile

    @halvard: I said, what prisoner?

    -> interrogation

  * Back off -> interrogation

* Ask about the warden

  ? player.knows_cell == true

  @halvard: Keep the warden out of this. You got what you wanted.

  > @halvard.mood = hostile

  -> interrogation

  ? player.knows_cell == false

  @halvard: The warden runs a clean operation. End of story.

  -> interrogation

* Try to bribe him -> bribe

* Ask about the escape route

  ? any:
    @halvard.mood == hostile
    @halvard.mood == suspicious

  @halvard: I don't talk to your kind.

  -> interrogation

  ? @halvard.mood == neutral

  @halvard: There's a passage behind the chapel.

  > player.knows_escape = true

  -> interrogation

* I'm done here -> farewell

== bribe

? @coin_purse in player

You slide the coin purse across the table.

  ? @halvard.mood == hostile

  @halvard pushes it back.

  @halvard: Not enough to buy what you're asking.

  -> interrogation

  ? @halvard.mood != hostile

  @halvard pockets it without looking down.

  @halvard: What do you want to know?

  > @halvard.mood = neutral

  > move @coin_purse -> @halvard

  -> interrogation

? @coin_purse not in player

@halvard: Bribe with what? You've got nothing.

-> interrogation

== farewell

? @halvard.mood == nervous

@halvard: Watch yourself out there.

He won't meet your eyes.

? @halvard.mood == hostile

@halvard says nothing. The door slams behind you.

? @halvard.mood == neutral

@halvard: Don't come back.

He says it without conviction.
