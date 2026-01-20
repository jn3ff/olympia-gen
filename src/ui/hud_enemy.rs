//! UI domain: world-space enemy health bars.

use bevy::prelude::*;

use crate::combat::{BossAI, Enemy, Health};

const ENEMY_HEALTHBAR_WIDTH: f32 = 40.0;
const ENEMY_HEALTHBAR_HEIGHT: f32 = 6.0;
const ENEMY_HEALTHBAR_OFFSET_Y: f32 = 25.0;

/// Marker for enemy world-space health bars
#[derive(Component)]
pub struct EnemyHealthBar {
    pub owner: Entity,
}

/// Marker for the fill portion of enemy health bars
#[derive(Component)]
pub struct EnemyHealthBarFill {
    pub owner: Entity,
}

pub(crate) fn spawn_enemy_healthbars(
    mut commands: Commands,
    enemy_query: Query<Entity, (With<Enemy>, Without<BossAI>, Added<Enemy>)>,
    existing_bars: Query<&EnemyHealthBar>,
) {
    for enemy_entity in &enemy_query {
        // Check if this enemy already has a health bar
        let has_bar = existing_bars.iter().any(|bar| bar.owner == enemy_entity);
        if has_bar {
            continue;
        }

        // Spawn the health bar as a world-space sprite
        // Background (dark)
        commands.spawn((
            EnemyHealthBar {
                owner: enemy_entity,
            },
            Sprite {
                color: Color::srgba(0.1, 0.1, 0.1, 0.8),
                custom_size: Some(Vec2::new(
                    ENEMY_HEALTHBAR_WIDTH + 2.0,
                    ENEMY_HEALTHBAR_HEIGHT + 2.0,
                )),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 5.0),
        ));

        // Foreground (health fill)
        commands.spawn((
            EnemyHealthBarFill {
                owner: enemy_entity,
            },
            Sprite {
                color: Color::srgb(0.8, 0.2, 0.2),
                custom_size: Some(Vec2::new(ENEMY_HEALTHBAR_WIDTH, ENEMY_HEALTHBAR_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 6.0),
        ));
    }
}

pub(crate) fn update_enemy_healthbars(
    enemy_query: Query<(&Transform, &Health, &Sprite), (With<Enemy>, Without<BossAI>)>,
    mut bar_query: Query<(&EnemyHealthBar, &mut Transform), Without<Enemy>>,
    mut fill_query: Query<
        (&EnemyHealthBarFill, &mut Transform, &mut Sprite),
        (Without<Enemy>, Without<EnemyHealthBar>),
    >,
) {
    // Update background bar positions
    for (bar, mut bar_transform) in &mut bar_query {
        if let Ok((enemy_transform, _health, enemy_sprite)) = enemy_query.get(bar.owner) {
            let enemy_height = enemy_sprite.custom_size.map(|s| s.y).unwrap_or(32.0);
            bar_transform.translation.x = enemy_transform.translation.x;
            bar_transform.translation.y =
                enemy_transform.translation.y + enemy_height / 2.0 + ENEMY_HEALTHBAR_OFFSET_Y;
        }
    }

    // Update fill bars
    for (fill, mut fill_transform, mut fill_sprite) in &mut fill_query {
        if let Ok((enemy_transform, health, enemy_sprite)) = enemy_query.get(fill.owner) {
            let enemy_height = enemy_sprite.custom_size.map(|s| s.y).unwrap_or(32.0);
            let percent = health.percent();
            let fill_width = ENEMY_HEALTHBAR_WIDTH * percent;

            // Position the fill bar (left-aligned within the background)
            fill_transform.translation.x =
                enemy_transform.translation.x - (ENEMY_HEALTHBAR_WIDTH - fill_width) / 2.0;
            fill_transform.translation.y =
                enemy_transform.translation.y + enemy_height / 2.0 + ENEMY_HEALTHBAR_OFFSET_Y;

            // Update size
            fill_sprite.custom_size = Some(Vec2::new(fill_width.max(0.0), ENEMY_HEALTHBAR_HEIGHT));

            // Update color based on health percent
            let color = if percent > 0.5 {
                Color::srgb(0.8, 0.2, 0.2)
            } else if percent > 0.25 {
                Color::srgb(0.9, 0.5, 0.1)
            } else {
                Color::srgb(0.6, 0.1, 0.1)
            };
            fill_sprite.color = color;
        }
    }
}

pub(crate) fn cleanup_enemy_healthbars(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    bar_query: Query<(Entity, &EnemyHealthBar)>,
    fill_query: Query<(Entity, &EnemyHealthBarFill)>,
) {
    // Clean up bars for dead enemies
    for (bar_entity, bar) in &bar_query {
        if enemy_query.get(bar.owner).is_err() {
            commands.entity(bar_entity).despawn();
        }
    }

    for (fill_entity, fill) in &fill_query {
        if enemy_query.get(fill.owner).is_err() {
            commands.entity(fill_entity).despawn();
        }
    }
}
