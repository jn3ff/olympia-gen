#!/usr/bin/env python3
"""
Spritegen - Game sprite generation CLI using Gemini API.

Generates pixel art sprites for Olympia game with consistent style
and god-specific color palettes.

Usage:
    spritegen generate --category player --name idle --prompt "standing hero"
    spritegen batch manifest.json --auto-apply
    spritegen list staging
    spritegen approve <uuid> --category player --subcategory base --name body
    spritegen reject <uuid> --reason "proportions off"
    spritegen apply <path>
    spritegen preview <uuid>
"""

import argparse
import json
import os
import sys
import uuid
from datetime import datetime
from pathlib import Path
from typing import Optional

from google import genai
from google.genai import types


# === Configuration ===

SPRITES_ROOT = Path(__file__).parent.parent.parent / "assets" / "sprites"
STYLE_GUIDE_PATH = SPRITES_ROOT / "style_guide.json"
MANIFEST_PATH = SPRITES_ROOT / "manifest.json"

# Default style prompt - minimal anchors for cohesion
DEFAULT_STYLE_PROMPT = """Retro video game pixel art sprite (NOT digital illustration), black 1px outlines, light source from upper-left (highlights on upper-left edges, shadows on lower-right), medium saturation colors (vibrant but earthy, not neon), crisp pixelated edges, no anti-aliasing, no smooth gradients"""

DEFAULT_AESTHETIC = """Mythic Greek world with weathered, lived-in feel, mix of human, divine, and monstrous"""

# Size defaults by category
SIZE_DEFAULTS = {
    "player": 64,
    "enemy.minor": 64,
    "enemy.major": 128,
    "enemy.boss": 256,
    "weapon": 64,
    "effect": 64,
    "terrain.platform": 64,
    "terrain.wall": 64,
    "terrain.background": 512,
    "terrain.tileset": 256,
    "ui.icon": 32,
    "ui.bar": 128,
    "ui": 64,
}

# God color palettes
GOD_PALETTES = {
    "zeus": {
        "primary": "#FFD700",      # Gold
        "secondary": "#00BFFF",    # Electric blue
        "accent": "#FFFFFF",       # White lightning
        "dark": "#1a1a2e",         # Storm dark
        "description": "gold, electric blue, white, dark storm colors"
    },
    "poseidon": {
        "primary": "#1a3a5c",      # Dark blue
        "secondary": "#000000",    # Black depths
        "accent": "#FFFFFF",       # White foam/waves
        "dark": "#0a1520",         # Abyssal dark
        "description": "dark blue, black, white foam colors"
    },
    "ares": {
        "primary": "#8B4513",      # Brown/bronze
        "secondary": "#DC143C",    # Crimson blood
        "accent": "#708090",       # Steel gray
        "dark": "#2a1a0a",         # War dark
        "description": "brown, crimson, steel gray colors"
    },
    "demeter": {
        "primary": "#90EE90",      # Gentle green
        "secondary": "#87CEEB",    # Frost blue
        "accent": "#F0E68C",       # Gentle yellow
        "dark": "#1a2a1a",         # Earth dark
        "description": "gentle green, frost blue, gentle yellow colors"
    },
}


# === Utility Functions ===

def get_api_client() -> genai.Client:
    """Initialize Gemini API client."""
    # Check environment variables (multiple names supported)
    api_key = os.environ.get("GEMINI_API_KEY") or os.environ.get("SPRITEGEN_API_KEY")

    if not api_key:
        # Try loading from .env file
        env_path = Path(__file__).parent / ".env"
        if env_path.exists():
            with open(env_path) as f:
                for line in f:
                    line = line.strip()
                    if line.startswith("GEMINI_API_KEY=") or line.startswith("SPRITEGEN_API_KEY="):
                        api_key = line.split("=", 1)[1].strip('"\'')
                        break

    if not api_key:
        print("Error: GEMINI_API_KEY or SPRITEGEN_API_KEY not found in environment or .env file")
        sys.exit(1)

    return genai.Client(api_key=api_key)


