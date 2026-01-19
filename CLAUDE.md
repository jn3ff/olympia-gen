# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build          # Build the project
cargo run            # Run the game
cargo check          # Fast type-checking during iteration
cargo test           # Run all tests
cargo test <name>    # Run a specific test
cargo clippy         # Lint
cargo fmt            # Format code (run before submitting changes)
```

## Project Overview

Olympia is a 2D boss-rush roguelike platformer built with Bevy 0.18 and avian2d physics. Players choose a demigod character tied to an Olympian god, navigate platforming rooms with combat encounters, defeat bosses, and build power through equipment, skills, and blessings.

**Core Loop**: Arena hub → Rooms (5 default) → Boss(es) (2 default) → Reward → Hub → Next segment

**Design Pillars**: Build crafting, combat/movement mastery, narrative progression

## Architecture

### Plugin Structure (src/)

Seven Bevy plugins organized by gameplay domain:

| Plugin | File | Purpose |
|--------|------|---------|
| **CorePlugin** | `core/mod.rs` | Game states, run config, difficulty scaling |
| **ContentPlugin** | `content/mod.rs` | Data type definitions for RON loading |
| **MovementPlugin** | `movement/mod.rs` | Hollow Knight-style controller (coyote time, jump buffer, wall slide, dash) |
| **CombatPlugin** | `combat/mod.rs` | Hitbox/hurtbox system, health, stagger, enemy AI, boss AI |
| **RoomsPlugin** | `rooms/mod.rs` | Room graph, portals, transitions, enemy spawning |
| **RewardsPlugin** | `rewards/mod.rs` | Reward tiers, equipment loadout, skill tree, selection UI |
| **UiPlugin** | `ui/mod.rs` | Health bars, boss bar, death screen |

### Key Types

**Game States**:
- `GameState`: Boot → MainMenu → Run → Reward → Paused
- `RunState`: Arena (hub) → Room → Boss → Reward

**Core Resources**:
- `RunConfig` - Seed and segment index for current run
- `DifficultyScaling` - Per-segment multipliers (health, damage, enemy count)
- `MovementTuning` - Movement physics constants (speeds, timers, cooldowns)
- `MovementInput` - Frame input state
- `PlayerBuild` - Equipment loadout, base stats, unlocked skill nodes
- `RoomGraph` - Current room tracking and cleared rooms list

**Combat Components**:
- `Health` - Current/max HP with `take_damage()`, `heal()`, `percent()`
- `Stagger` - Timer-based vulnerability state
- `Invulnerable` - I-frames after taking damage
- `Hitbox`/`Hurtbox` - Collision detection with owner tracking
- `Team` - Player vs Enemy for friendly fire prevention
- `AttackState` - Current attack type, direction, combo counter
- `AttackType` - Light (8 dmg), Heavy (25 dmg), Special (40 dmg)

**Enemy Types**:
- `EnemyTier`: Minor/Major/Special/Boss with stat multipliers
- `EnemyAI`: Patrol → Chase → Attack → Staggered state machine
- `BossAI`: Complex multi-phase state machine with attack sequences

**Events** (Bevy Message system):
- `DamageEvent`, `DeathEvent`
- `RoomClearedEvent`, `BossDefeatedEvent`
- `EnterRoomEvent`, `ExitRoomEvent`
- `RewardOfferedEvent`, `RewardChosenEvent`

### Physics Layers

Uses avian2d collision layers: Player, Enemy, Ground, Wall, PlayerHitbox, EnemyHitbox, Sensor

## Content System

### RON Data Files (assets/data/)

Game content is data-driven via 22 RON files:

**Characters & Skills**: `characters.ron`, `gods.ron`, `skills.ron`, `skill_trees.ron`, `blessings.ron`

**Weapons & Combat**: `weapon_categories.ron`, `movesets.ron`, `weapon_items.ron`

**Enemies & Encounters**: `enemies.ron`, `enemy_pools.ron`, `encounter_tables.ron`, `encounter_tags.ron`

**World**: `rooms.ron`, `biomes.ron`

**Economy**: `equipment_items.ron`, `minor_items.ron`, `reward_tables.ron`, `reward_pools.ron`, `shops.ron`, `shop_inventory_tables.ron`

**Events & Config**: `events.ron`, `gameplay_defaults.ron`

### Content Definitions (src/content/mod.rs)

- `CharacterDef` - id, name, blessing_id
- `GodBlessingDef` - Starting weapon + 3 skill slots + skill tree
- `WeaponDef` - id, name, weapon_type
- `SkillDef` - id, name, description, slot (Passive/Common/Heavy)
- `SkillTreeDef` - id, nodes with parent relationships
- `RoomDef` - id, exits, dimensions, boss_room flag

## Game Design Context

### V1 Characters (4)
- **Ares sword** - Child of Ares
- **Demeter sword** - Child of Demeter
- **Poseidon spear** - Child of Poseidon
- **Zeus spear** - Child of Zeus

### Combat Defaults
- Light/heavy/special attacks per weapon; no stamina, cooldown-based
- Stance system: 4 heavies or 7 lights to break; 1 light per 10s regen
- Parry = instant break (8 heavies worth of stance damage)
- No repeats for significant enemies (>30s health pool)

### Economy Defaults
- ~300 coin value per segment (drops + rewards)
- Items: 50-150 typical, rare: 250-800
- Tier-up cost ≈ cost of buying that tier
- Passives scale by tier (tier 1 = 1 passive, up to tier 5 = 5)

### Encounter Tags
- Each encounter guarantees one specialty tag
- Curated tags tied to specific weapons
- Buff tags scale with equipment tier

## Current Implementation Status

**Working (POC)**:
- Player movement (run, jump, wall slide, air jump, dash with cooldown)
- Enemy spawning with tier scaling and patrol/chase/attack AI
- Boss spawning with multi-phase attack sequences
- Hitbox/hurtbox collision with knockback
- Stagger and invulnerability frames
- Room generation with 4-directional exits
- Portal conditions (always enabled vs clear-to-exit)
- Arena hub with portal selection
- Reward system with 3-choice UI
- Health bars (player, enemies, boss)
- Death screen with retry
- Difficulty scaling per segment

**Not Implemented**:
- RON data loading (types defined but files not loaded)
- Main menu and character selection
- Weapon movesets from data
- Stance/guard break and parry
- Skills and passive effects
- Shop system
- Encounter tags and events
- Pause menu, save/load

## Implementation Roadmap (PLAN.md)

| Milestone | Goals |
|-----------|-------|
| **M1** | RON data pipeline with validation and ContentRegistry |
| **M2** | Character + build bootstrapping from data |
| **M3** | Weapon movesets + stance/parry system |
| **M4** | Segment flow + room selection + no-repeat logic |
| **M5** | Rewards + economy + shop loop |
| **M6** | Encounter tags + curated events |
| **M7** | Faith tracking + adversarial events |
| **M8** | Debug/creative mode |

M1 is prerequisite for all others. M2+M3 required before M4. M8 can be built in parallel.

## Coding Conventions

- **Rust style**: 4-space indent, `snake_case` functions/fields, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants
- **Bevy patterns**: `XxxPlugin` naming, clear Component/Resource/Event derivations
- **Module boundaries**: Map to gameplay domains; keep systems in relevant modules
- Run `cargo fmt` before submitting; `cargo clippy` recommended for behavior changes

## Key File Locations

- Entry point: `src/main.rs`
- Plugin modules: `src/{core,content,movement,combat,rooms,rewards,ui}/mod.rs`
- Data files: `assets/data/*.ron`
- Design docs: `concept/GAME-SPEC.md`, `concept/INTERVIEW.md`
- Roadmap: `PLAN.md`
- Repository guidelines: `AGENTS.md`
