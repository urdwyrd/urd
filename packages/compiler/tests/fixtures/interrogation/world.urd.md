---
types:
  Person [interactable]:
    mood: enum(hostile, neutral, friendly) = neutral
    trust: integer = 0
  Evidence [portable]:
    name: string
entities:
  @suspect: Person
  @detective: Person
  @evidence: Evidence { name: "The Letter" }
---