def ensure_directories():
    """Create sprite directory structure if it doesn't exist."""
    dirs = [
        SPRITES_ROOT / "staging",
        SPRITES_ROOT / "used" / "player" / "base",
        SPRITES_ROOT / "used" / "player" / "hair",
        SPRITES_ROOT / "used" / "player" / "face",
        SPRITES_ROOT / "used" / "player" / "clothing",
        SPRITES_ROOT / "used" / "enemies" / "minor",
        SPRITES_ROOT / "used" / "enemies" / "major",
        SPRITES_ROOT / "used" / "enemies" / "boss",
        SPRITES_ROOT / "used" / "terrain" / "platforms",
        SPRITES_ROOT / "used" / "terrain" / "walls",
        SPRITES_ROOT / "used" / "terrain" / "backgrounds",
        SPRITES_ROOT / "used" / "terrain" / "tilesets",
        SPRITES_ROOT / "used" / "weapons" / "sword",
        SPRITES_ROOT / "used" / "weapons" / "spear",
        SPRITES_ROOT / "used" / "effects" / "hit",
        SPRITES_ROOT / "used" / "effects" / "dash",
        SPRITES_ROOT / "used" / "ui" / "health",
        SPRITES_ROOT / "used" / "ui" / "icons",
        SPRITES_ROOT / "alternatives",
    ]
    for d in dirs:
        d.mkdir(parents=True, exist_ok=True)


def load_style_guide() -> dict:
    """Load style guide configuration."""
    if STYLE_GUIDE_PATH.exists():
        with open(STYLE_GUIDE_PATH) as f:
            return json.load(f)
    return {
        "base_style": DEFAULT_STYLE_PROMPT,
        "aesthetic": DEFAULT_AESTHETIC,
        "palettes": GOD_PALETTES,
    }


def save_style_guide(guide: dict):
    """Save style guide configuration."""
    with open(STYLE_GUIDE_PATH, "w") as f:
        json.dump(guide, f, indent=2)


def load_manifest() -> dict:
    """Load asset manifest."""
    if MANIFEST_PATH.exists():
        with open(MANIFEST_PATH) as f:
            return json.load(f)
    return {"version": 1, "assets": {}}


def save_manifest(manifest: dict):
    """Save asset manifest."""
    with open(MANIFEST_PATH, "w") as f:
        json.dump(manifest, f, indent=2)


def get_size_for_category(category: str, subcategory: Optional[str] = None) -> int:
    """Get default size for a category."""
    if subcategory:
        key = f"{category}.{subcategory}"
        if key in SIZE_DEFAULTS:
            return SIZE_DEFAULTS[key]
    return SIZE_DEFAULTS.get(category, 64)


def build_prompt(
    base_prompt: str,
    category: str,
    size: int,
    frames: int,
    god: Optional[str] = None,
    style_guide: Optional[dict] = None,
) -> str:
    """Build full generation prompt with style and constraints."""
    style = style_guide.get("base_style", DEFAULT_STYLE_PROMPT) if style_guide else DEFAULT_STYLE_PROMPT
    aesthetic = style_guide.get("aesthetic", DEFAULT_AESTHETIC) if style_guide else DEFAULT_AESTHETIC

    parts = [style]

    # Add size constraint
    if frames > 1:
        total_width = size * frames
        parts.append(f"sprite sheet with {frames} frames arranged horizontally, total image size {total_width}x{size} pixels, each frame {size}x{size} pixels")
    else:
        parts.append(f"single sprite, {size}x{size} pixels")

    # Add aesthetic context
    parts.append(aesthetic)

    # Add god palette if specified
    if god and god in GOD_PALETTES:
        palette = GOD_PALETTES[god]
        parts.append(f"using {palette['description']} color palette")

    # Add the actual content prompt
    parts.append(base_prompt)

    return ", ".join(parts)


# === Commands ===

