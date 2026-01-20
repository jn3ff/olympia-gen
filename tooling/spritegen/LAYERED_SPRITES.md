# Layered Sprite System

## Overview

Olympia uses a layered sprite system allowing mix-and-match of character bodies, equipment, and weapons. This enables:
- Weapon swapping within a class (different swords with same animations)
- Equipment customization (different armor appearances)
- Character customization (hair, face, skin tone via palette)

## Layer Stack (render order bottom to top)

```
┌─────────────────────────────────────┐
│  5. Effects (hit sparks, buffs)    │  <- top
├─────────────────────────────────────┤
│  4. Weapon                          │
├─────────────────────────────────────┤
│  3. Front arm (if needed)           │
├─────────────────────────────────────┤
│  2. Armor/Clothing                  │
├─────────────────────────────────────┤
│  1. Base body + back arm + cape     │  <- bottom
└─────────────────────────────────────┘
```

## Generation Strategy

### Character Base (weaponless)
Generate character sprites WITHOUT weapons. Hand should be in grip pose.

```bash
spritegen generate -c player -s base -n zeus_idle_base \
  --reference <character_reference.png> \
  --prompt "same character but WITHOUT sword, empty hand in grip pose ready to hold weapon"
```

### Weapon Sprites (isolated)
Generate weapons separately with transparent backgrounds. Use first weapon as reference for scale consistency.

```bash
# Generate base weapon (idle pose)
spritegen generate -c weapon -s sword -n basic_sword_idle \
  --prompt "Greek xiphos sword, blade at 45 degrees down-right, isolated, no hand, transparent background"

# Generate other poses using base as reference
spritegen generate -c weapon -s sword -n basic_sword_raised \
  --reference <basic_sword_idle.png> \
  --prompt "same sword rotated, blade pointing up for overhead strike"
```

## Attachment Points

Weapons attach to character hands via defined anchor points.

### Standard Anchor (64x64 sprites)
- **Character grip point**: Pixel coordinate where weapon hilt should align
- **Weapon hilt point**: Bottom-center of weapon hilt

### Per-Animation Anchors
Each animation frame may have different anchor positions as the hand moves:

```json
{
  "zeus_idle": {
    "frame_1": {"weapon_anchor": [45, 38], "weapon_rotation": -45},
    "frame_2": {"weapon_anchor": [45, 39], "weapon_rotation": -45},
    ...
  },
  "zeus_attack": {
    "frame_1": {"weapon_anchor": [30, 20], "weapon_rotation": -135},  // raised
    "frame_2": {"weapon_anchor": [50, 35], "weapon_rotation": 0},     // mid-swing
    "frame_3": {"weapon_anchor": [55, 45], "weapon_rotation": 45},    // follow-through
  }
}
```

## Weapon Poses Required

For each weapon class, generate these poses:

| Pose | Angle | Used In |
|------|-------|---------|
| `idle` | 45° down-right | Idle, walk animations |
| `raised` | Up/back | Attack wind-up |
| `swing_h` | Horizontal right | Mid-attack |
| `swing_down` | 45° down-right extended | Attack follow-through |

## Directory Structure

```
assets/sprites/used/
  player/
    base/           # Weaponless character animations
      zeus_idle_1.png
      zeus_idle_2.png
      zeus_walk_1.png
      ...
    clothing/       # Armor/outfit layers (future)

  weapons/
    sword/
      basic/
        idle.png
        raised.png
        swing_h.png
        swing_down.png
      achilles/     # Legendary sword variant
        idle.png
        raised.png
        ...
    spear/
      basic/
        idle.png
        thrust.png
        ...
```

## Manifest Schema (Extended)

```json
{
  "version": 2,
  "assets": {
    "player.base.zeus_idle_1": {
      "path": "sprites/used/player/base/zeus_idle_1.png",
      "frames": 1,
      "size": 64,
      "weapon_anchor": [45, 38],
      "weapon_rotation": -45
    },
    "weapon.sword.basic.idle": {
      "path": "sprites/used/weapons/sword/basic/idle.png",
      "frames": 1,
      "size": 64,
      "hilt_point": [32, 58]  // Where hilt connects to hand
    }
  }
}
```

## Bevy Compositing

```rust
// Spawn layered character
fn spawn_player(commands: &mut Commands, sprites: &SpriteManifest) {
    commands.spawn((
        // Base body layer
        SpriteBundle { texture: sprites.get("player.base.zeus_idle_1"), ..default() },
        PlayerBody,
    )).with_children(|parent| {
        // Weapon layer (positioned relative to body)
        parent.spawn((
            SpriteBundle {
                texture: sprites.get("weapon.sword.basic.idle"),
                transform: Transform::from_xyz(anchor.x, anchor.y, 1.0)
                    .with_rotation(Quat::from_rotation_z(weapon_rotation)),
                ..default()
            },
            WeaponSprite,
        ));
    });
}
```

## Workflow Summary

1. **Generate hero frame** for character (with weapon, for reference)
2. **Generate weaponless base** using hero as reference
3. **Generate weapon idle** pose isolated
4. **Generate weapon poses** using idle as reference for scale
5. **Define anchors** - manually note pixel positions for attachment
6. **Test composite** in game, adjust anchors as needed

## Known Limitations

- Anchor points currently require manual definition
- Weapon rotation per frame needs manual specification
- Some poses may need touch-up for clean layering
- Complex overlaps (arm in front of weapon) may need separate arm layer
