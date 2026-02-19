---
world:
  name: the-sunken-citadel
  version: "1.0"
  start: village-square
  entry: main-quest
  seed: 7919
  description: "A sprawling adventure through a cursed coastal village and the sunken citadel beneath it. Designed to stress-test every v1 Urd schema feature."
  author: "Urd Compiler Stress Test"

types:
  # ── Characters ──────────────────────────────────────
  Villager [interactable]:
    name: string
    trust: int(0, 100) = 0
    mood: enum(hostile, suspicious, neutral, friendly, terrified) = neutral
    ~secret: string = "none"                 # hidden visibility
    alive: bool = true
    dialogue_count: integer = 0

  Guard [interactable, mobile, container]:   # mobile + container + interactable
    name: string
    alertness: enum(asleep, drowsy, alert, panicked) = alert
    loyalty: int(0, 100) = 50
    ~orders: string = "patrol"               # hidden
    bribed: bool = false
    carrying_torch: bool = true

  Scholar [interactable, container]:         # container (carries scrolls) but not mobile
    name: string
    knowledge: int(0, 100) = 60
    trust: int(0, 100) = 10
    ~knows_ritual: bool = true               # hidden
    ~real_identity: string = "cultist"        # hidden
    mood: enum(calm, nervous, panicked) = calm
    health: number = 100.0                   # float type
    alive: bool = true

  Merchant [interactable, container]:
    name: string
    mood: enum(greedy, neutral, generous) = greedy
    gold_reserve: int(0, 500) = 200
    discount_given: bool = false
    trades_completed: integer = 0

  Spirit [interactable, mobile]:             # mobile but NOT container
    name: string
    form: enum(invisible, translucent, corporeal) = invisible
    hostility: int(0, 100) = 75
    bound: bool = true
    ~true_name: string = "unknown"           # hidden

  # ── Items ───────────────────────────────────────────
  Key [portable]:
    name: string
    material: enum(iron, bronze, bone, crystal) = iron
    used: bool = false

  Weapon [portable, interactable]:
    name: string
    damage: int(1, 50) = 5
    enchanted: bool = false
    durability: num(0.0, 100.0) = 100.0      # float with range

  Scroll [portable]:
    name: string
    ~contents: string = "illegible"          # hidden text
    translated: bool = false
    language: enum(common, ancient, cipher, runic) = common

  Potion [portable]:
    name: string
    effect: enum(healing, invisibility, strength, truth) = healing
    doses: int(0, 3) = 1

  StorageChest [portable, container, interactable]:  # portable + container
    name: string
    locked: bool = false
    requires: ref(Key)                       # ref type

  Treasure [portable]:
    name: string
    value: int(0, 1000) = 0
    cursed: bool = false
    weight: num(0.1, 50.0) = 1.0            # float

  Evidence [portable]:
    name: string
    tags: list = []                          # list type
    examined: bool = false

  # ── Structures ──────────────────────────────────────
  Door [interactable]:
    locked: bool = true
    requires: ref(Key)                       # ref type
    material: enum(wood, iron, stone, crystal) = wood
    broken: bool = false

  Mechanism [interactable]:
    name: string
    state: enum(inactive, primed, activated, broken) = inactive
    required_items: list = []                # list type
    charge: int(0, 10) = 0

  Inscription [interactable]:
    surface_text: string = "Worn stone."
    ~hidden_message: string = "nothing"      # hidden
    deciphered: bool = false

  # ── Mirror with hidden message ─────────────────────
  MagicMirror [interactable]:
    surface_description: string = "A tarnished mirror."
    ~true_reflection: string = "The mirror shows a hidden passage behind the bookcase."

