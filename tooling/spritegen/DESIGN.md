# Spritegen Design Document

## Overview

Spritegen is a CLI tool + Claude Code skill for generating, managing, and applying game sprites using the Gemini API. It maintains visual coherence across all assets through style guides and god-specific color palettes.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Claude Code Skill                           │
│                      /spritegen                                 │
└─────────────────────┬───────────────────────────────────────────┘
                      │ invokes
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Python CLI                                  │
│              tooling/spritegen/main.py                          │
│                                                                 │
│  Commands:                                                      │
│  - generate   Generate single sprite/sheet                      │
│  - batch      Generate from manifest file                       │
│  - approve    Move staging → used                               │
│  - reject     Move staging → alternatives                       │
│  - list       Show assets by status                             │
│  - apply      Update Bevy code to use asset                     │
│  - preview    Open asset in viewer                              │
└─────────────────────┬───────────────────────────────────────────┘
                      │ calls
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Gemini API                                    │
│            gemini-3-pro-image-preview                           │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
assets/
  sprites/
    staging/              # Generated, awaiting review
      <uuid>/
        sprite.png
        metadata.json     # Prompt, params, timestamp
    used/                 # Approved and referenced by game
      player/
        base/             # Body shapes
        hair/             # Hair styles
        face/             # Face variations
        clothing/         # God-affiliated clothing
      enemies/
        minor/
        major/
        boss/
      terrain/
        platforms/
        walls/
        backgrounds/
      weapons/
        sword/
        spear/
      effects/
        hit/
        dash/
      ui/
        health/
        icons/
    alternatives/         # Rejected but kept for reference
      <uuid>/
        sprite.png
        metadata.json

    manifest.json         # Master list of all used assets
    style_guide.json      # Style prompts and palettes
```

## Style System

### Base Style Prompt
Applied to ALL generations for visual coherence:
```
"Clean pixel art style, 64x64 pixels, Pokemon Emerald Gen 3 aesthetic,
limited color palette, crisp edges, no anti-aliasing, game sprite"
```

### God Palettes
```json
{
  "zeus": {
    "primary": "#FFD700",      // Gold
    "secondary": "#00BFFF",    // Electric blue
    "accent": "#FFFFFF",       // White lightning
    "dark": "#1a1a2e"          // Storm dark
  },
  "poseidon": {
    "primary": "#1a3a5c",      // Dark blue
    "secondary": "#000000",    // Black depths
    "accent": "#FFFFFF",       // White foam/waves
    "dark": "#0a1520"          // Abyssal dark
  },
  "ares": {
    "primary": "#8B4513",      // Brown/bronze
    "secondary": "#DC143C",    // Crimson blood
    "accent": "#708090",       // Steel gray
    "dark": "#2a1a0a"          // War dark
  },
  "demeter": {
    "primary": "#90EE90",      // Gentle green
    "secondary": "#87CEEB",    // Frost blue
    "accent": "#F0E68C",       // Gentle yellow
    "dark": "#1a2a1a"          // Earth dark
  }
}
```

## CLI Commands

### `spritegen generate`
Generate a single sprite or sprite sheet.

```bash
spritegen generate \
  --category player|enemy|terrain|weapon|effect|ui \
  --name "idle_base" \
  --prompt "standing humanoid figure, neutral pose, facing right" \
  --god zeus \                    # Optional: apply god palette
  --frames 4 \                    # Optional: sprite sheet frames
  --size 64 \                     # Pixel size (default 64)
  --output staging                # staging (default) | direct
