# Playtest Guide - Debug/Creative Mode

This document covers the debug tools available for fast iteration and testing.

## Enabling Debug Mode

Press `F1` or `` ` `` (backtick) to toggle the debug UI panel.

The panel appears in the top-right corner and provides clickable buttons for all debug actions. You can also use keyboard shortcuts directly without opening the panel.

## Keyboard Shortcuts

All shortcuts use `Ctrl+Key` and work whether the debug panel is open or closed.

### Player

| Shortcut | Action |
|----------|--------|
| `Ctrl+I` | Toggle invincibility (auto-heals, permanent i-frames) |
| `Ctrl+H` | Full heal |

### Cheats

| Shortcut | Action |
|----------|--------|
| `Ctrl+M` | Give money (+500 coins) |
| `Ctrl+K` | Kill all enemies in current room (instakill) |

### Spawning

| Shortcut | Action |
|----------|--------|
| `Ctrl+T` | Cycle enemy tier (Minor → Major → Special → Boss) |
| `Ctrl+E` | Spawn enemy at player position (uses selected tier) |
| `Ctrl+B` | Spawn boss at player position |

Spawned enemies use current difficulty scaling based on segment index.

### Warp / State Transitions

| Shortcut | Action |
|----------|--------|
| `Ctrl+1` | Warp to Arena (hub) |
| `Ctrl+2` | Warp to Room |
| `Ctrl+3` | Warp to Boss fight |

### Miscellaneous

| Shortcut | Action |
|----------|--------|
| `Ctrl+S` | Show current seed / set new random seed |
| `Ctrl+D` | Toggle debug info overlay |

## Debug Info Overlay

When enabled (`Ctrl+D`), displays real-time info in the bottom-left:

### Player
- Position (x, y)
- Health (current/max)
- Coins
- Invincibility status

### Run
- Seed
- Segment index
- Current RunState

### Progress
- Rooms cleared (this segment)
- Bosses defeated (this segment)
- Total bosses defeated (entire run)

### Room
- Current room ID
- Current biome
- Enemy count (alive in room)
- Total rooms cleared

## Reproducibility

The run seed determines random elements like enemy spawns and reward rolls. To reproduce a specific run:

1. Note the seed from the debug overlay or `Ctrl+S`
2. Start a new run
3. Use `Ctrl+S` to generate a new seed (or modify `RunConfig.seed` in code)

## Building Without Debug Tools

To build a release version without debug tooling:

```bash
cargo build --release --no-default-features
```

Or explicitly disable the `dev-tools` feature:

```bash
cargo build --release --features ""
```

The debug module is compiled only when the `dev-tools` feature is enabled (default for development builds).
