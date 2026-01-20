//! Sprite manifest loading and asset management.
//!
//! Loads the sprite manifest JSON which defines all available sprites,
//! their frame counts, sizes, and attachment points.

#![allow(dead_code)]

use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Resource containing all loaded sprite definitions.
#[derive(Resource, Default)]
pub struct SpriteManifest {
    /// Version of the manifest schema.
    pub version: u32,
    /// Map of asset keys to their definitions.
    pub assets: HashMap<String, SpriteAssetDef>,
    /// Loaded texture handles, keyed by asset key.
    pub textures: HashMap<String, Handle<Image>>,
}

/// Definition of a single sprite asset.
#[derive(Debug, Clone, Deserialize)]
pub struct SpriteAssetDef {
    /// Path to the sprite image file, relative to assets/.
    pub path: String,
    /// Number of animation frames (1 for static sprites).
    pub frames: u32,
    /// Size of each frame in pixels.
    pub size: u32,
    /// Weapon anchor point for this frame (for character sprites).
    #[serde(default)]
    pub weapon_anchor: Option<Vec2Def>,
    /// Weapon rotation in degrees for this frame.
    #[serde(default)]
    pub weapon_rotation: Option<f32>,
    /// Hilt attachment point (for weapon sprites).
    #[serde(default)]
    pub hilt_point: Option<Vec2Def>,
}

/// Serializable Vec2 for JSON.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}

impl From<Vec2Def> for Vec2 {
    fn from(v: Vec2Def) -> Self {
        Vec2::new(v.x, v.y)
    }
}

/// Raw manifest JSON structure.
#[derive(Deserialize)]
struct ManifestJson {
    version: u32,
    assets: HashMap<String, SpriteAssetDef>,
}

impl SpriteManifest {
    /// Load the manifest from a JSON file.
    pub fn load_from_file(&mut self, path: &str, asset_server: &AssetServer) {
        let manifest_path = Path::new(path);

        if !manifest_path.exists() {
            warn!(
                "Sprite manifest not found at {:?}, using empty manifest",
                path
            );
            return;
        }

        let contents = match fs::read_to_string(manifest_path) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to read sprite manifest: {}", e);
                return;
            }
        };

        let manifest_json: ManifestJson = match serde_json::from_str(&contents) {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to parse sprite manifest: {}", e);
                return;
            }
        };

        self.version = manifest_json.version;
        self.assets = manifest_json.assets;

        // Preload all textures
        for (key, def) in &self.assets {
            let handle = asset_server.load(&def.path);
            self.textures.insert(key.clone(), handle);
        }

        info!(
            "Loaded sprite manifest v{} with {} assets",
            self.version,
            self.assets.len()
        );
    }

    /// Get a sprite definition by key.
    pub fn get(&self, key: &str) -> Option<&SpriteAssetDef> {
        self.assets.get(key)
    }

    /// Get a texture handle by key.
    pub fn get_texture(&self, key: &str) -> Option<Handle<Image>> {
        self.textures.get(key).cloned()
    }

    /// Get both definition and texture for a sprite.
    pub fn get_sprite(&self, key: &str) -> Option<(&SpriteAssetDef, Handle<Image>)> {
        let def = self.assets.get(key)?;
        let texture = self.textures.get(key)?;
        Some((def, texture.clone()))
    }

    /// Check if a sprite key exists.
    pub fn contains(&self, key: &str) -> bool {
        self.assets.contains_key(key)
    }

    /// Get all keys matching a prefix (e.g., "player.base.zeus_idle").
    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<&String> {
        self.assets
            .keys()
            .filter(|k| k.starts_with(prefix))
            .collect()
    }
}

/// Helper to build sprite asset keys.
pub fn sprite_key(category: &str, subcategory: &str, name: &str) -> String {
    format!("{}.{}.{}", category, subcategory, name)
}

/// Helper for animation frame keys.
pub fn animation_frame_key(base: &str, frame: u32) -> String {
    format!("{}_{}", base, frame)
}