entities:
  # ── NPCs ────────────────────────────────────────────
  @elder_maren: Villager { name: "Elder Maren", trust: 20, mood: "friendly", secret: "knows the citadel entrance" }
  @fisherman_col: Villager { name: "Col the Fisherman", mood: "terrified", secret: "saw the lights beneath the waves", trust: 0 }
  @innkeeper_bessa: Villager { name: "Bessa", trust: 10, mood: "neutral", secret: "hides smuggled goods" }
  @child_pip: Villager { name: "Pip", mood: "friendly", trust: 40, secret: "none" }

  @captain_rhys: Guard { name: "Captain Rhys", alertness: "alert", loyalty: 80, orders: "guard the cliffs", carrying_torch: true }
  @gate_guard: Guard { name: "Torben", alertness: "drowsy", loyalty: 30, orders: "watch the gate", carrying_torch: false }

  @scholar_voss: Scholar { name: "Voss", knowledge: 85, trust: 5, knows_ritual: true, real_identity: "cultist", mood: "calm", health: 100.0 }

  @merchant_dara: Merchant { name: "Dara", mood: "greedy", gold_reserve: 300 }

  @spirit_lament: Spirit { name: "The Lament", form: "invisible", hostility: 90, bound: true, true_name: "Aelith" }
  @spirit_warden: Spirit { name: "Citadel Warden", form: "translucent", hostility: 40, bound: true, true_name: "Korrath" }

  # ── Items ───────────────────────────────────────────
  @iron_key: Key { name: "Rusty Iron Key", material: "iron" }
  @bone_key: Key { name: "Carved Bone Key", material: "bone" }
  @crystal_key: Key { name: "Resonating Crystal Key", material: "crystal" }
  @bronze_key: Key { name: "Tarnished Bronze Key", material: "bronze" }

  @old_sword: Weapon { name: "Nicked Shortsword", damage: 8, durability: 45.5 }
  @enchanted_blade: Weapon { name: "The Tidecaller", damage: 25, enchanted: true, durability: 100.0 }

  @ritual_scroll: Scroll { name: "Scroll of Binding", contents: "Speak the true name to bind the spirit", language: "ancient", translated: false }
  @cipher_note: Scroll { name: "Smuggler's Cipher", contents: "Shipment arrives at the third bell", language: "cipher" }
  @runic_tablet: Scroll { name: "Runic Tablet Fragment", contents: "The citadel sleeps beneath the tide", language: "runic" }

  @healing_potion: Potion { name: "Healing Draught", effect: "healing", doses: 2 }
  @truth_serum: Potion { name: "Serum of Clarity", effect: "truth", doses: 1 }
  @strength_elixir: Potion { name: "Giant's Breath", effect: "strength", doses: 1 }
  @invisibility_vial: Potion { name: "Veil of Shadows", effect: "invisibility", doses: 1 }

  @lockbox: StorageChest { name: "Scholar's Lockbox", locked: true, requires: @bone_key }
  @old_chest: StorageChest { name: "Waterlogged Chest", locked: true, requires: @bronze_key }

  @gold_idol: Treasure { name: "Golden Idol of the Deep", value: 500, cursed: true, weight: 8.5 }
  @pearl_necklace: Treasure { name: "Pearl Necklace", value: 150, cursed: false, weight: 0.3 }
  @ancient_coin: Treasure { name: "Ancient Coin", value: 50, cursed: false, weight: 0.1 }

  @torn_letter: Evidence { name: "Torn Letter", tags: [conspiracy, voss], examined: false }
  @bloody_cloth: Evidence { name: "Bloody Cloth", tags: [violence, dock], examined: false }
  @cult_symbol: Evidence { name: "Carved Symbol", tags: [cult, ritual], examined: false }

  @gate_door: Door { locked: true, requires: @iron_key, material: "iron" }
  @crypt_door: Door { locked: true, requires: @bone_key, material: "stone" }
  @vault_door: Door { locked: true, requires: @crystal_key, material: "crystal" }
  @cellar_door: Door { locked: false, material: "wood" }

  @tide_mechanism: Mechanism { name: "Tidal Lock", state: "inactive", required_items: [@crystal_key], charge: 0 }
  @bell_mechanism: Mechanism { name: "Signal Bell Mechanism", state: "inactive", charge: 0 }

  @cliff_inscription: Inscription { surface_text: "Salt-worn markings line the cliff face.", hidden_message: "When the tide retreats, the path opens" }
  @altar_inscription: Inscription { surface_text: "Ancient text circles the altar.", hidden_message: "Korrath binds, Aelith frees" }

  @magic_mirror: MagicMirror

  # ── Player (explicit declaration) ───────────────────
  @player: Guard { name: "The Investigator", alertness: "alert", loyalty: 0, orders: "investigate", carrying_torch: true, bribed: false }
---

// ═══════════════════════════════════════════════════════
// LOCATION 1: Village Square — the hub
// ═══════════════════════════════════════════════════════

# Village Square

A windswept square at the heart of a dying fishing village. Cracked cobblestones and boarded-up windows. The salt air carries a faint smell of decay.

[@elder_maren, @child_pip, @fisherman_col, @ancient_coin, @cellar_door]

