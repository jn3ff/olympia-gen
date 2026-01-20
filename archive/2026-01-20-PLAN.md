# Olympia Implementation Plan

## North Star (Next Stage)
- Data-driven playable loop using RON content from `assets/data`.
- Segment flow: rooms -> bosses -> hub -> next segment, with configurable defaults (5 rooms, 2 bosses).
- 4 V1 characters (Ares/Demeter sword, Poseidon/Zeus spear) with starter weapons and skills.
- Weapon movesets from data, stance/guard break, parry breaks stance.
- Reward economy and basic shops (armory, blacksmith, enchanter).
- Encounter tags and curated events, one specialty tag guaranteed per encounter.
- Debug/creative mode: spawn enemies/bosses, apply builds, invincible toggle.

## Guiding Decisions (from concept/GAME-SPEC)
- No visible map; portals show region theme, difficulty, rewards.
- No backtracking between rooms; portals are one-way.
- No repeats for significant enemies (anything with a >30s health pool).
- Segment hub after each segment; win condition defaults to boss count (configurable).
- Combat: light/heavy/special per weapon, no stamina, cooldown-based.
- Stance defaults: 4 heavies or 7 lights to break; 1 light per 10s regen; parry = instant break.
- Economy baseline: ~300 coin value per segment, items 50-150, rare 250-800.
- Blessings stack in-run; meta blessings require loadout slots.

## Milestones

### M1: RON Data Pipeline + Validation
Goals:
- Load all RON content in `assets/data` into registries.
- Validate cross-references (ids, pools, movesets, weapon categories, tags).

Work items:
- Add data structs for each RON file in `src/content` (or new `src/content/data` module).
- Implement a loader that reads the full catalog at startup (and optionally hot-reloads).
- Build `ContentRegistry` with `HashMap<String, T>` lookups.
- Add validation pass with clear error messages for missing ids.

Definition of Done:
- Game boots and logs a summary of loaded content counts.
- Invalid id references cause a readable error and fail fast.

### M2: Character + Build Bootstrapping
Goals:
- Create a player build from RON data and support character selection.

Work items:
- Add `CharacterDef` expanded to include parent god, starting weapon, skills, base stats.
- Implement selection UI or temporary console selection (Ares/Demeter/Poseidon/Zeus).
- Spawn player with starting weapon/moveset and skill slots from data.

Definition of Done:
- Selecting a character updates weapon moveset, base stats, and skill loadout.

### M3: Weapon Movesets + Stance System
Goals:
- Drive attacks from data and integrate stance/guard break.

Work items:
- Replace hardcoded attack configs with moveset data from `movesets.ron`.
- Add strike combos, loop points, cooldowns, and per-strike hitboxes.
- Add stance meter to enemies; apply stance damage from strikes.
- Implement parry window and parry stance break.

Definition of Done:
- Light/heavy/special use moveset data.
- Stance breaks trigger a short vulnerable state; parry breaks stance.

### M4: Segment Flow + Room Selection
Goals:
- Implement data-driven segment progression and hub selection.

Work items:
- Segment resource: room count, boss count, biome, and encounter table.
- Room selection from `rooms.ron` by biome and type.
- Boss encounter tracking and no-repeat enforcement for significant enemies.
- Hub state with portal info and next-segment choice.

Definition of Done:
- Player can complete a segment and return to hub with next-segment choice.
- Significant enemies do not repeat within a run.

### M5: Rewards + Economy + Shops
Goals:
- Data-driven rewards and basic shop loop.

Work items:
- Reward tables and pools from `reward_tables.ron` and `reward_pools.ron`.
- Money drops with segment value target and sell-only loot support.
- Shop inventory and pricing from `shops.ron` and `shop_inventory_tables.ron`.
- Upgrade/enchant rules and costs (tier parity).

Definition of Done:
- Boss rewards pull from blessing/equipment pools.
- Hub shops can buy gear, upgrade tier, and add passives.

### M6: Encounter Tags + Events
Goals:
- Curated encounter tags tied to weapons and guaranteed tag per encounter.

Work items:
- Tag system (curated event tags + buff tags).
- Apply tags at encounter start; on completion, transform curated tag into a buff.
- Basic event dispatcher that can spawn combat/narrative encounter hooks.

Definition of Done:
- Each encounter applies one specialty tag by default.
- Curated tag triggers a matching event or modifier.

### M7: Faith and Meta Hooks (Minimal)
Goals:
- Add run-faith tracking and basic negative faith events.

Work items:
- Track run faith per god with floor at 0 for meta faith.
- If run faith < 0, schedule adversarial event within 3 segments.
- Add placeholder post-run screen to show faith deltas.

Definition of Done:
- Faith values update during run and can trigger adversarial events.

### M8: Debug/Creative Mode
Goals:
- Fast iteration tools.

Work items:
- Dev UI overlay with toggles: invincible, spawn enemy/boss, apply build template.
- Warp to hub/room/boss phases without code changes.
- Optional seed set for reproducibility.

Definition of Done:
- Can enter a boss fight, apply a build, and enable invincibility from UI.

## Sequencing and Dependencies
- M1 is prerequisite for all other milestones.
- M2 and M3 are required before M4 (segment flow needs builds and combat).
- M5 depends on M1 and partial M4 (hub location).
- M6 can start after M1, but is best after M4 (encounter flow).
- M7 depends on M6.
- M8 can be built in parallel after M1.

## Verification Checklist
- `cargo run` loads RON data without errors.
- Basic run loop: hub -> rooms -> boss -> hub.
- No-repeat logic works for elites/bosses.
- Shop purchases and upgrades affect weapon stats/passives.
- Curated encounter tags trigger events or modifiers.
- Debug menu can spawn boss and apply loadouts.

## Open Decisions (Deferred)
- Win-condition composition rules (AND/OR across boss count, segments, faith, quest steps).
- Exact enemy taxonomy labels and thresholds for "significant".
- Long-term boss pool scaling and variant rules.