def cmd_generate(args):
    """Generate a sprite or sprite sheet."""
    ensure_directories()
    style_guide = load_style_guide()

    # Determine size
    size = args.size
    if size is None:
        size = get_size_for_category(args.category, args.subcategory)

    # Build prompt
    full_prompt = build_prompt(
        args.prompt,
        args.category,
        size,
        args.frames,
        args.god,
        style_guide,
    )

    print(f"Generating sprite...")
    print(f"  Category: {args.category}")
    if args.subcategory:
        print(f"  Subcategory: {args.subcategory}")
    print(f"  Name: {args.name}")
    print(f"  Size: {size}x{size}")
    print(f"  Frames: {args.frames}")
    if args.god:
        print(f"  God palette: {args.god}")
    if hasattr(args, 'reference') and args.reference:
        print(f"  Reference: {args.reference}")
    print(f"  Prompt: {full_prompt[:100]}...")
    print()

    # Generate with Gemini
    client = get_api_client()

    # Build content parts
    parts = []

    # Add reference image if provided
    if hasattr(args, 'reference') and args.reference:
        ref_path = Path(args.reference)
        if ref_path.exists():
            with open(ref_path, "rb") as f:
                image_data = f.read()
            import base64
            # Determine mime type
            suffix = ref_path.suffix.lower()
            mime_type = {"png": "image/png", "jpg": "image/jpeg", "jpeg": "image/jpeg"}.get(suffix[1:], "image/png")
            parts.append(types.Part.from_bytes(data=image_data, mime_type=mime_type))
            parts.append(types.Part.from_text(text=f"Using the above image as a style reference, generate in the EXACT same pixel art style: {full_prompt}"))
        else:
            print(f"Warning: Reference image not found: {ref_path}")
            parts.append(types.Part.from_text(text=full_prompt))
    else:
        parts.append(types.Part.from_text(text=full_prompt))

    contents = [
        types.Content(
            role="user",
            parts=parts,
        ),
    ]

    config = types.GenerateContentConfig(
        response_modalities=["IMAGE", "TEXT"],
    )

    # Generate unique ID for this asset
    asset_id = str(uuid.uuid4())[:8]

    # Determine output location
    if args.output == "direct" or args.auto_apply:
        # Direct to used folder
        if args.subcategory:
            output_dir = SPRITES_ROOT / "used" / args.category / args.subcategory
        else:
            output_dir = SPRITES_ROOT / "used" / args.category
        output_dir.mkdir(parents=True, exist_ok=True)
        output_path = output_dir / f"{args.name}.png"
        metadata_path = output_dir / f"{args.name}.meta.json"
        status = "used"
    else:
        # Staging
        staging_dir = SPRITES_ROOT / "staging" / asset_id
        staging_dir.mkdir(parents=True, exist_ok=True)
        output_path = staging_dir / "sprite.png"
        metadata_path = staging_dir / "metadata.json"
        status = "staging"

    # Call API
    try:
        image_saved = False
        for chunk in client.models.generate_content_stream(
            model="gemini-2.0-flash-exp",
            contents=contents,
            config=config,
        ):
            if (
                chunk.candidates is None
                or chunk.candidates[0].content is None
                or chunk.candidates[0].content.parts is None
            ):
                continue

            part = chunk.candidates[0].content.parts[0]
            if part.inline_data and part.inline_data.data:
                # Save image
                with open(output_path, "wb") as f:
                    f.write(part.inline_data.data)
                image_saved = True
                print(f"Saved sprite to: {output_path}")
            elif hasattr(part, 'text') and part.text:
                print(f"API note: {part.text}")

        if not image_saved:
            print("Error: No image was generated")
            return 1

    except Exception as e:
        print(f"Error generating image: {e}")
        return 1

    # Save metadata
    metadata = {
        "id": asset_id,
        "created_at": datetime.now().isoformat(),
        "prompt": args.prompt,
        "full_prompt": full_prompt,
        "god_palette": args.god,
        "category": args.category,
        "subcategory": args.subcategory,
        "name": args.name,
        "size": size,
        "frames": args.frames,
        "status": status,
    }

    with open(metadata_path, "w") as f:
        json.dump(metadata, f, indent=2)

    # Update manifest if direct/auto-apply
    if status == "used":
        manifest = load_manifest()
        asset_key = f"{args.category}.{args.subcategory}.{args.name}" if args.subcategory else f"{args.category}.{args.name}"
        manifest["assets"][asset_key] = {
            "path": str(output_path.relative_to(SPRITES_ROOT.parent.parent)),
            "frames": args.frames,
            "size": size,
        }
        save_manifest(manifest)
        print(f"Updated manifest with key: {asset_key}")
    else:
        print(f"\nAsset staged with ID: {asset_id}")
        print(f"To approve: spritegen approve {asset_id} --category {args.category} --name {args.name}")

    return 0