// Exits with various conditions and blocked messages

-> north: Cliff Path
  ? @captain_rhys.alertness != panicked
  ! Captain Rhys has sealed the cliff path. No one gets through.

-> east: The Drowned Anchor Inn

-> south: Village Gate
  ? @gate_door.locked == false
  ! The iron gate is locked. You need a key.

-> west: Market Stalls

-> down: Cellar Steps
  ? @cellar_door.locked == false
  ? @player.carrying_torch == true
  ! You need a light source to descend.

// ── Village Square dialogue hub ──────────────────────

== square_talk

// hub prompt

@elder_maren: Another stranger. We've had enough trouble.

+ Ask about the village
  @elder_maren sighs heavily.
  @elder_maren: This place has been cursed since the citadel sank. Fish won't bite. The young ones leave.
  > @elder_maren.trust + 5
  -> square_talk

* Ask about the citadel
  ? @elder_maren.trust >= 15
  @elder_maren lowers her voice.
  @elder_maren: The entrance is through the sea caves at low tide. But you'd be a fool to go.
  > @elder_maren.trust + 10
  > @elder_maren.secret = "revealed"
  -> square_talk

  ? @elder_maren.trust < 15
  @elder_maren: I don't know what you're talking about.
  -> square_talk

* Ask about the scholar
  @elder_maren: Voss? Arrived two months ago. Says he's researching folklore. Spends too much time at the docks after dark.
  > @elder_maren.dialogue_count + 1
  -> square_talk

+ Talk to Pip
  @child_pip: Are you here to fight the monsters?

  * Tell the truth
    @child_pip: I knew it! The lights under the water — they're real!
    > @child_pip.trust + 10
    -> square_talk

  * Lie to protect the child
    @child_pip: Okay. But be careful.
    -> square_talk

* Show the torn letter
  ? @torn_letter in player
  ? @torn_letter.examined == true
  @elder_maren goes pale.
  @elder_maren: Where did you get this? Voss... I always suspected.
  > @elder_maren.trust + 20
  > @elder_maren.mood = terrified
  -> square_talk

* Pick up the coin
  ? @ancient_coin in here
  You pocket the old coin.
  > move @ancient_coin -> player
  -> square_talk

@elder_maren: I've said my piece. Be careful who you trust.

// ═══════════════════════════════════════════════════════
// LOCATION 2: The Drowned Anchor Inn
// ═══════════════════════════════════════════════════════

# The Drowned Anchor Inn

Low ceilings and guttering candles. The inn smells of stale ale and damp wood. A fire crackles in the hearth but does little against the chill.

[@innkeeper_bessa, @cipher_note, @healing_potion, @magic_mirror]

-> west: Village Square

-> upstairs: Scholar's Room
  ? @innkeeper_bessa.trust >= 20
  ! Bessa blocks the stairs. "Guests only up there."

== inn_talk

@innkeeper_bessa: What can I get you?

+ Order a drink
  @innkeeper_bessa pours something brown and strong.
  @innkeeper_bessa: Two coppers. Don't ask what's in it.
  > @innkeeper_bessa.trust + 3
  -> inn_talk

* Ask about the cipher note
  ? @cipher_note in here
  @innkeeper_bessa: That? Just scrap paper. Nothing important.
  @innkeeper_bessa glances toward the cellar door.
  > @innkeeper_bessa.mood = suspicious
  -> inn_talk

+ Ask about Voss
  @innkeeper_bessa: Pays on time. Keeps to himself. That's all I need from a guest.
  -> inn_talk

* Confront about smuggling
  ? @cipher_note in player
  ? @bloody_cloth in player

  @innkeeper_bessa: Keep your voice down!

  * Threaten to tell the captain
    ? any:
      @captain_rhys.alertness == alert
      @captain_rhys.alertness == panicked
    @innkeeper_bessa: Fine. Fine! Take the key. Just leave me out of it.
    > move @bronze_key -> player
    > @innkeeper_bessa.trust = 0
    > @innkeeper_bessa.mood = hostile
    -> inn_talk

  * Offer to keep quiet for a price
    @innkeeper_bessa: You're no better than the rest of us.
    > @innkeeper_bessa.trust + 10
    > move @healing_potion -> player
    > @innkeeper_bessa.mood = neutral
    -> inn_talk

* Pick up the cipher note
  ? @cipher_note in here
  You slip the coded note into your pocket.
  > move @cipher_note -> player
  -> inn_talk

