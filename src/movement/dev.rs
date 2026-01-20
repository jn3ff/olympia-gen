//! Movement domain: legacy and debug-only spawn helpers.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::{
    AttackState, Combatant, ComboState, Health, Invulnerable, ParryState, PlayerMoveset,
    SkillSlots, Stagger, Team, Weapon,
};
use crate::movement::MovementTuning;
use crate::movement::{GameLayer, Ground, MovementState, Player, Wall, WallJumpLock};

/// Old spawn_player function kept for reference/fallback
#[allow(dead_code)]
pub(crate) fn spawn_player_legacy(mut commands: Commands, tuning: Res<MovementTuning>) {
    commands.spawn((
        // Identity & Movement
        (
            Player,
            Combatant,
            Team::Player,
            MovementState {
                air_jumps_remaining: tuning.max_air_jumps,
                ..default()
            },
            WallJumpLock::default(),
        ),
        // Combat
        (
            Health::new(100.0),
            Stagger::default(),
            Invulnerable::default(),
            Weapon::default(),
            AttackState::default(),
            SkillSlots::default(),
            PlayerMoveset::default(),
            ComboState::default(),
            ParryState::new(0.2),
        ),
        // Rendering
        Sprite {
            color: Color::srgb(0.9, 0.9, 0.9),
            custom_size: Some(Vec2::new(24.0, 48.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 100.0, 0.0),
        // Physics
        (
            RigidBody::Dynamic,
            Collider::rectangle(24.0, 48.0),
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
            GravityScale(0.0), // We handle gravity manually for more control
            Friction::new(0.0),
            CollisionEventsEnabled,
            CollisionLayers::new(
                GameLayer::Player,
                [
                    GameLayer::Ground,
                    GameLayer::Wall,
                    GameLayer::EnemyHitbox,
                    GameLayer::Sensor,
                ],
            ),
        ),
    ));
}

pub(crate) fn spawn_test_room(mut commands: Commands) {
    let wall_color = Color::srgb(0.3, 0.3, 0.4);
    let ground_color = Color::srgb(0.4, 0.5, 0.4);
    let platform_color = Color::srgb(0.5, 0.4, 0.3);

    let ground_layers =
        CollisionLayers::new(GameLayer::Ground, [GameLayer::Player, GameLayer::Enemy]);
    let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Player, GameLayer::Enemy]);

    // Ground
    commands.spawn((
        Ground,
        Sprite {
            color: ground_color,
            custom_size: Some(Vec2::new(800.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -200.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(800.0, 40.0),
        ground_layers,
    ));

    // Left wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 500.0)),
            ..default()
        },
        Transform::from_xyz(-420.0, 50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 500.0),
        wall_layers,
    ));

    // Right wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 500.0)),
            ..default()
        },
        Transform::from_xyz(420.0, 50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 500.0),
        wall_layers,
    ));

    // Platform 1 - left side
    commands.spawn((
        Ground,
        Sprite {
            color: platform_color,
            custom_size: Some(Vec2::new(150.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(-250.0, -50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(150.0, 20.0),
        ground_layers,
    ));

    // Platform 2 - right side, higher
    commands.spawn((
        Ground,
        Sprite {
            color: platform_color,
            custom_size: Some(Vec2::new(150.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(250.0, 50.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(150.0, 20.0),
        ground_layers,
    ));

    // Platform 3 - center, highest
    commands.spawn((
        Ground,
        Sprite {
            color: platform_color,
            custom_size: Some(Vec2::new(120.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 150.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(120.0, 20.0),
        ground_layers,
    ));

    // Small pillar for wall jumping practice
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(30.0, 200.0)),
            ..default()
        },
        Transform::from_xyz(-100.0, -80.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(30.0, 200.0),
        wall_layers,
    ));
}
