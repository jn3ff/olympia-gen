# Olympia Game Spec (Working Draft)

Purpose: Capture current vision, gaps, and implementation-impacting decisions.
Sources: concept/GAME.md, src/ code scan.

## Current Prototype Snapshot (from src/)
- Core loop skeleton: RunState = Arena -> Room -> Reward, with segment_index driving difficulty scaling.
- Movement: single jump, ground-only dash by default, wall slide/jump, coyote time, jump buffer.
- Combat: light/heavy/special attacks with hitboxes; basic enemy AI (patrol/chase/attack), boss AI with scripted sequences.
- Rooms: directional exits (Up/Down/Left/Right), arena hub selection, portal enable conditions, simple platform placement.
- Rewards: skill node, equipment, or stat reward; tier system with rarity weights; equipment + skill registries are stubbed data.
- UI: health bars, boss bar, reward UI, death screen.
- Missing from code: gods/characters, blessing trees, shops, currency, equipment passives, narrative events, meta-progression.

## Vision Summary (from concept/GAME.md)
- 2D side-scrolling boss-rush roguelike platformer; run starts in arena hub.
- Core loop: platforming room -> regular enemies -> directional choice -> boss -> reward.
- Modular content: rooms, bosses, upgrades are plug-and-play.
- Movement modeled after Hollow Knight; dash/roll starts ground-only; movement upgrades unlock.
- Equipment slots (armor + main hand); items grant stats/passives and can tag narrative encounters.
- Weapons have base category movesets with per-attack overrides (light/heavy/special).
- 12 demigod characters (one per Olympian); each has passive + skill + ultimate slot.
- God blessing trees + cross-god champion synergies; skills swap between rooms.
- Roguelike twist: gods’ POV meta progression using “faith” currency (improve trees, stats, starting gear, encounter rates).
- Optional ideas: ascension to godhood, special encounter buffs.

## Interview Answers Summary (from concept/INTERVIEW.md)

### Vision and Pillars
- Pillars: build crafting, combat and movement mastery, narrative progression.
- Desired memories: builds, RPG decisions, standout boss fights, and layered story (demigod plus god POV).
- Roguelite emphasis: meta upgrades are impactful; combat should be hard without them.

### Run Structure and Pacing
- Segment = configurable 1-5 rooms culminating in a boss; segment types can be boss-heavy.
- Default segment is 5 rooms with 2 bosses; this is modular/configurable.
- Major hub appears at the end of each segment (shops + next segment decision).
- No visible map; portals show next region theme, difficulty, and reward info.
- Direction choices can map to biomes and movement modifiers; default movement is left-to-right.
- No short runs; target 1-2 hours for a successful run.
- Win condition is a god-specific end-state leading to a final battle; path varies per god.
- Default win condition can be hardcoded to 5 segments for now; should be fully configurable.

### Rooms and World Layout
- Mix of handcrafted rooms and procedural templates; theme consistency matters.
- Rooms can be traversal- or combat-weighted; modifiers and hazards allowed.
- No backtracking between rooms; portals are one-way.

### Combat and Weapons
- Light/heavy/special are weapon movesets; blessings modify those movesets.
- No stamina; attack cooldowns only.
- Parry windows for eligible weapons; stance/guard break system planned; no perfect-dodge rewards.
- At launch, one unique weapon category per player; ranged later.
- Rarity is numeric scaling; blessings add behavior modifiers.
- Elemental tags from blessings; stacking allowed; broken builds are acceptable.
- Off-class weapons are allowed; default reward bounds: min 1, max 2 in-class options.
- Each strike has damage, stance damage, and cooldown; combos are configurable and loopable.
- Stance is per-enemy with regen; parry usually stuns, but bosses may require multiple parries.
- Stance defaults: 4 heavies or 7 lights to break; regen is 1 light per 10s.
- Parry applies an instant break (8 heavies worth of stance damage).
- On break, stance refills to 100% times an internal multiplier.

### Movement and Player Feel
- Wall jump/slide is base kit.
- Character-specific movement modifiers exist; movement tech unlockable.
- Air dash is an unlock; reset on ground touch (configurable later).
- Equipment movement effects are reversible; god/character passives are permanent for the run.

### Characters, Gods, and Skills
- Character chosen at run start; god tied to character.
- Champion of another god via choices/events with tradeoffs; adds a synergy tree.
- Boons are positive; curses arise from narrative choices.
- God tree size not defined; cannot complete a tree in a single run.
- V1 characters (4):
  - Sword: child of Ares
  - Sword: child of Demeter
  - Spear: child of Poseidon
  - Spear: child of Zeus

