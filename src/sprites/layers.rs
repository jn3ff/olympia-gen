//! Layered sprite components and bundles.
//!
//! Provides a hierarchical sprite system where characters are composed of
//! multiple layers (body, armor, weapon) that can be individually swapped.

use bevy::prelude::*;

/// Marker component for the root of a layered sprite entity.
#[derive(Component, Debug)]
pub struct LayeredSprite {
    /// Whether the sprite is facing right (false = facing left).
    pub facing_right: bool,
}

impl Default for LayeredSprite {
    fn default() -> Self {
        Self { facing_right: true }
    }
}

/// Defines the render order for sprite layers.
/// Lower values render behind higher values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpriteLayer {
    /// Back effects (shadows, auras behind character).
    BackEffect = 0,
    /// Character body (base layer).
    Body = 10,
    /// Armor/clothing layer.
    Armor = 20,
    /// Weapon layer.
    Weapon = 30,
    /// Front arm (for attacks where arm should be in front of weapon).
    FrontArm = 40,
    /// Front effects (hit sparks, buffs, status icons).
    FrontEffect = 50,
}

impl SpriteLayer {
    /// Convert to Z coordinate for 2D ordering.
    pub fn z_index(&self) -> f32 {
        (*self as i32) as f32 * 0.01
    }
}

/// Component marking a sprite as a specific layer.
#[derive(Component, Debug)]
pub struct SpriteLayerMarker {
    pub layer: SpriteLayer,
}

/// Component for body layer sprites.
#[derive(Component, Debug, Default)]
pub struct BodyLayer {
    /// Current sprite key (e.g., "player.base.zeus_idle_1").
    pub sprite_key: String,
}

/// Component for armor/clothing layer sprites.
#[derive(Component, Debug, Default)]
pub struct ArmorLayer {
    /// Current sprite key.
    pub sprite_key: String,
}

/// Component for weapon layer sprites.
#[derive(Component, Debug, Default)]
pub struct WeaponLayer {
    /// Current weapon sprite key (e.g., "weapon.sword.basic.idle").
    pub sprite_key: String,
    /// Whether this is a sword-class or spear-class weapon.
    pub weapon_class: WeaponClass,
}

/// Weapon class for animation matching.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WeaponClass {
    #[default]
    Sword,
    Spear,
}

/// Component for effect layers (hit sparks, buffs, etc.).
#[derive(Component, Debug)]
pub struct EffectLayer {
    /// Current effect sprite key.
    pub sprite_key: Option<String>,
    /// Time remaining for temporary effects.
    pub lifetime: Option<f32>,
}

/// System to sync layer facing direction with parent.
pub fn sync_layer_facing(
    parent_query: Query<(&LayeredSprite, &Children), Changed<LayeredSprite>>,
    mut child_query: Query<&mut Sprite, With<SpriteLayerMarker>>,
) {
    for (layered_sprite, children) in &parent_query {
        for child in children.iter() {
            if let Ok(mut sprite) = child_query.get_mut(child) {
                sprite.flip_x = !layered_sprite.facing_right;
            }
        }
    }
}

/// Helper to spawn a complete layered character sprite.
pub fn spawn_layered_character(
    commands: &mut Commands,
    manifest: &super::SpriteManifest,
    body_key: &str,
    weapon_key: Option<&str>,
    position: Vec3,
) -> Entity {
    let body_texture = manifest.get_texture(body_key).unwrap_or_default();

    let body_def = manifest.get(body_key);
    let body_size = body_def.map(|d| d.size as f32).unwrap_or(64.0);

    let parent = commands
        .spawn((
            LayeredSprite::default(),
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .id();

    // Spawn body layer
    let body_child = commands
        .spawn((
            SpriteLayerMarker {
                layer: SpriteLayer::Body,
            },
            Sprite {
                image: body_texture,
                custom_size: Some(Vec2::splat(body_size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, SpriteLayer::Body.z_index()),
            BodyLayer {
                sprite_key: body_key.to_string(),
            },
        ))
        .id();

    commands.entity(parent).add_child(body_child);

    // Spawn weapon layer if provided
    if let Some(wkey) = weapon_key {
        let weapon_texture = manifest.get_texture(wkey).unwrap_or_default();
        let weapon_def = manifest.get(wkey);
        let weapon_size = weapon_def.map(|d| d.size as f32).unwrap_or(64.0);

        let weapon_child = commands
            .spawn((
                SpriteLayerMarker {
                    layer: SpriteLayer::Weapon,
                },
                Sprite {
                    image: weapon_texture,
                    custom_size: Some(Vec2::splat(weapon_size)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, SpriteLayer::Weapon.z_index()),
                WeaponLayer {
                    sprite_key: wkey.to_string(),
                    weapon_class: WeaponClass::Sword,
                },
            ))
            .id();

        commands.entity(parent).add_child(weapon_child);
    }

    parent
}

/// Helper to add a layer to an existing layered sprite.
pub fn add_sprite_layer(
    commands: &mut Commands,
    parent: Entity,
    layer: SpriteLayer,
    texture: Handle<Image>,
    size: f32,
) -> Entity {
    let child = commands
        .spawn((
            SpriteLayerMarker { layer },
            Sprite {
                image: texture,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, layer.z_index()),
        ))
        .id();

    commands.entity(parent).add_child(child);
    child
}