* Examine the mirror
  ? @truth_serum in player
  The mirror's surface shimmers. Instead of your face, you see a passage behind the bookcase.
  > reveal @magic_mirror.true_reflection
  -> inn_talk

  ? @truth_serum not in player
  A tarnished mirror. You see your own tired reflection.
  -> inn_talk

* Head upstairs
  ? @innkeeper_bessa.trust >= 20
  -> exit:upstairs

@innkeeper_bessa: I've got work to do.

// ═══════════════════════════════════════════════════════
// LOCATION 3: Scholar's Room
// ═══════════════════════════════════════════════════════

# Scholar's Room

A cramped room stacked with books and loose papers. Strange symbols are chalked on the floorboards.

[@scholar_voss, @lockbox, @bone_key, @ritual_scroll, @torn_letter, @cult_symbol]

-> downstairs: The Drowned Anchor Inn

== voss_study

@scholar_voss: Ah, a visitor. Careful with those books — some are quite valuable.

+ Ask about his research
  @scholar_voss adjusts his spectacles.
  @scholar_voss: Ancient maritime civilizations. The citadel was a temple complex, I believe. Fascinating acoustics.
  > @scholar_voss.trust + 3
  > @scholar_voss.knowledge + 1
  -> voss_study

* Ask about the symbols on the floor
  ? @scholar_voss.trust >= 10
  @scholar_voss: A protective ward. Or so the texts claim. One can't be too careful.
  > @scholar_voss.mood = nervous
  -> voss_study

  ? @scholar_voss.trust < 10
  @scholar_voss: Decoration. Nothing more.
  -> voss_study

* Show the cult symbol
  ? @cult_symbol in player
  @scholar_voss freezes.
  @scholar_voss: Where did you find that?

  * Accuse him
    @scholar_voss: You don't understand what's at stake!
    > @scholar_voss.mood = panicked
    > @scholar_voss.trust = 0
    > reveal @scholar_voss.real_identity
    -> voss_confrontation

  * Play dumb
    @scholar_voss: Just... be careful where you show that. Some symbols attract attention.
    > @scholar_voss.mood = nervous
    > @scholar_voss.trust + 5
    -> voss_study

* Examine the torn letter
  ? @torn_letter in here
  ? @torn_letter.examined == false
  You unfold the letter carefully. It mentions a ritual at the next new moon and references "Brother Voss."
  > @torn_letter.examined = true
  > move @torn_letter -> player
  -> voss_study

* Take the ritual scroll
  ? @ritual_scroll in here
  ? @scholar_voss.mood == panicked
  Voss is too distracted to notice you pocket the scroll.
  > move @ritual_scroll -> player
  -> voss_study

  ? @ritual_scroll in here
  ? @scholar_voss.mood != panicked
  @scholar_voss: Please don't touch my materials.
  -> voss_study

* Try the lockbox
  ? @bone_key in player
  ? @lockbox.locked == true
  The bone key fits perfectly. Inside you find a bone key — no, something else.
  > @lockbox.locked = false
  -> voss_study

+ Take the cult symbol
  ? @cult_symbol in here
  You pocket the carved symbol.
  > move @cult_symbol -> player
  -> voss_study

@scholar_voss returns to his books, ignoring you completely.

== voss_confrontation

@scholar_voss stands, knocking his chair back.

@scholar_voss: The ritual is necessary! The citadel must be awakened — it's the only way to save this village!

* Demand the truth
  @scholar_voss: The spirits were bound by the old priests. Their binding is failing. If we don't renew it, the village drowns.
  > @scholar_voss.trust + 15
  > reveal @scholar_voss.knows_ritual
  -> voss_study

* Attack him
  @scholar_voss stumbles backward.
  > @scholar_voss.health = 50.0
  > @scholar_voss.mood = panicked
  > @scholar_voss.alive = false
  > destroy @scholar_voss
  You search his body.
  > move @bone_key -> player
  -> voss_study

// ═══════════════════════════════════════════════════════
// LOCATION 4: Market Stalls
// ═══════════════════════════════════════════════════════

# Market Stalls

A row of weathered stalls. Most are shuttered. Only one trader remains, her wares spread on an oilcloth.

[@merchant_dara, @old_sword, @strength_elixir, @invisibility_vial]

-> east: Village Square

== market

@merchant_dara: Looking to buy? Everything's final sale.