### Rewards and Economy
- Every room has a reward; bosses grant heavier rewards.
- Regular rooms can grant money, max health, or minor items.
- Blessings are permanent (scope TBD); skills and equipment can be upgraded or replaced.
- Shops appear after meta-segments; wandering shops can appear in rooms.
- Shrines allow money-for-rarity or build buffs.
- Money from enemy drops, sales, or rewards.
- Rerolls unlockable but narrow; choice philosophy is deterministic.
- Consumables are minor and consumed on purchase; sell-only loot is rare.
- Inventory is effectively unlimited, but limit how much junk can drop.
- Shop economy defaults are open to configuration for now.
- Segment economy target: ~300 coin value per segment (including drops).
- Item costs: 50-150 typical; rare items 250-800.
- Upgrades scale aggressively; tier-up cost ~= cost of buying that tier.
- Passives per weapon scale by tier (tier 1 = 1 passive, up to tier 5 = 5 passives).

### Meta Progression (Faith)
- Faith upgrades persist per god; choose a starting loadout from unlocked buffs.
- Faith earned from performance and story; can go negative for opposing gods.
- Horizontal and vertical progression both allowed.
- Meta faith cannot drop below zero; run faith can.
- Negative run faith can unlock adversarial events with redemption paths.
- Adversarial events should trigger within 3 segments by default (configurable).

### Narrative and Events
- Blend of in-run (short, choice-driven) and post-run (relational, expositional).
- Rare high-impact events preferred.
- Equipment tags can increase odds or guarantee encounters; guaranteed tags transform after completion.
- Deaths produce narrative beats based on severity.

### Bosses and Enemies
- V1 target: 5 bosses; no repeats within a run.
- No repeats for any significant combat; avoid re-fighting anything with a >30s health pool.
- Elites drop high-rarity rewards; bosses are narrative, segment-ending, and have curated pools.
- A run can have 10-20 boss fights based on choices; longer-term pool target is 40-100 bosses.
- Bosses scale by segment/game phase; harder bosses appear later.
- Multi-phase/stance support is configurable; telegraph readability varies by boss.
- Win condition default: number of bosses defeated (configurable).

### Content Pipeline and Modularity
- Content authored in RON; designers should tweak data without code changes.
- Debug/creative mode required: spawn, apply builds, invincibility; seed control is optional.
### Encounter Tags and Equipment
- Curated encounter tags are associated with specific weapons.
- Regular buff tags exist on every weapon, with quality and count scaling by tier.

## Remaining Gaps for Modular Implementation
- Win-condition state machine extensions beyond boss count (quest steps, narrative flags).
- Boss density controls per segment (tuning knobs for room count vs boss count).
- Stance/guard-break numeric defaults (meter sizes, regen rates, parry multipliers).
- Blessing scope details (run-only vs meta loadout limits; slotting rules).
- Reward economy (drop rates for junk vs meaningful items).
- Shop economy (pricing curve, upgrade/enchant limits, rarity odds).
- Faith system edge cases (adversarial event frequency, redemption thresholds).
- Encounter tag system (tag taxonomy, consumption rules, guarantee cadence).

## Follow-up Questions (post-interview)
1. Define concrete quest/flag conditions and how AND/OR composition should be authored in data.

## Pitfalls and Best-Practice Notes (based on common roguelite patterns)
- Long runs (30-120 min) risk fatigue; consider optional checkpoints, or segment breaks with strong “closure” beats.
- Boss-rush fatigue: too many similar bosses without build variance can feel repetitive; prioritize meaningful build shifts.
- Power creep: stacking permanent buffs can trivialize early segments; lean on horizontal unlocks or difficulty scaling.
- Pure RNG rewards can feel unfair; use weighted pools, guarantees, or pity timers to align with build intent.
- Too many overlapping systems early can overwhelm; gate complexity over early meta progression.
- Directional choices feel hollow without clear tradeoffs; give each direction a consistent identity (theme, risk, reward).

## Areas That Likely Benefit from Specific Mechanics
- Boss fights: telegraph language + punish/reward windows (parry, perfect dodge, stagger).
- Build identity: 1-2 strong synergies per run; allow targeted rerolls or “god favor” bias.
- Economy: add clear sinks (upgrades, rerolls, gamble) to prevent hoarding.
- Run flow: “rest” or “shop” rooms between bosses to reset tempo.
- Difficulty: introduce adaptive scaling if run time extends beyond target.

## System Architecture Notes (for modular extensibility)
- Data-driven definitions for: gods, trees, skills, weapons, equipment, rooms, bosses, encounters.
- Tagging system for equipment/skills to drive narrative triggers and encounter modifiers.
- Reward pipeline should accept “constraints” (min/max weapon types, slot needs, tree bias).
- Separate “effects” from “content”: buffs/debuffs as composable, reusable components.
- Save data schema should distinguish run state vs meta state and support versioning.

## Open Assumptions (until answered)
- Blessings are run-permanent unless explicitly marked as meta-persistent.
- Boss density defaults are tuned for 5 rooms / 2 bosses per segment.
- Drops should always be valuable; "junk" = off-build items for sale.
- Encounters guarantee a single specialty tag by default (configurable).

## Win-Condition Extension Notes
- For now, win condition defaults to boss count; leave commented-out options for segments cleared, faith gained, and quest steps.
- Eventual goal: compose win conditions with AND/OR logic in data.