def cmd_batch(args):
    """Generate multiple sprites from a manifest file."""
    manifest_file = Path(args.manifest)
    if not manifest_file.exists():
        print(f"Error: Manifest file not found: {manifest_file}")
        return 1

    with open(manifest_file) as f:
        batch_manifest = json.load(f)

    items = batch_manifest.get("items", [])
    print(f"Processing {len(items)} items...")

    for i, item in enumerate(items, 1):
        print(f"\n[{i}/{len(items)}] Generating {item.get('name', 'unnamed')}...")

        # Build args-like object for cmd_generate
        class Args:
            pass

        gen_args = Args()
        gen_args.category = item.get("category", "player")
        gen_args.subcategory = item.get("subcategory")
        gen_args.name = item.get("name", f"sprite_{i}")
        gen_args.prompt = item.get("prompt", "")
        gen_args.god = item.get("god")
        gen_args.frames = item.get("frames", 1)
        gen_args.size = item.get("size")
        gen_args.output = "direct" if args.auto_apply else "staging"
        gen_args.auto_apply = args.auto_apply

        result = cmd_generate(gen_args)
        if result != 0:
            print(f"Warning: Failed to generate {item.get('name')}")

    print(f"\nBatch complete. Generated {len(items)} sprites.")
    return 0


def cmd_list(args):
    """List assets by status."""
    ensure_directories()

    if args.status == "staging":
        staging_dir = SPRITES_ROOT / "staging"
        if not staging_dir.exists():
            print("No staging assets.")
            return 0

        items = list(staging_dir.iterdir())
        if not items:
            print("No staging assets.")
            return 0

        print(f"Staging assets ({len(items)}):\n")
        for item in sorted(items):
            if item.is_dir():
                meta_path = item / "metadata.json"
                if meta_path.exists():
                    with open(meta_path) as f:
                        meta = json.load(f)
                    print(f"  {item.name}")
                    print(f"    Category: {meta.get('category')}")
                    print(f"    Name: {meta.get('name')}")
                    print(f"    Prompt: {meta.get('prompt', '')[:50]}...")
                    print()

    elif args.status == "used":
        manifest = load_manifest()
        assets = manifest.get("assets", {})

        if args.category:
            assets = {k: v for k, v in assets.items() if k.startswith(args.category)}

        if not assets:
            print("No used assets" + (f" in category '{args.category}'" if args.category else "") + ".")
            return 0

        print(f"Used assets ({len(assets)}):\n")
        for key, info in sorted(assets.items()):
            print(f"  {key}")
            print(f"    Path: {info.get('path')}")
            print(f"    Size: {info.get('size')}x{info.get('size')}")
            print(f"    Frames: {info.get('frames')}")
            print()

    elif args.status == "alternatives":
        alt_dir = SPRITES_ROOT / "alternatives"
        if not alt_dir.exists():
            print("No alternative assets.")
            return 0

        items = list(alt_dir.iterdir())
        if not items:
            print("No alternative assets.")
            return 0

        print(f"Alternative assets ({len(items)}):\n")
        for item in sorted(items):
            if item.is_dir():
                meta_path = item / "metadata.json"
                if meta_path.exists():
                    with open(meta_path) as f:
                        meta = json.load(f)
                    print(f"  {item.name}")
                    print(f"    Original: {meta.get('category')}/{meta.get('name')}")
                    print(f"    Reason: {meta.get('rejection_reason', 'N/A')}")
                    print()

    return 0


def cmd_approve(args):
    """Move asset from staging to used."""
    ensure_directories()

    staging_dir = SPRITES_ROOT / "staging" / args.uuid
    if not staging_dir.exists():
        print(f"Error: Staging asset not found: {args.uuid}")
        return 1

    # Load metadata
    meta_path = staging_dir / "metadata.json"
    if not meta_path.exists():
        print(f"Error: Metadata not found for {args.uuid}")
        return 1

    with open(meta_path) as f:
        meta = json.load(f)

    # Use provided category/name or fall back to metadata
    category = args.category or meta.get("category")
    subcategory = args.subcategory or meta.get("subcategory")
    name = args.name or meta.get("name")

    if not category or not name:
        print("Error: Must specify --category and --name, or they must be in metadata")
        return 1

    # Determine destination
    if subcategory:
        dest_dir = SPRITES_ROOT / "used" / category / subcategory
    else:
        dest_dir = SPRITES_ROOT / "used" / category
    dest_dir.mkdir(parents=True, exist_ok=True)

    # Move files
    src_sprite = staging_dir / "sprite.png"
    dest_sprite = dest_dir / f"{name}.png"
    dest_meta = dest_dir / f"{name}.meta.json"

    import shutil
    shutil.copy2(src_sprite, dest_sprite)

    # Update metadata
    meta["status"] = "used"
    meta["approved_at"] = datetime.now().isoformat()
    with open(dest_meta, "w") as f:
        json.dump(meta, f, indent=2)

    # Update manifest
    manifest = load_manifest()
    asset_key = f"{category}.{subcategory}.{name}" if subcategory else f"{category}.{name}"
    manifest["assets"][asset_key] = {
        "path": str(dest_sprite.relative_to(SPRITES_ROOT.parent.parent)),
        "frames": meta.get("frames", 1),
        "size": meta.get("size", 64),
    }
    save_manifest(manifest)

    # Remove staging directory
    shutil.rmtree(staging_dir)

    print(f"Approved {args.uuid} -> {asset_key}")
    print(f"Saved to: {dest_sprite}")
    return 0


