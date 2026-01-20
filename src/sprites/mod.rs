//! Sprites module for layered sprite rendering and animation.
//!
//! This module handles:
//! - Loading sprite manifests from JSON
//! - Layered sprite compositing (body, armor, weapon layers)
//! - Animation state machines and playback
//! - Weapon attachment with per-frame anchors

pub mod animation;
pub mod layers;
pub mod manifest;
pub mod weapon;

use bevy::prelude::*;

pub use animation::*;
pub use layers::*;
pub use manifest::*;
pub use weapon::*;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpriteManifest>()
            .init_resource::<AnchorDatabase>()
            .add_message::<AnimationStateChanged>()
            .add_message::<AnimationFinished>()
            .add_systems(Startup, load_sprite_manifest)
            .add_systems(
                Update,
                (
                    animation_state_machine,
                    update_animation_frames,
                    update_weapon_attachment,
                    sync_layer_facing,
                ),
            );
    }
}

/// System to load the sprite manifest at startup.
fn load_sprite_manifest(mut manifest: ResMut<SpriteManifest>, asset_server: Res<AssetServer>) {
    manifest.load_from_file("assets/sprites/manifest.json", &asset_server);
}
