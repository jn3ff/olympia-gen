Vision and Pillars

  1. What are the top 3 pillars (e.g., boss mastery, platforming flow, build crafting)?
    - Build crafting
    - Combat and movement mastery
    - Narrative progression
  2. What should players remember after a run: bosses, movement, builds, story?
    - Builds
    - RPG decisions (kill/save, which upgrades to prioritize, which skills to learn)
    - Combat (that boss fight was so cool! I can't wait to face them again)
    - Story (micro-character perspective i.e. lore, character interface and macro-god perspective i.e. retro on demigod child's performance, relationships impacted with other gods, macro-upgrade decisions to make for persistent character upgrades.)
  3. How “roguelike” vs “roguelite” do you want this to feel?
    - Roguelite, god-perspective upgrades are impactful. Game skill should be first class, so combat is challenging and "too hard" without deaths -> permanent upgrades.
  Run Structure and Pacing
  4. What is a “segment” concretely: how many rooms, and does each segment always end in a boss?
    - A segment is a series of rooms that culminates in a boss fight. Each segment typically consists of 1-5 rooms, which is configurable. The number of rooms can be adjusted based on the desired difficulty and pacing. The number of rooms in a segment can be randomized or fixed, depending on the type of segment/region/specialty encounter. Some segments may have a boss fight in every room, while others may have a boss fight at the end of the segment.
  5. What is the win condition: defeat a final boss, clear N segments, or endless?
    - The goal is to reach some end-state. This will be triggered on equipment/reward/character upgrades, and driven via directional/narrative decisions. Depending on parent god, the end state will look different, e.g. Hades' child will want to reach the underworld, Zeus' Olympus, Ares' the site of a war, etc. Concrete reunion with the parent god is the ultimate goal. The win condition will always be a final battle or boss (depending on the parent god's nature) and won't necessarily be uniform for a given god (e.g. for Ares, maybe it's to turn the tide of a war (swarm god) or maybe it's to defeat some canonical enemy of Ares, or maybe it's a long-running quest to bring him an artifact/piece of equipment.)
  6. Do you want a visible map (Hades-like), or purely in-room directional choices?
    - no visible map, but "next region" choices will have information by the portal - next segment's theme, difficulty, and rewards.
  7. Are up/down/left/right tied to biomes/themes, difficulty, or specific rewards?
    - Sort of. e.g. undersea combat will have a different biome with room-specific movement modifiers. We should be able to configure this emergently as gameplay vision progresses during development. Most rooms should be tradition left to right platforming.
  8. Should runs be shrinkable (short mode) or only long-form?
    - no shrinkable runs. One run is one run, intended gameplay time is 1-2 hours for a successful run.

  Rooms and World Layout
  9. Are rooms handcrafted, procedural from templates, or fully procedural?
    - some handcrafted, some procedural from templates. Fully procedural is less likely - a room's theme should be respected, which means certain aspects of movement/combat in the room should be consistent.
  10. Do you want traversal puzzles or mostly combat arenas with light platforming?
    - both. The platforms should be engaging from movement alone, within a combat context. Some rooms should be more traversal-focused, while others should be more combat-focused.
  11. Should rooms have modifiers (hazards, darkness, enemy elites, platform rules)?
    - Yes.
  12. How strict should “no backtracking” be?
    - Very strict. No backtracking at all. Once you select a door, you cannot go back. Within a room, you can go anywhere, but portals are one-way.

  Combat and Weapons
  13. Do light/heavy/special map to weapon movesets only, or also to god skills?
    - Just weapon movesets. God blessings will be applied to those movesets. God skills have the regularly triggerable one, and also the "ultimate". 
  14. Will attacks consume stamina or a resource? If yes, define regen rules.
    - No stamina. Just attack cooldown. 
  15. Do you want hitstop, parry windows, perfect-dodge rewards, guard breaks?
    - Yes - if a weapon can parry, there should be a parry window. Guard breaks are a thing, configurable per enemy + weapon ("stance damage"). No perfect-dodge rewards.
  16. How many weapon categories at launch? Which must be first-class?
      - At launch, one unique weapon category per player. But these behave roughly similarly (light, heavy, special). To start, let's not focus on ranged weapons, but they will be added later.
  17. Is weapon rarity purely numeric scaling or unique behavior per tier?
    - Numeric scaling, with buffs having their own behavior & numeric scaling per tier. E.g. Zeus upgrades will add lightning or wind buffs to the weapon - these can also be weaker or stronger depending on the tier. Some upgrades may modify behavior, e.g. "the second strike in a heavy combo will bring down an enormous bolt of lightning that strikes the enemy and all nearby enemies, dealing <number> damage."
  18. Will weapons have elemental tags or damage types (fire, lightning, etc.)
    - Yes, this will come from the blessing, which can either be applied on the character, or on the weapon itself. Elemental tags are not unique/singular - a weapon can deal e.g. fire and lightning if it's an innately fire sword and the character is taking zeus buffs, or if the player has a regular sword and is taking both zeus and hestia buffs. Etc, this can compose indefinitely. Risk of rain type system where "broken" builds are part of the fun.

  Movement and Player Feel
  19. Is wall jump/slide part of the base kit or an unlock?
    - wall jump/slide is part of the base kit. Certain characters will have unique movement mechanics, such as the ability to glide or perform a double jump. E.g. the child of hermes may have a higher base movement speed, the child of zeus may have a higher base jump height and allow air dash, the child of hephaestus may have a lower base movement speed but landing on the ground will AOE and knockback nearby enemies.
  20. Is air dash an unlock, and should it reset on hit/kill?
    - air dash is an unlock. Allow configurability for reset conditions, but at the start, it resets on ground touch.
  21. Any movement tech like pogo, bounce, grapples, or slide?
    - Yes, movement tech like pogo, bounce, grapples can be unlocked (either in-run or as permanant/one time buffs) and configured for different characters.
  22. Should movement upgrades be reversible during a run (swap gear) or permanent?
    - For movement upgrades that come from equipment, they will be reversible if the piece of equipment is taken off. For full passives from god or character upgrades, they are permanent.

  Characters, Gods, and Skills
  23. How is a character chosen at run start, and is the god tied to the character?
    - Characters are chosen at run start, and the god is tied to the character. Each character has a unique set of skills and abilities that are tied to their god. Secondary god encounters are findable during the run and have their own set of tradeoffs. These encounter rates can be boosted/set in the persistent upgrade phase at the end of the run when the player is embodying the god. The player can choose to become a champion of another god during the run, which can introduce tradeoffs or curses tied into narrative progression & story decisions (i.e. do you forsake your parent? champion can be synergizing or exclusive). Choices throughout a run should shape the narrative and impose real tradeoffs at each turn.
  24. Does each god tree have unique mechanics (e.g., Zeus = chaining lightning)?
    - Yes, each god tree has unique mechanics that are tied to their respective gods. These mechanics can include special abilities, passive effects, and unique item interactions. Each god tree also has its own unique set of rewards and mechanics that are tied to their respective gods.
  25. How does a player become a champion of another god (choice, event, cost)?
    - choice, at multiple levels. E.g. if I am playing as child of zeus, and I go underwater, I may have significant debuff in the underwater land, but perhaps encounter poseidon down there. or I can choose a different path (not to the beach) and avoid it. Sometimes this may have a cost, a choice may lock the player out of one set of options, decisions made respective to events should have benefits and costs.
  26. Can a run have multiple champion trees or only one?
    - One run has one base tree for the player's god, but becoming a champion introduces an auxiliary tree that synergizes with the base god's mechanics. E.g. Hephaestus + Hades may have a "rare metals" upgrade that synergizes respective mechanics. 
  27. Are god boons always positive, or can they introduce tradeoffs/curses?
    - Always positive, with a natural tradeoff in "what choice is better for my build?" If narrative events between gods come, curses should be driven by player choices in those scenarios.
  28. What is the expected size of a god tree (nodes, tiers, depth)?
    - No comment on this for now. Relatively sophisticated, unique per god, player should not be able to unlock the entire tree in a given run (if endless mode is added, then they can there)

  Rewards and Economy
  29. How frequently do rewards occur (after every boss only, also mid-run)?
    - Every room has a reward, but reward tier is determined by encounter. Boss will be heavy reward, regular room clear might just be some money or max health, or minor item (which can be carried and sold at shops.)
  30. How should blessing rewards differ from skill nodes and equipment?
    - Blessing rewards are powerful and permanent, skills and equipment can be upgraded or replaced.
  31. What are shop types and how often are they placed?
    - Generally players can visit the same set of shops after a meta-segment (beach, forest, mountain, city, plains, underworld, etc) for each meta-segment. These are equipment purchasing, equipment upgrades (blacksmithing for base weapon improvement or enchanting for weapon buffs tied to weapon) Wandering shops can be placed in rooms with a more limited set of options. Money can also be sacrificed to gods at their shrine to improve reward rarity rate, or otherwise buff the player's ability to generate a build.
  32. How is money earned (enemy drops, objectives, perfect fights)?
    - Every enemy should drop some money. Items can be sold. Money can be chosen as a reward in certain encounters traded off against equipment. 
  33. Do you want rerolls, lock-ins, or “choose later” reward banking?
    - Rerolls are unlockable but should be narrow for now. Lets focus on that later. Choices are deterministic philosophically ("what you see is what you get") but will be randomized, with the exception of special rewards, a set of which may always be offered from certain bosses or encounters.

  Meta Progression (Faith System)
  34. What is persistent across runs: stats, trees, gear unlocks, narrative flags?
    - God's child upgrades purchased with faith are persistent. The gods are the "main characters" that the players get to know. Base stats, equipment, starting buffs are unlockable from the faith system, but constraints will be imposed. E.g. when enough is unlocked, a player as a character will choose a "starting loadout" of buffs (rather than them always being active as soon as they're unlocked). This system should be modular & flexible.
  35. How should “faith” be earned: boss kills, performance, story decisions?
    - All of the above. A bucketed meta score will be kept during gameplay which tracks "faith generated/lost" for each god. It should be rare, but it is possible to lose faith below 0 in the meta stat tracker if a player does a particularly adversarial action towards a specific god. I.e. if my persistent faith score on Ares is 5, and I play as a child of Aphrodite and diplomatically end a war (angering Ares) I may end up with -1 on the run for Ares, lowering his faith to 5. This sort of broad shaping decision I expect to provide the player with long-term narrative coherence & consequences. Faith is spent on upgrades.
  36. Do you want horizontal unlocks (new options) vs vertical power increases?
    - both
  37. Should meta progression be per god, per character, or account-wide?
    - per god and per character are the same thing. The character is noted not by name but by "Son/Daughter of <God>" So god-level progression applies to all runs where the player plays as the child of that god.

  Narrative and Events
  38. How much narrative in-run vs post-run?
    - a good blend. in run narrative should be shorter, very choice-impact driven. post run narrative can be more relational, expositional, and character-driven. The player should want to keep exploring & fighting within run, while post-run is about lore exploration, character development, story progression, and shouldn't have the same sense of urgency.
  39. What event cadence feels right: rare high-impact vs frequent low-impact?
    - rare high-impact events narratively are good. Choices don't need to be made e.g. fighting a boss - just win and collect your reward, or die. Deaths should be narratively acknowledged at the start of a post-run segment.
  40. Should equipment tags trigger guaranteed events or just increase odds?
    - both, depending on the equipment tag. E.g. one weapon buff might be "increased odds of encountering wandering shops" or "give bosses 20% more HP for better rewards", while another might be "Winter is coming" which guaratees an encounter with Demeter, while another might be "The stone lusts" which guarantees a boss fight with Medusa. Upon completing a guaranteed encounter buff, the buff should be transformed as a part of the reward for that completion.
  41. Any “failure state” story beats (e.g., dying to a specific boss)?
    - Yes. dying to minor enemy should produce a minor acknowledgement ("oh, those nymphs can be vicious!") while dying to a major enemy may produce more nemesis-like persistent narrative beats. 
  

  Bosses and Enemies
  42. Boss count for V1 and how many repeats are acceptable?
    - No repeats within a run. for V1, lets just do 5 bosses so we don't die due to the work, but this should be developmentally uncapped. Within a single run a player may encounter 10-20 bosses depending on the choices they make.
  43. Do bosses scale with segment or have fixed tiers and variants?
    - Yes, they scale with segment, this is drive by "game phase" - a rough heuristic on how strong the character could be at a given point (n bosses defeated, n rooms cleared, etc.) Some bosses are uniformly more difficult and so will only show up in later segments.
  44. Should bosses have multi-phase or stance mechanics by default?
    - Yes, configurably. An easier boss may have only one phase, while a harder boss may have multiple phases or stances.
  45. How “readable” should telegraphs be (Cuphead clear vs subtle)?
    - different depending on the boss.

  Content Pipeline and Modularity
  46. Preferred data format for content (RON/JSON/Serde assets)?
    - RON
  47. Should designers tweak knobs in data without code changes?
    - Yes, they should be able to tweak knobs in data without code changes - but this is purely from an code-organization perspective. 
  48. Do you want a debug/balance UI (spawn any room, set run seed)?
    - Yes, critically, I need to be able to run tests without code changes, and "drop in" to phases of the game while under development. some standard build templates need to be available, as well as a fully modifiable build template. Ideally this looks like a "creative mode" within-game for development's sake, which could turn nicely into a feature or a modding tool.
