# Spritegen Skill

Generate and manage game sprites for Olympia using the Gemini API.

## Overview

This skill provides a conversational interface to the spritegen CLI tool for generating pixel art sprites with consistent Pokemon Emerald Gen 3 aesthetic and god-specific color palettes.

## CLI Location

```bash
# Activate venv and run
source tooling/spritegen/.venv/bin/activate && python tooling/spritegen/main.py <command>
```

## Available Commands

### Generate a single sprite
```bash
python tooling/spritegen/main.py generate \
  --category <player|enemy|terrain|weapon|effect|ui> \
  --name <asset_name> \
  --prompt "<description>" \
  [--subcategory <subcategory>] \
  [--god <zeus|poseidon|ares|demeter>] \
  [--frames <N>] \
  [--size <pixels>] \
  [--auto-apply]
```

### Batch generate from manifest
```bash
python tooling/spritegen/main.py batch <manifest.json> [--auto-apply]
```

### List assets
```bash
python tooling/spritegen/main.py list <staging|used|alternatives> [--category <cat>]
```

### Approve staged asset
```bash
python tooling/spritegen/main.py approve <uuid> [--category <cat>] [--subcategory <sub>] [--name <name>]
```

### Reject staged asset
```bash
python tooling/spritegen/main.py reject <uuid> [--reason "<reason>"]
```

### Preview asset
```bash
python tooling/spritegen/main.py preview <uuid|path|manifest_key>
```

## Asset Categories & Default Sizes

| Category | Subcategory | Size | Description |
|----------|-------------|------|-------------|
| `player` | `base`, `hair`, `face`, `clothing` | 64x64 | Player character layers |
| `enemy` | `minor` | 64x64 | Small enemies (1 tile) |
| `enemy` | `major` | 128x128 | Larger enemies (2x2 tiles) |
| `enemy` | `boss` | 256x256 | Boss sprites (4x4 tiles) |
| `weapon` | `sword`, `spear` | 64x64 | Weapon sprites |
| `effect` | `hit`, `dash` | 64x64 | Visual effects |
| `terrain` | `platform`, `wall` | 64x64 | Single tiles |
| `terrain` | `background` | 512x512 | Room backgrounds (8x8 tiles) |
| `terrain` | `tileset` | 256x256 | Tile grids |
| `ui` | `icon` | 32x32 | UI icons |
| `ui` | `health`, `bar` | varies | UI elements |

## God Palettes

Apply themed colors with `--god`:

- **zeus**: Gold, electric blue, white, storm dark
- **poseidon**: Dark blue, black, white foam
- **ares**: Brown/bronze, crimson, steel gray
- **demeter**: Gentle green, frost blue, gentle yellow

## Workflow

### Standard (with review)
1. Generate sprite (goes to `staging/`)
2. Preview and review
3. Approve (moves to `used/`, updates manifest) or Reject (moves to `alternatives/`)

### Auto-apply (for batch work)
Add `--auto-apply` flag to skip staging and write directly to `used/`.

### Reference-Based Animation Workflow (RECOMMENDED)

For consistent animation sets, use a reference image to lock the style:

1. **Generate hero frame** - Create one perfect standing/idle frame that nails the style
2. **Save as reference** - Copy to `assets/sprites/examples/` as the character's style reference
3. **Generate animation frames** - Use `--reference` flag for all subsequent frames:

```bash
# Generate walk frames using hero frame as reference
python tooling/spritegen/main.py generate \
  -c player -s base -n zeus_walk_1 \
  --reference assets/sprites/examples/zeus_reference.png \
  --prompt "same character walking, left foot forward"

python tooling/spritegen/main.py generate \
  -c player -s base -n zeus_walk_2 \
  --reference assets/sprites/examples/zeus_reference.png \
  --prompt "same character walking, right foot forward"
```

**Why this works:**
- Without reference: Each generation is a fresh interpretation (inconsistent)
- With reference: Model matches the exact pixel art style, proportions, and character design

**Key phrases for reference prompts:**
- "same character" - maintains design
- Describe only what CHANGES (pose, action)
- Don't re-describe the character's appearance (armor, colors) - let reference handle it

## Batch Manifest Format

Create a JSON file with an `items` array:

```json
{
  "items": [
    {
      "category": "player",
      "subcategory": "base",
      "name": "body_athletic",
      "prompt": "athletic humanoid body, neutral standing pose, side view",
      "frames": 1
    },
    {
      "category": "enemy",
      "subcategory": "minor",
      "name": "skeleton_idle",
      "prompt": "skeleton warrior, idle stance, holding rusty sword",
      "frames": 4
    }
  ]
}
```

## Sprite Sheet Format

Multi-frame sprites are horizontal strips:
- Each frame is `size x size`
- Total width = `size * frames`
- Example: 4-frame 64x64 = 256x64 PNG

## File Locations

- Sprites: `assets/sprites/`
  - `staging/` - Generated, awaiting review
  - `used/` - Approved and in-game
  - `alternatives/` - Rejected alternatives
- Manifest: `assets/sprites/manifest.json`
- Style guide: `assets/sprites/style_guide.json`
- Metadata: `.meta.json` alongside each sprite

## Prompt Guidelines

For best results:

1. **Be specific**: "skeleton warrior with rusty sword, facing right" not "an enemy"
2. **Include pose**: "idle stance", "mid-swing attack", "walking cycle"
3. **Mention view**: "side view", "three-quarter view", "facing right"
4. **For animations**: Describe the action sequence
5. **God palette**: Only use when the asset belongs to that god's domain

## Example Session

User: "Generate idle animations for the 4 player base bodies"

Claude would:
1. Create batch manifest with 4 body types
2. Run batch command with appropriate prompts
3. Present staging assets for review
4. Approve/reject based on feedback
5. Update game to reference new sprites

## Decomposing Complex Requests

When asked to generate multiple related sprites:

1. **Identify all needed assets** - What categories, how many variants?
2. **Plan composition** - For layered sprites, what layers are needed?
3. **Create consistent prompts** - Same pose/style across related sprites
4. **Generate in batches** - Group related sprites together
5. **Review cohesion** - Ensure sprites look like they belong together

## Layered Sprite System

Olympia uses layered sprites for equipment swapping. See `tooling/spritegen/LAYERED_SPRITES.md` for full documentation.

### Quick Reference

**Layer stack (bottom to top):**
1. Base body (weaponless character with grip pose)
2. Armor/clothing
3. Weapon
4. Effects

**Generating layered sprites:**

```bash
# 1. Generate weaponless character base
spritegen generate -c player -s base -n zeus_idle_base \
  --reference <hero_frame.png> \
  --prompt "same character but WITHOUT sword, empty hand in grip pose"

# 2. Generate weapon idle (establishes scale)
spritegen generate -c weapon -s sword -n basic_sword_idle \
  --prompt "Greek xiphos sword, 45 degrees down-right, isolated, transparent background"

# 3. Generate other weapon poses using idle as reference
spritegen generate -c weapon -s sword -n basic_sword_raised \
  --reference <basic_sword_idle.png> \
  --prompt "same sword rotated, blade pointing up for overhead strike"
```

**Weapon poses needed per weapon:**
- `idle` - 45° down (idle, walk)
- `raised` - Up/back (attack wind-up)
- `swing_h` - Horizontal (mid-attack)
- `swing_down` - Extended forward (follow-through)