+ Browse weapons
  @merchant_dara: Got a sword here. Seen some use, but it'll do the job.
  -> market

* Buy the sword
  ? @old_sword in here
  @merchant_dara: Yours for a fair price.
  > move @old_sword -> player
  > @merchant_dara.trades_completed + 1
  -> market

+ Browse potions
  @merchant_dara: Potions from the capital. Genuine article.
  -> market

* Buy the strength elixir
  ? @strength_elixir in here
  > move @strength_elixir -> player
  > @merchant_dara.trades_completed + 1
  -> market

* Buy the invisibility vial
  ? @invisibility_vial in here
  > move @invisibility_vial -> player
  > @merchant_dara.trades_completed + 1
  -> market

* Ask for a discount
  ? @merchant_dara.trades_completed >= 2
  ? @merchant_dara.discount_given == false
  @merchant_dara: Loyal customer, eh? Fine. Next one's cheaper.
  > @merchant_dara.discount_given = true
  > @merchant_dara.mood = generous
  -> market

  ? @merchant_dara.trades_completed < 2
  @merchant_dara: Buy something first, then we'll talk.
  -> market

* Ask about the docks
  @merchant_dara lowers her voice.
  @merchant_dara: Strange folk down at the docks after dark. I don't ask questions. You shouldn't either.
  > @merchant_dara.mood = neutral
  -> market

@merchant_dara: Come back when you've got coin.

// ═══════════════════════════════════════════════════════
// LOCATION 5: Village Gate
// ═══════════════════════════════════════════════════════

# Village Gate

A heavy iron gate blocks the southern road. The metalwork is old but sturdy. Beyond it, the road winds into fog.

[@gate_guard, @gate_door, @iron_key]

-> north: Village Square

== gate_encounter

@gate_guard yawns.

@gate_guard: Nobody in, nobody out. Captain's orders.

+ Ask why the gate is locked
  @gate_guard: Captain says something's coming from the south. Doesn't want anyone leaving until it's sorted.
  > @gate_guard.alertness = alert
  -> gate_encounter

* Bribe the guard
  ? @ancient_coin in player
  ? @gate_guard.bribed == false

  @gate_guard eyes the coin.

  ? @gate_guard.loyalty < 50
  @gate_guard: For something that old? Deal.
  > @gate_guard.bribed = true
  > move @ancient_coin -> @gate_guard
  > move @iron_key -> player
  > @gate_guard.alertness = drowsy
  -> gate_encounter

  ? @gate_guard.loyalty >= 50
  @gate_guard: I don't take bribes. Get lost.
  -> gate_encounter

* Pick up the iron key
  ? @iron_key in here
  ? @gate_guard.alertness == asleep
  You quietly take the key from the hook.
  > move @iron_key -> player
  -> gate_encounter

  ? @iron_key in here
  ? any:
    @gate_guard.alertness == alert
    @gate_guard.alertness == drowsy
  @gate_guard: Don't even think about it.
  -> gate_encounter

* Wait for the guard to sleep
  ? @gate_guard.alertness == drowsy
  You sit quietly. Eventually, Torben's head drops.
  > @gate_guard.alertness = asleep
  -> gate_encounter

* Unlock the gate
  ? @iron_key in player
  ? @gate_door.locked == true
  The key turns with a groan. The gate swings open.
  > @gate_door.locked = false
  > @iron_key.used = true
  -> gate_encounter

@gate_guard: Move along. Nothing for you here.

// ═══════════════════════════════════════════════════════
// LOCATION 6: Cliff Path
// ═══════════════════════════════════════════════════════

# Cliff Path

A narrow path along the cliff edge. Wind howls from the sea below. To the north, the path descends toward sea caves.

[@captain_rhys, @cliff_inscription, @bloody_cloth]

-> south: Village Square

-> north: Sea Caves
  ? @captain_rhys.alertness != alert
  ! Captain Rhys blocks the path. "No one goes to the caves."

-> east: Watchtower

== cliff_talk

@captain_rhys: State your business.

+ Ask about the caves
  @captain_rhys: Off limits. Something down there isn't right. Lost two men last week.
  > @captain_rhys.loyalty + 5
  -> cliff_talk

* Show the bloody cloth
  ? @bloody_cloth in player
  @captain_rhys examines the cloth.
  @captain_rhys: This is Joren's. He went to the caves and didn't come back.
  > @captain_rhys.alertness = panicked
  > @captain_rhys.loyalty + 10
  -> cliff_talk

