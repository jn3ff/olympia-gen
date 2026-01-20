//! Weapon attachment and positioning.
//!
//! Handles positioning weapon sprites relative to character body sprites
//! using anchor points defined in the sprite manifest.

use bevy::prelude::*;

use super::{
    AnimationController, BodyLayer, SpriteLayer, SpriteLayerMarker, SpriteManifest, WeaponClass,
    WeaponLayer,
};

/// Component storing weapon attachment configuration.
#[derive(Component, Debug)]
pub struct WeaponAttachment {
    /// Offset from body anchor to weapon hilt.
    pub offset: Vec2,
    /// Rotation offset in radians.
    pub rotation_offset: f32,
}

impl Default for WeaponAttachment {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            rotation_offset: 0.0,
        }
    }
}

/// System to update weapon position based on body animation frame.
pub fn update_weapon_attachment(
    manifest: Res<SpriteManifest>,
    anchor_db: Res<AnchorDatabase>,
    parent_query: Query<(&AnimationController, &Children)>,
    body_query: Query<&BodyLayer>,
    mut weapon_query: Query<
        (&WeaponLayer, &mut Transform, Option<&WeaponAttachment>),
        With<SpriteLayerMarker>,
    >,
) {
    for (controller, children) in &parent_query {
        // Find body layer to get anchor point
        let mut anchor_data = anchor_db.get_or_default(&controller.current_sprite_key());

        for child in children.iter() {
            if let Ok(body_layer) = body_query.get(child) {
                // Try to get anchor from manifest first
                if let Some(def) = manifest.get(&body_layer.sprite_key) {
                    if let Some(anchor) = &def.weapon_anchor {
                        anchor_data.weapon_anchor = Vec2::new(anchor.x, anchor.y);
                    }
                    if let Some(rot) = def.weapon_rotation {
                        anchor_data.weapon_rotation = rot.to_radians();
                    }
                }
            }
        }

        // Update weapon position
        for child in children.iter() {
            if let Ok((weapon_layer, mut transform, attachment)) = weapon_query.get_mut(child) {
                // Get weapon hilt point from manifest
                let mut hilt_offset = Vec2::new(32.0, 58.0); // Default bottom-center
                if let Some(def) = manifest.get(&weapon_layer.sprite_key) {
                    if let Some(hilt) = &def.hilt_point {
                        hilt_offset = Vec2::new(hilt.x, hilt.y);
                    }
                }

                // Calculate position: anchor - hilt_offset (to align hilt with hand)
                // Convert from sprite coordinates (origin top-left) to Bevy (origin center)
                let sprite_size = 64.0; // TODO: Get from manifest
                let body_anchor_centered =
                    anchor_data.weapon_anchor - Vec2::splat(sprite_size / 2.0);
                let hilt_centered = hilt_offset - Vec2::splat(sprite_size / 2.0);

                let mut position = body_anchor_centered - hilt_centered;

                // Apply attachment offset if present
                if let Some(att) = attachment {
                    position += att.offset;
                }

                // Set transform
                transform.translation.x = position.x;
                transform.translation.y = position.y;
                transform.translation.z = SpriteLayer::Weapon.z_index();

                // Apply rotation
                let total_rotation = anchor_data.weapon_rotation
                    + attachment.map(|a| a.rotation_offset).unwrap_or(0.0);
                transform.rotation = Quat::from_rotation_z(total_rotation);
            }
        }
    }
}

/// Precomputed anchor data for common animations.
/// This can be loaded from a JSON file or computed from sprite analysis.
#[derive(Resource, Default)]
pub struct AnchorDatabase {
    /// Map of sprite_key -> anchor data.
    pub anchors: std::collections::HashMap<String, AnchorData>,
}

/// Anchor data for a single sprite frame.
#[derive(Debug, Clone)]
pub struct AnchorData {
    /// Where the weapon should attach (in sprite pixel coordinates).
    pub weapon_anchor: Vec2,
    /// Rotation of the weapon in radians.
    pub weapon_rotation: f32,
}

impl AnchorDatabase {
    /// Get anchor data for a sprite, falling back to defaults.
    pub fn get_or_default(&self, sprite_key: &str) -> AnchorData {
        self.anchors.get(sprite_key).cloned().unwrap_or_else(|| {
            // Provide sensible defaults based on animation type
            if sprite_key.contains("attack") {
                if sprite_key.contains("_1") {
                    // Wind-up: weapon raised
                    AnchorData {
                        weapon_anchor: Vec2::new(30.0, 20.0),
                        weapon_rotation: -2.4, // ~-135 degrees
                    }
                } else if sprite_key.contains("_2") {
                    // Mid-swing: weapon horizontal
                    AnchorData {
                        weapon_anchor: Vec2::new(50.0, 35.0),
                        weapon_rotation: 0.0,
                    }
                } else {
                    // Follow-through: weapon extended
                    AnchorData {
                        weapon_anchor: Vec2::new(55.0, 45.0),
                        weapon_rotation: 0.8, // ~45 degrees
                    }
                }
            } else {
                // Idle/walk: weapon at side
                AnchorData {
                    weapon_anchor: Vec2::new(45.0, 38.0),
                    weapon_rotation: -0.8, // ~-45 degrees
                }
            }
        })
    }

    /// Load anchor database from JSON file.
    #[allow(unused_variables)]
    pub fn load_from_file(&mut self, path: &str) {
        // TODO: Implement JSON loading
        // For now, populate with defaults
        warn!("Anchor database loading not yet implemented, using defaults");
    }
}

/// Helper to equip a new weapon on a character.
pub fn equip_weapon(
    commands: &mut Commands,
    manifest: &SpriteManifest,
    character_entity: Entity,
    weapon_base_key: &str,
    children: &Children,
    weapon_query: &Query<Entity, With<WeaponLayer>>,
) {
    // Remove existing weapon layer
    for child in children.iter() {
        if weapon_query.get(child).is_ok() {
            commands.entity(child).despawn();
        }
    }

    // Spawn new weapon layer
    let weapon_idle_key = format!("{}.idle", weapon_base_key);
    if let Some(texture) = manifest.get_texture(&weapon_idle_key) {
        let weapon_def = manifest.get(&weapon_idle_key);
        let weapon_size = weapon_def.map(|d| d.size as f32).unwrap_or(64.0);

        let weapon_child = commands
            .spawn((
                SpriteLayerMarker {
                    layer: SpriteLayer::Weapon,
                },
                Sprite {
                    image: texture,
                    custom_size: Some(Vec2::splat(weapon_size)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, SpriteLayer::Weapon.z_index()),
                WeaponLayer {
                    sprite_key: weapon_idle_key,
                    weapon_class: WeaponClass::Sword,
                },
                WeaponAttachment::default(),
            ))
            .id();

        commands.entity(character_entity).add_child(weapon_child);
    }
}