def cmd_reject(args):
    """Move asset from staging to alternatives."""
    ensure_directories()

    staging_dir = SPRITES_ROOT / "staging" / args.uuid
    if not staging_dir.exists():
        print(f"Error: Staging asset not found: {args.uuid}")
        return 1

    # Move to alternatives
    alt_dir = SPRITES_ROOT / "alternatives" / args.uuid

    import shutil
    shutil.move(str(staging_dir), str(alt_dir))

    # Update metadata with rejection reason
    meta_path = alt_dir / "metadata.json"
    if meta_path.exists():
        with open(meta_path) as f:
            meta = json.load(f)
        meta["status"] = "rejected"
        meta["rejection_reason"] = args.reason
        meta["rejected_at"] = datetime.now().isoformat()
        with open(meta_path, "w") as f:
            json.dump(meta, f, indent=2)

    print(f"Rejected {args.uuid}")
    if args.reason:
        print(f"Reason: {args.reason}")
    return 0


def cmd_apply(args):
    """Apply an asset (add to manifest if not already there)."""
    asset_path = Path(args.path)
    if not asset_path.exists():
        print(f"Error: Asset not found: {asset_path}")
        return 1

    # Try to load associated metadata
    meta_path = asset_path.with_suffix(".meta.json")
    if meta_path.exists():
        with open(meta_path) as f:
            meta = json.load(f)
    else:
        meta = {}

    # Build asset key from path or metadata
    if meta:
        category = meta.get("category", "unknown")
        subcategory = meta.get("subcategory")
        name = meta.get("name", asset_path.stem)
        asset_key = f"{category}.{subcategory}.{name}" if subcategory else f"{category}.{name}"
    else:
        # Infer from path
        parts = asset_path.relative_to(SPRITES_ROOT / "used").parts
        asset_key = ".".join(parts[:-1]) + "." + asset_path.stem

    # Update manifest
    manifest = load_manifest()
    manifest["assets"][asset_key] = {
        "path": str(asset_path.relative_to(SPRITES_ROOT.parent.parent)),
        "frames": meta.get("frames", 1),
        "size": meta.get("size", 64),
    }
    save_manifest(manifest)

    print(f"Applied {asset_key} to manifest")
    return 0


def cmd_preview(args):
    """Open asset in system viewer."""
    # Try to find the asset
    path = None

    # Check if it's a UUID (staging)
    staging_path = SPRITES_ROOT / "staging" / args.id / "sprite.png"
    if staging_path.exists():
        path = staging_path

    # Check alternatives
    if not path:
        alt_path = SPRITES_ROOT / "alternatives" / args.id / "sprite.png"
        if alt_path.exists():
            path = alt_path

    # Check if it's a direct path
    if not path:
        direct_path = Path(args.id)
        if direct_path.exists():
            path = direct_path

    # Check if it's an asset key in manifest
    if not path:
        manifest = load_manifest()
        if args.id in manifest.get("assets", {}):
            rel_path = manifest["assets"][args.id]["path"]
            path = SPRITES_ROOT.parent.parent / rel_path

    if not path or not path.exists():
        print(f"Error: Could not find asset: {args.id}")
        return 1

    # Open with system viewer
    import subprocess
    import platform

    system = platform.system()
    if system == "Darwin":
        subprocess.run(["open", str(path)])
    elif system == "Linux":
        subprocess.run(["xdg-open", str(path)])
    elif system == "Windows":
        subprocess.run(["start", str(path)], shell=True)
    else:
        print(f"Asset path: {path}")

    return 0