* Convince him to let you pass
  ? any:
    @captain_rhys.loyalty >= 70
    @captain_rhys.alertness == panicked
  @captain_rhys: Fine. But if you don't come back, I'm sealing the path for good.
  > @captain_rhys.alertness = drowsy
  -> cliff_talk

  ? @captain_rhys.loyalty < 70
  ? @captain_rhys.alertness != panicked
  @captain_rhys: Not a chance. Turn around.
  -> cliff_talk

* Examine the cliff inscription
  ? @cliff_inscription.deciphered == false
  ? @ritual_scroll in player
  You compare the scroll's ancient text to the cliff markings. They match.
  > @cliff_inscription.deciphered = true
  > reveal @cliff_inscription.hidden_message
  -> cliff_talk

  ? @cliff_inscription.deciphered == false
  ? @ritual_scroll not in player
  Salt-worn markings. You can't read them without a reference.
  -> cliff_talk

* Pick up the bloody cloth
  ? @bloody_cloth in here
  You pick up the stained cloth. It's still damp.
  > move @bloody_cloth -> player
  -> cliff_talk

@captain_rhys: Keep moving. And stay away from the edge.

// ═══════════════════════════════════════════════════════
// LOCATION 7: Watchtower
// ═══════════════════════════════════════════════════════

# Watchtower

A crumbling stone tower overlooking the sea. From here you can see the dark outline of reefs and, at low tide, the entrance to something below.

[@runic_tablet, @enchanted_blade, @crystal_key]

-> west: Cliff Path

== watchtower_explore

The wind is deafening up here.

* Examine the runic tablet
  ? @runic_tablet in here
  A fragment of carved stone wedged between the crenellations.
  > move @runic_tablet -> player
  -> watchtower_explore

* Survey the coastline
  ? @player.carrying_torch == true
  You signal with your torch. Far below, something flickers in response.
  > @spirit_lament.form = translucent
  -> watchtower_explore

  ? @player.carrying_torch == false
  You squint into the darkness but can see nothing useful.
  -> watchtower_explore

* Look for hidden compartments
  ? @enchanted_blade.container != player
  You pry loose a stone. Behind it, wrapped in oilcloth: a blade that hums with power.
  > move @enchanted_blade -> player
  -> watchtower_explore

Nothing else to find here.

// ═══════════════════════════════════════════════════════
// LOCATION 8: Sea Caves
// ═══════════════════════════════════════════════════════

# Sea Caves

Saltwater drips from the ceiling. Bioluminescent algae casts an eerie blue glow across the wet stone. The sound of the ocean is everywhere.

[@spirit_lament, @crypt_door, @pearl_necklace, @tide_mechanism]

-> south: Cliff Path

-> down: Submerged Passage
  ? @crypt_door.locked == false
  ? @tide_mechanism.state == activated
  ! The passage is sealed by stone and tide.

== caves_explore

The cave groans with the tide.

* Examine the crypt door
  ? @crypt_door.locked == true
  ? @bone_key in player
  The bone key slides into the lock. Ancient tumblers turn.
  > @crypt_door.locked = false
  > @bone_key.used = true
  > destroy @bone_key
  -> caves_explore

  ? @crypt_door.locked == true
  ? @bone_key not in player
  A door of dark stone. There's a keyhole shaped like a finger bone.
  -> caves_explore

* Encounter the spirit
  ? @spirit_lament.form != invisible
  ? @spirit_lament.hostility > 50

  @spirit_lament: You should not have come here.

  The air grows cold. Your torch flickers.

  * Speak the true name
    ? @ritual_scroll in player
    ? @ritual_scroll.translated == true
    You speak the name: Aelith.
    > @spirit_lament.hostility = 0
    > @spirit_lament.bound = false
    > @spirit_lament.form = corporeal
    -> spirit_freed

  * Fight the spirit
    ? @enchanted_blade in player
    The blade blazes with light.
    > @spirit_lament.hostility = 30
    > @spirit_lament.form = translucent
    -> caves_explore

  * Flee
    -> exit:south

* Take the pearl necklace
  ? @pearl_necklace in here
  You pry the pearls from a calcified hand.
  > move @pearl_necklace -> player
  -> caves_explore

