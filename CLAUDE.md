# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
cargo build          # Build the project
cargo run            # Run the game
cargo test           # Run all tests
cargo test <name>    # Run a specific test
cargo clippy         # Lint
cargo fmt            # Format code
```

## Architecture

Olympia is a 2D roguelike action game built with Bevy 0.18 and bevy_xpbd_2d for physics. The game follows a run-based structure: arena → rooms → boss → reward → next segment.

### Plugin Structure

The app is organized into seven Bevy plugins in `src/`:

- **core** - Game states (`GameState`: Boot/MainMenu/Run/Reward/Paused, `RunState`: Arena/Room/Boss/Reward), run configuration, camera setup
- **content** - Data-driven definitions loaded from RON: `CharacterDef`, `GodBlessingDef`, `WeaponDef`, `SkillDef`, `SkillTreeDef`, `RoomDef`
- **movement** - Player controller with Hollow Knight-inspired mechanics (coyote time, jump buffering, dash). Uses `MovementTuning` resource for tweaking values
- **combat** - Hitbox/hurtbox system, health, stagger, invulnerability frames. `DamageEvent` for damage propagation
- **rooms** - Room graph navigation, room instances, exit handling. Events: `RoomClearedEvent`, `BossDefeatedEvent`
- **rewards** - Build progression: equipment loadout (5 slots), base stats, skill tree nodes. Events: `RewardOfferedEvent`, `RewardChosenEvent`
- **ui** - UI systems (currently stub)

### Key Resources

- `RunConfig` - Seed and segment index for the current run
- `MovementTuning` - All movement physics constants
- `MovementInput` - Frame input state
- `PlayerBuild` - Player's current equipment, stats, and unlocked skill nodes
- `RoomGraph` - Current room tracking

### Content System

Game content is defined via RON files using serde-serializable structs. Characters have god blessings that determine starting weapon and three skill slots (passive, common, heavy) plus a skill tree.