def cmd_init(args):
    """Initialize sprite directory structure and style guide."""
    ensure_directories()

    # Create style guide if it doesn't exist
    if not STYLE_GUIDE_PATH.exists():
        style_guide = {
            "base_style": DEFAULT_STYLE_PROMPT,
            "palettes": GOD_PALETTES,
        }
        save_style_guide(style_guide)
        print(f"Created style guide: {STYLE_GUIDE_PATH}")

    # Create manifest if it doesn't exist
    if not MANIFEST_PATH.exists():
        manifest = {"version": 1, "assets": {}}
        save_manifest(manifest)
        print(f"Created manifest: {MANIFEST_PATH}")

    print("Sprite directories initialized.")
    return 0


# === Main ===

def main():
    parser = argparse.ArgumentParser(
        description="Spritegen - Game sprite generation CLI",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    subparsers = parser.add_subparsers(dest="command", help="Commands")

    # init
    init_parser = subparsers.add_parser("init", help="Initialize directories and config")
    init_parser.set_defaults(func=cmd_init)

    # generate
    gen_parser = subparsers.add_parser("generate", help="Generate a sprite")
    gen_parser.add_argument("--category", "-c", required=True,
                           choices=["player", "enemy", "terrain", "weapon", "effect", "ui"],
                           help="Asset category")
    gen_parser.add_argument("--subcategory", "-s", help="Asset subcategory (e.g., minor, major, boss)")
    gen_parser.add_argument("--name", "-n", required=True, help="Asset name")
    gen_parser.add_argument("--prompt", "-p", required=True, help="Generation prompt")
    gen_parser.add_argument("--god", "-g", choices=["zeus", "poseidon", "ares", "demeter"],
                           help="Apply god color palette")
    gen_parser.add_argument("--frames", "-f", type=int, default=1, help="Number of animation frames")
    gen_parser.add_argument("--size", type=int, help="Sprite size (default based on category)")
    gen_parser.add_argument("--output", "-o", choices=["staging", "direct"], default="staging",
                           help="Output location")
    gen_parser.add_argument("--auto-apply", action="store_true", help="Skip staging, apply directly")
    gen_parser.add_argument("--reference", "-r", help="Reference image path for style matching")
    gen_parser.set_defaults(func=cmd_generate)

    # batch
    batch_parser = subparsers.add_parser("batch", help="Generate from manifest")
    batch_parser.add_argument("manifest", help="Path to batch manifest JSON")
    batch_parser.add_argument("--auto-apply", action="store_true", help="Skip staging, apply directly")
    batch_parser.set_defaults(func=cmd_batch)

    # list
    list_parser = subparsers.add_parser("list", help="List assets")
    list_parser.add_argument("status", choices=["staging", "used", "alternatives"],
                            help="Asset status to list")
    list_parser.add_argument("--category", "-c", help="Filter by category")
    list_parser.set_defaults(func=cmd_list)

    # approve
    approve_parser = subparsers.add_parser("approve", help="Approve staging asset")
    approve_parser.add_argument("uuid", help="Staging asset UUID")
    approve_parser.add_argument("--category", "-c", help="Override category")
    approve_parser.add_argument("--subcategory", "-s", help="Override subcategory")
    approve_parser.add_argument("--name", "-n", help="Override name")
    approve_parser.set_defaults(func=cmd_approve)

    # reject
    reject_parser = subparsers.add_parser("reject", help="Reject staging asset")
    reject_parser.add_argument("uuid", help="Staging asset UUID")
    reject_parser.add_argument("--reason", "-r", help="Rejection reason")
    reject_parser.set_defaults(func=cmd_reject)

    # apply
    apply_parser = subparsers.add_parser("apply", help="Apply asset to manifest")
    apply_parser.add_argument("path", help="Path to asset file")
    apply_parser.set_defaults(func=cmd_apply)

    # preview
    preview_parser = subparsers.add_parser("preview", help="Preview asset")
    preview_parser.add_argument("id", help="Asset UUID, path, or manifest key")
    preview_parser.set_defaults(func=cmd_preview)

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 1

    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