* Activate the tide mechanism
  ? @crystal_key in player
  ? @tide_mechanism.state == inactive
  You insert the crystal key. Gears grind. Water drains from the passage below.
  > @tide_mechanism.state = activated
  > @tide_mechanism.charge + 5
  -> caves_explore

  ? @crystal_key not in player
  ? @tide_mechanism.state == inactive
  A complex mechanism of gears and crystal sockets. It needs a crystal key.
  -> caves_explore

The cave echoes with each wave.

== spirit_freed

@spirit_lament: Free. After all these centuries.

@spirit_lament: The warden below still holds. Use my name against him, and the citadel will open.

> @spirit_lament.bound = false

// ═══════════════════════════════════════════════════════
// LOCATION 9: Cellar Steps
// ═══════════════════════════════════════════════════════

# Cellar Steps

Narrow stone steps descend into darkness. The air smells of brine and something older.

[@old_chest, @truth_serum, @bronze_key]

-> up: Village Square

== cellar_explore

Your torch casts dancing shadows.

* Open the old chest
  ? @old_chest.locked == true
  ? @bronze_key in player
  The bronze key fits. The chest contains a potion that glows faintly.
  > @old_chest.locked = false
  > move @truth_serum -> player
  > @bronze_key.used = true
  -> cellar_explore

  ? @old_chest.locked == true
  ? @bronze_key not in player
  A waterlogged chest. The lock is encrusted with salt but looks like it takes a bronze key.
  -> cellar_explore

* Pick up the bronze key
  ? @bronze_key in here
  A key half-buried in debris.
  > move @bronze_key -> player
  -> cellar_explore

* Search the walls
  ? @player.carrying_torch == true
  You find scratch marks and what might be a hidden passage — but it's collapsed.
  -> cellar_explore

  ? @player.carrying_torch == false
  Too dark to see anything.
  -> cellar_explore

The stairs creak behind you. Probably just the wind.

// ═══════════════════════════════════════════════════════
// LOCATION 10: Submerged Passage
// ═══════════════════════════════════════════════════════

# Submerged Passage

A flooded corridor of ancient stone, now drained by the mechanism above. Sea creatures cling to the walls. Ahead, the passage opens into something vast.

[@altar_inscription]

-> up: Sea Caves

-> forward: Citadel Antechamber
  ? @altar_inscription.deciphered == true
  ! The passage ahead is sealed by warding glyphs.

== passage_explore

Water pools around your ankles.

* Examine the altar inscription
  ? @altar_inscription.deciphered == false
  ? @ritual_scroll in player
  You hold the scroll against the altar text. The words align.
  > @altar_inscription.deciphered = true
  > reveal @altar_inscription.hidden_message
  > @ritual_scroll.translated = true
  -> passage_explore

  ? @altar_inscription.deciphered == false
  ? @ritual_scroll not in player
  Ancient text circles the altar. Without a reference, it's meaningless.
  -> passage_explore

* Search for treasure
  You find nothing but barnacles and broken pottery.
  -> passage_explore

The corridor stretches into darkness ahead.

// ═══════════════════════════════════════════════════════
// LOCATION 11: Citadel Antechamber
// ═══════════════════════════════════════════════════════

# Citadel Antechamber

A vast chamber of blue-black stone. Columns rise into darkness above. At the centre, a raised platform holds a crystalline throne. The spirit of the Citadel Warden hovers before it.

[@spirit_warden, @vault_door, @gold_idol]

-> back: Submerged Passage

-> vault: The Sealed Vault
  ? @vault_door.locked == false
  ! The vault is sealed. Crystal wards shimmer across its surface.

== antechamber_encounter

@spirit_warden: Who disturbs the deep?

The warden's form pulses with cold light.

+ State your purpose
  @spirit_warden: Many have claimed noble purpose. Few spoke the truth.
  > @spirit_warden.hostility - 5
  -> antechamber_encounter

* Speak Korrath's true name
  ? @altar_inscription.deciphered == true
  You call out: Korrath!
  @spirit_warden: You know my name. Then you know what I guard.
  > @spirit_warden.hostility = 0
  > @spirit_warden.bound = false
  > @spirit_warden.form = corporeal
  -> warden_freed

* Present the scroll of binding
  ? @ritual_scroll in player
  ? @ritual_scroll.translated == true
  @spirit_warden: The old words. You have been to the surface altar.
  > @spirit_warden.hostility - 20
  -> antechamber_encounter

