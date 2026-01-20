//! Combat domain: boss AI updates and attack execution.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::attacks::AttackStep;
use crate::combat::components::{
    BossAI, BossAttackCooldowns, BossAttackSlots, BossState, Enemy, Hitbox, HitboxLifetime,
    Stagger, Team,
};
use crate::movement::{GameLayer, Player};

pub(crate) fn update_boss_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut boss_query: Query<(
        &Transform,
        &mut BossAI,
        &BossAttackSlots,
        &mut BossAttackCooldowns,
        &crate::combat::components::Health,
        &Stagger,
    )>,
) {
    let dt = time.delta_secs();

    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut ai, slots, mut cooldowns, health, stagger) in &mut boss_query {
        // Handle stagger
        if stagger.is_staggered() && ai.state != BossState::Staggered {
            ai.state = BossState::Staggered;
            ai.state_timer = 0.0;
            continue;
        }

        // Check for phase transitions based on health
        let health_percent = health.percent();
        let expected_phase = if health_percent <= 0.25 {
            3
        } else if health_percent <= 0.5 {
            2
        } else {
            1
        };

        if expected_phase > ai.phase && ai.state != BossState::PhaseTransition {
            ai.state = BossState::PhaseTransition;
            ai.state_timer = 0.0;
            ai.phase = expected_phase;
            continue;
        }

        let boss_pos = transform.translation.truncate();
        let distance = (player_pos - boss_pos).length();

        ai.state_timer += dt;

        match ai.state {
            BossState::Idle => {
                // Choose next action
                if cooldowns.signature <= 0.0 && distance < 200.0 {
                    ai.current_sequence_index = 2; // Signature
                    ai.sequence_step = 0;
                    ai.state = BossState::Telegraph;
                    ai.state_timer = 0.0;
                } else if cooldowns.secondary <= 0.0 && distance < 150.0 {
                    ai.current_sequence_index = 1; // Secondary
                    ai.sequence_step = 0;
                    ai.state = BossState::Telegraph;
                    ai.state_timer = 0.0;
                } else if cooldowns.primary <= 0.0 && distance < 100.0 {
                    ai.current_sequence_index = 0; // Primary
                    ai.sequence_step = 0;
                    ai.state = BossState::Telegraph;
                    ai.state_timer = 0.0;
                } else {
                    // Move toward player
                    ai.state = BossState::Moving;
                    ai.state_timer = 0.0;
                }
            }
            BossState::Telegraph => {
                let sequence = match ai.current_sequence_index {
                    0 => &slots.primary,
                    1 => &slots.secondary,
                    _ => &slots.signature,
                };

                if let Some(AttackStep::Telegraph { duration }) =
                    sequence.steps.get(ai.sequence_step)
                {
                    if ai.state_timer >= *duration {
                        ai.sequence_step += 1;
                        ai.state = BossState::Attacking;
                        ai.state_timer = 0.0;
                    }
                } else {
                    ai.state = BossState::Attacking;
                    ai.state_timer = 0.0;
                }
            }
            BossState::Attacking => {
                let sequence = match ai.current_sequence_index {
                    0 => &slots.primary,
                    1 => &slots.secondary,
                    _ => &slots.signature,
                };

                if ai.sequence_step >= sequence.steps.len() {
                    // Sequence complete, apply cooldown
                    match ai.current_sequence_index {
                        0 => cooldowns.primary = sequence.cooldown,
                        1 => cooldowns.secondary = sequence.cooldown,
                        _ => cooldowns.signature = sequence.cooldown,
                    }
                    ai.state = BossState::Idle;
                    ai.state_timer = 0.0;
                    continue;
                }

                let step = &sequence.steps[ai.sequence_step];
                let step_duration = match step {
                    AttackStep::Telegraph { duration } => *duration,
                    AttackStep::Hitbox { duration, .. } => *duration,
                    AttackStep::Move { duration, .. } => *duration,
                    AttackStep::Jump { duration, .. } => *duration,
                    AttackStep::Wait { duration } => *duration,
                    AttackStep::Recovery { duration } => *duration,
                    AttackStep::Projectile { .. } => 0.1,
                };

                if ai.state_timer >= step_duration {
                    ai.sequence_step += 1;
                    ai.state_timer = 0.0;
                }
            }
            BossState::Recovery => {
                if ai.state_timer >= ai.recovery_timer {
                    ai.state = BossState::Idle;
                    ai.state_timer = 0.0;
                }
            }
            BossState::Moving => {
                if ai.state_timer >= 1.0 || distance < 80.0 {
                    ai.state = BossState::Idle;
                    ai.state_timer = 0.0;
                }
            }
            BossState::PhaseTransition => {
                if ai.state_timer >= 2.0 {
                    ai.state = BossState::Idle;
                    ai.state_timer = 0.0;
                }
            }
            BossState::Staggered => {
                if !stagger.is_staggered() {
                    ai.state = BossState::Idle;
                    ai.state_timer = 0.0;
                }
            }
            BossState::Defeated => {
                // Do nothing
            }
        }
    }
}

pub(crate) fn apply_boss_movement(
    player_query: Query<&Transform, With<Player>>,
    mut boss_query: Query<
        (&Transform, &mut LinearVelocity, &BossAI),
        (With<Enemy>, Without<Player>),
    >,
) {
    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut velocity, ai) in &mut boss_query {
        let boss_pos = transform.translation.truncate();

        if ai.state == BossState::Moving {
            let dir = (player_pos - boss_pos).normalize_or_zero();
            velocity.x = dir.x * 100.0;
        }
        // Other states: let damping slow them down
    }
}

pub(crate) fn process_boss_attacks(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    boss_query: Query<(Entity, &Transform, &BossAI, &BossAttackSlots)>,
) {
    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform, ai, slots) in &boss_query {
        if ai.state != BossState::Attacking {
            continue;
        }

        let sequence = match ai.current_sequence_index {
            0 => &slots.primary,
            1 => &slots.secondary,
            _ => &slots.signature,
        };

        if ai.sequence_step >= sequence.steps.len() {
            continue;
        }

        let step = &sequence.steps[ai.sequence_step];

        // Only spawn hitbox at the start of the step
        if ai.state_timer > 0.05 {
            continue;
        }

        if let AttackStep::Hitbox {
            damage,
            knockback,
            size,
            offset,
            duration,
        } = step
        {
            let boss_pos = transform.translation.truncate();
            let dir_to_player = (player_pos - boss_pos).normalize_or_zero();

            // Apply offset in direction of player
            let hitbox_pos = boss_pos + *offset + dir_to_player * 40.0;

            commands.spawn((
                Hitbox {
                    damage: *damage,
                    knockback: *knockback,
                    stance_damage: 0.0, // Bosses don't deal stance damage to player
                    owner: entity,
                    hit_entities: Vec::new(),
                },
                Team::Enemy,
                HitboxLifetime(*duration),
                Sprite {
                    color: Color::srgba(1.0, 0.2, 0.2, 0.6),
                    custom_size: Some(*size),
                    ..default()
                },
                Transform::from_xyz(hitbox_pos.x, hitbox_pos.y, 1.0),
                Collider::rectangle(size.x, size.y),
                Sensor,
                CollisionEventsEnabled,
                CollisionLayers::new(GameLayer::EnemyHitbox, [GameLayer::Player]),
            ));
        }
    }
}
