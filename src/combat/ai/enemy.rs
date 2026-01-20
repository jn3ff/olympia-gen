//! Combat domain: enemy AI updates and attacks.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::components::{AIState, Enemy, EnemyAI, EnemyTier, Hitbox, HitboxLifetime, Team};
use crate::combat::resources::EnemyTuning;
use crate::movement::{GameLayer, Player};

pub(crate) fn update_enemy_ai(
    time: Res<Time>,
    tuning: Res<EnemyTuning>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<
        (
            &Transform,
            &mut EnemyAI,
            &crate::combat::components::Stagger,
            &EnemyTier,
        ),
        (With<Enemy>, Without<crate::combat::components::BossAI>),
    >,
) {
    let dt = time.delta_secs();

    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut ai, stagger, _tier) in &mut enemy_query {
        if stagger.is_staggered() {
            ai.state = AIState::Staggered;
            continue;
        }

        let enemy_pos = transform.translation.truncate();
        let to_player = player_pos - enemy_pos;
        let distance = to_player.length();

        ai.state_timer += dt;

        match ai.state {
            AIState::Patrol => {
                if distance < ai.detection_range {
                    ai.state = AIState::Chase;
                    ai.state_timer = 0.0;
                } else {
                    let offset_from_origin = enemy_pos.x - ai.patrol_origin.x;
                    if offset_from_origin.abs() > ai.patrol_range {
                        ai.patrol_direction = -ai.patrol_direction;
                        ai.state_timer = 0.0;
                    }
                }
            }
            AIState::Chase => {
                if distance > ai.detection_range * 1.5 {
                    ai.state = AIState::Patrol;
                    ai.state_timer = 0.0;
                } else if distance < ai.attack_range {
                    ai.state = AIState::Attack;
                    ai.state_timer = 0.0;
                }
            }
            AIState::Attack => {
                if ai.state_timer > tuning.attack_duration + tuning.attack_cooldown {
                    ai.state = AIState::Chase;
                    ai.state_timer = 0.0;
                }
            }
            AIState::Staggered => {
                ai.state = AIState::Chase;
                ai.state_timer = 0.0;
            }
        }
    }
}

pub(crate) fn apply_enemy_movement(
    tuning: Res<EnemyTuning>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut LinearVelocity, &EnemyAI, &EnemyTier),
        (
            With<Enemy>,
            Without<Player>,
            Without<crate::combat::components::BossAI>,
        ),
    >,
) {
    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut velocity, ai, tier) in &mut enemy_query {
        let enemy_pos = transform.translation.truncate();
        let (_, _, speed_mult) = tier.stat_multipliers();

        // Only apply horizontal movement velocity, let physics handle the rest
        match ai.state {
            AIState::Patrol => {
                if ai.state_timer > tuning.patrol_pause_time {
                    velocity.x = ai.patrol_direction * tuning.move_speed * speed_mult;
                }
            }
            AIState::Chase => {
                let dir = (player_pos - enemy_pos).normalize_or_zero();
                velocity.x = dir.x * tuning.chase_speed * speed_mult;
            }
            AIState::Attack | AIState::Staggered => {
                // Let damping naturally slow them down during attack/stagger
            }
        }
    }
}

pub(crate) fn process_enemy_attacks(
    mut commands: Commands,
    tuning: Res<EnemyTuning>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<
        (Entity, &Transform, &EnemyAI, &EnemyTier),
        (With<Enemy>, Without<crate::combat::components::BossAI>),
    >,
) {
    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, ai, tier) in &enemy_query {
        if ai.state == AIState::Attack && ai.state_timer < 0.05 {
            let enemy_pos = transform.translation.truncate();
            let dir = (player_pos - enemy_pos).normalize_or_zero();
            let hitbox_offset = dir * 25.0 * tier.scale();
            let hitbox_pos = enemy_pos + hitbox_offset;
            let (_, damage_mult, _) = tier.stat_multipliers();
            let hitbox_size = Vec2::splat(35.0 * tier.scale());

            commands.spawn((
                Hitbox {
                    damage: tuning.attack_damage * damage_mult,
                    knockback: tuning.attack_knockback * damage_mult,
                    stance_damage: 0.0, // Enemies don't deal stance damage to player
                    owner: entity,
                    hit_entities: Vec::new(),
                },
                Team::Enemy,
                HitboxLifetime(tuning.attack_duration * 0.5),
                Sprite {
                    color: Color::srgba(1.0, 0.3, 0.3, 0.5),
                    custom_size: Some(hitbox_size),
                    ..default()
                },
                Transform::from_xyz(hitbox_pos.x, hitbox_pos.y, 1.0),
                Collider::rectangle(hitbox_size.x, hitbox_size.y),
                Sensor,
                CollisionEventsEnabled,
                CollisionLayers::new(GameLayer::EnemyHitbox, [GameLayer::Player]),
            ));
        }
    }
}