* Attack the warden
  ? @enchanted_blade in player
  The Tidecaller blazes. The warden recoils.
  > @spirit_warden.hostility + 30
  > @enchanted_blade.durability - 25.0
  -> antechamber_encounter

  ? @enchanted_blade not in player
  Your mundane weapon passes through the spirit.
  > @spirit_warden.hostility + 10
  -> antechamber_encounter

* Use the golden idol
  ? @gold_idol in player
  ? @gold_idol.cursed == true
  The idol pulses with dark energy. The warden shrieks.
  > @spirit_warden.hostility + 50
  > @gold_idol.cursed = false
  > @gold_idol.value = 0
  -> antechamber_encounter

* Take the golden idol
  ? @gold_idol in here
  You lift the heavy idol. It's warm to the touch.
  > move @gold_idol -> player
  -> antechamber_encounter

The chamber hums with ancient power.

== warden_freed

@spirit_warden: The binding is broken. The citadel is yours to enter.

> @vault_door.locked = false
> @spirit_warden.form = invisible

The vault door shimmers and falls silent.

// ═══════════════════════════════════════════════════════
// LOCATION 12: The Sealed Vault
// ═══════════════════════════════════════════════════════

# The Sealed Vault

A circular chamber of impossible geometry. The walls curve inward and upward, meeting at a point of blinding light. This is the heart of the citadel.

[@bell_mechanism]

-> back: Citadel Antechamber

== vault_explore

The light pulses like a heartbeat.

* Activate the bell mechanism
  ? @bell_mechanism.state == inactive
  ? @crystal_key in player
  You place the crystal key in the mechanism. Gears engage. A deep tone resonates through the citadel.
  > @bell_mechanism.state = activated
  > @bell_mechanism.charge + 10
  > @crystal_key.used = true
  -> vault_finale

* Examine the light source
  ? @truth_serum in player
  Under the serum's influence, you see the truth: the light is a prison. Something is trapped inside.
  -> vault_explore

  ? @truth_serum not in player
  Blinding light. You can't look directly at it.
  -> vault_explore

* Use strength elixir
  ? @strength_elixir in player
  You drink the elixir. Your muscles surge with power.
  > destroy @strength_elixir
  -> vault_explore

* Drop everything here
  > move @gold_idol -> here
  > move @pearl_necklace -> here
  -> vault_explore

The light intensifies. Something is about to happen.

== vault_finale

The bell's tone shakes the foundations. Water rushes in. The citadel is rising.

? @spirit_lament.bound == false
? @spirit_warden.bound == false

@spirit_lament: Both names spoken. Both chains broken.

@spirit_warden: The deep surrenders what it took.

? @spirit_lament.bound == true
? @spirit_warden.bound == true

The citadel shudders. Without the spirits freed, it begins to collapse.

? @scholar_voss.alive == true

@scholar_voss: The ritual is complete! I told you it was necessary!

? @scholar_voss.alive == false

The ritual completes without its architect. Ironic.

-> end

// ═══════════════════════════════════════════════════════
// SEQUENCES — main quest with phased progression
// ═══════════════════════════════════════════════════════

## Main Quest

// Sequence: main-quest — referenced by world.entry

### Investigation

Investigate the village. Talk to the residents. Something is wrong here.

* Talk to the villagers
  -> square_talk

### Discovery

? @elder_maren.secret == "revealed"

You've learned about the citadel entrance. Find a way to the sea caves.

* Proceed to the cliffs
  -> cliff_talk

### The Descent (auto)

The path to the deep opens before you.

> @captain_rhys.alertness = drowsy

### Confrontation

? @crypt_door.locked == false

The citadel awaits. Face what lies beneath.

* Enter the depths

### Resolution (auto)

? @bell_mechanism.state == activated

The citadel rises. The curse is broken. The village will live.

// ═══════════════════════════════════════════════════════
// RULES — NPC behavioral rules
// ═══════════════════════════════════════════════════════

rule spirit_manifests:
  actor: @spirit_lament action manifest
  selects target from [@spirit_lament]
    where target.form == translucent
    where target.bound == true
  > target.form = corporeal
  > target.hostility + 10

rule guard_patrols:
  actor: @captain_rhys action patrol
  selects target from [@captain_rhys, @gate_guard]
    where target.alertness != asleep
    where target.alertness != panicked
  > target.alertness = alert

rule merchant_restocks:
  actor: @merchant_dara action restock
  selects target from [@old_sword, @strength_elixir, @invisibility_vial]
    where target.container != @merchant_dara
    where target.container != player
  > move target -> @merchant_dara
