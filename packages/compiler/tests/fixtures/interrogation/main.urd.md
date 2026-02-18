---
world:
  name: Interrogation
  start: interrogation-room
import: ./world.urd.md
---

# Interrogation Room

[@suspect, @detective, @evidence]

-> lobby: Lobby

== approach

@detective: We know you were there.

? any:
  @suspect.trust >= 3
  @suspect.mood == friendly

* Press harder
  ? @suspect.mood != hostile
  @detective: Tell me what you know.
  > @suspect.trust - 1

* Show evidence
  ? @evidence in player
  @detective: Explain this.
  > @suspect.trust + 2

  * Push further
    ? @suspect.trust >= 2
    @suspect: Alright, alright...
    -> confession

== confession

@suspect: It was the butler.

# Lobby

The lobby is quiet.

-> south: Interrogation Room