```

Output: Creates `staging/<uuid>/` with sprite and metadata.

### `spritegen batch`
Generate multiple sprites from a manifest.

```bash
spritegen batch manifest.json --auto-apply
```

Manifest format:
```json
{
  "style_override": null,
  "items": [
    {
      "category": "player",
      "subcategory": "base",
      "name": "body_athletic",
      "prompt": "athletic humanoid body, neutral pose",
      "frames": 1
    },
    {
      "category": "player",
      "subcategory": "hair",
      "name": "hair_short",
      "prompt": "short spiky hair, side view",
      "frames": 1
    }
  ]
}
```

With `--auto-apply`: skips staging, writes directly to `used/` and updates manifest.

### `spritegen approve <uuid>`
Move asset from staging to used.

```bash
spritegen approve abc123 --category player --subcategory base --name body_default
```

### `spritegen reject <uuid>`
Move asset from staging to alternatives.

```bash
spritegen reject abc123 --reason "proportions off"
```

### `spritegen list`
List assets by status.

```bash
spritegen list staging           # Show pending review
spritegen list used              # Show approved assets
spritegen list alternatives      # Show rejected alternatives
spritegen list used --category player  # Filter by category
```

### `spritegen apply <asset-path>`
Update Bevy code to reference a new asset. Generates/updates:
- `assets/sprites/manifest.json` (asset registry)
- Relevant Rust const/path references if needed

### `spritegen preview <uuid|path>`
Open asset in system image viewer.

## Sprite Sheet Format

Horizontal strip format for Bevy `TextureAtlas`:
- All frames in single row
- Each frame is `size x size` pixels
- Total image: `(size * frames) x size`

Example: 4-frame 64x64 animation = 256x64 PNG

## Asset Size Standards

Different asset categories have different size requirements based on tile coverage:

| Category | Base Size | Notes |
|----------|-----------|-------|
| `player` | 64x64 | 1 tile, character sprites |
| `enemy.minor` | 64x64 | 1 tile, small enemies |
| `enemy.major` | 128x128 | 2x2 tiles, larger enemies |
| `enemy.boss` | 256x256 | 4x4 tiles, boss sprites |
| `weapon` | 64x64 | Matches character scale |
| `effect` | 64x64 | Hit sparks, dash trails |
| `terrain.platform` | 64x64 | Single tile platforms |
| `terrain.wall` | 64x64 | Single tile walls |
| `terrain.background` | 512x512 | 8x8 tiles, room backgrounds |
| `terrain.tileset` | 256x256 | 4x4 grid of 64x64 tiles |
| `ui` | varies | Icons 32x32, bars 128x16, etc. |

The CLI will apply appropriate defaults based on category.

## Player Character Composition

Layered system with these slots:
1. **Base body** - Body shape/type (3-4 options)
2. **Face** - Face shape/expression (3-4 options)
3. **Hair** - Hairstyle (5-6 options)
4. **Clothing** - God-affiliated outfit (1 per god, upgradeable)

At runtime, Bevy renders layers in order. Color customization (skin tone, hair color) handled via shader or palette swap.

## Metadata Format

Each generated asset has `metadata.json`:
```json
{
  "id": "uuid-here",
  "created_at": "2025-01-19T12:00:00Z",
  "prompt": "full prompt used",
  "style_prompt": "base style applied",
  "god_palette": "zeus",
  "category": "player",
  "subcategory": "base",
  "name": "body_athletic",
  "size": 64,
  "frames": 4,
  "status": "staging|used|alternative",
  "rejection_reason": null
}
```

## Master Manifest

`assets/sprites/manifest.json` tracks all used assets:
```json
{
  "version": 1,
  "assets": {
    "player.base.body_athletic": {
      "path": "sprites/used/player/base/body_athletic.png",
      "frames": 4,
      "size": 64
    },
    "enemy.minor.skeleton_idle": {
      "path": "sprites/used/enemies/minor/skeleton_idle.png",
      "frames": 4,
      "size": 64
    }
  }
}
```

Bevy loads this manifest to resolve sprite paths.

## Claude Code Skill

The `/spritegen` skill provides a conversational interface:

```
User: /spritegen

Claude: What would you like to generate?
- Describe what you need (e.g., "idle animation for a skeleton enemy")
- Or provide a batch manifest
- Or manage existing assets (approve/reject/list)

[Claude decomposes request into CLI calls, manages workflow]
```

The skill handles:
- Breaking down complex requests into atomic sprite generations
- Maintaining style coherence across batch operations
- Presenting staging assets for review
- Applying approved assets to the game

## Integration with Bevy

### Asset Loading
```rust
// In content/mod.rs or similar
#[derive(Resource, Deserialize)]
pub struct SpriteManifest {
    pub version: u32,
    pub assets: HashMap<String, SpriteAssetDef>,
}

#[derive(Deserialize)]
pub struct SpriteAssetDef {
    pub path: String,
    pub frames: u32,
    pub size: u32,
}
```

### Usage
```rust
fn load_player_sprite(
    manifest: Res<SpriteManifest>,
    asset_server: Res<AssetServer>,
) {
    let def = manifest.assets.get("player.base.body_athletic").unwrap();
    let texture = asset_server.load(&def.path);
    // Create TextureAtlas from frames/size...
}
```

## Open Questions

1. Should we support animated GIF output in addition to sprite sheets?
2. Do we need a "regenerate" command that keeps same params but gets new output?
3. Should style guide be editable via CLI or manual JSON editing?

## Next Steps

1. Implement core CLI with generate/list/approve/reject
2. Set up directory structure
3. Create initial style_guide.json with palettes
4. Build Claude Code skill wrapper
5. Test with a simple generation (single enemy sprite)
6. Iterate on style prompt based on results
