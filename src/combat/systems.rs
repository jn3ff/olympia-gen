//! Combat domain: combat systems for input, damage, and cleanup.

use avian2d::prelude::*;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::combat::attacks::{AttackDirection, AttackType};
use crate::combat::components::{
    ActiveAttackType, AttackState, BossAI, BossAttackCooldowns, BossState, Combatant, ComboState,
    Enemy, EnemyIdentity, EnemyTier, Hitbox, HitboxLifetime, Invulnerable, ParryState, Stagger,
    Stance, Team, TelegraphIndicator, Weapon,
};
use crate::combat::events::{
    BossDefeatedEvent, BossPhaseChangeEvent, DamageEvent, DeathEvent, ParrySuccessEvent,
    StanceBreakEvent,
};
use crate::combat::resources::{AttackTuning, CombatInput, CombatTuning, GoldDropConfig};
use crate::content::{ContentRegistry, GameplayDefaults};
use crate::core::SegmentProgress;
use crate::movement::{Facing, GameLayer, MovementInput, MovementState, Player};
use crate::rewards::{CoinGainedEvent, CoinSource};

pub(crate) fn read_combat_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<CombatInput>,
) {
    input.light_attack =
        keyboard.just_pressed(KeyCode::KeyZ) || keyboard.just_pressed(KeyCode::KeyU);
    input.heavy_attack =
        keyboard.just_pressed(KeyCode::KeyX) || keyboard.just_pressed(KeyCode::KeyI);
    input.special_attack =
        keyboard.just_pressed(KeyCode::KeyC) || keyboard.just_pressed(KeyCode::KeyO);
    // Parry is on V/P keys
    input.parry = keyboard.just_pressed(KeyCode::KeyV) || keyboard.just_pressed(KeyCode::KeyP);
}

pub(crate) fn update_combat_timers(
    time: Res<Time>,
    mut query: Query<(&mut Stagger, &mut Invulnerable, Option<&mut AttackState>)>,
    mut boss_cooldowns: Query<&mut BossAttackCooldowns>,
) {
    let dt = time.delta_secs();

    for (mut stagger, mut invuln, attack_state) in &mut query {
        if stagger.timer > 0.0 {
            stagger.timer -= dt;
        }
        if invuln.timer > 0.0 {
            invuln.timer -= dt;
        }
        if let Some(mut attack) = attack_state {
            if attack.attack_timer > 0.0 {
                attack.attack_timer -= dt;
                if attack.attack_timer <= 0.0 {
                    attack.current_attack = None;
                }
            }
            if attack.cooldown_timer > 0.0 {
                attack.cooldown_timer -= dt;
            }
            if attack.combo_window > 0.0 {
                attack.combo_window -= dt;
                if attack.combo_window <= 0.0 {
                    attack.combo_count = 0;
                }
            }
        }
    }

    // Update boss attack cooldowns
    for mut cooldowns in &mut boss_cooldowns {
        if cooldowns.primary > 0.0 {
            cooldowns.primary -= dt;
        }
        if cooldowns.secondary > 0.0 {
            cooldowns.secondary -= dt;
        }
        if cooldowns.signature > 0.0 {
            cooldowns.signature -= dt;
        }
    }
}

/// Update combo state timers
pub(crate) fn update_combo_state(time: Res<Time>, mut query: Query<&mut ComboState, With<Player>>) {
    let dt = time.delta_secs();

    for mut combo in &mut query {
        // Update attack timer
        if combo.attack_timer > 0.0 {
            combo.attack_timer -= dt;
            if combo.attack_timer <= 0.0 {
                combo.active_attack = None;
            }
        }

        // Update cooldown timer
        if combo.cooldown_timer > 0.0 {
            combo.cooldown_timer -= dt;
        }

        // Update combo window - reset indices when window expires
        if combo.combo_window > 0.0 {
            combo.combo_window -= dt;
            if combo.combo_window <= 0.0 {
                combo.light_index = 0;
                combo.heavy_index = 0;
            }
        }
    }
}

/// Update parry state timers
pub(crate) fn update_parry_state(time: Res<Time>, mut query: Query<&mut ParryState, With<Player>>) {
    let dt = time.delta_secs();
    for mut parry in &mut query {
        parry.update(dt);
    }
}

/// Update stance meters for all enemies
pub(crate) fn update_stance(
    time: Res<Time>,
    gameplay_defaults: Option<Res<GameplayDefaults>>,
    mut query: Query<&mut Stance, With<Enemy>>,
) {
    let dt = time.delta_secs();
    // Default regen rate: 1 light worth per 10 seconds
    let regen_rate = gameplay_defaults
        .map(|d| d.stance_defaults.regen_light_per_seconds)
        .unwrap_or(10.0);

    for mut stance in &mut query {
        stance.update_break(dt);
        stance.regenerate(dt, regen_rate);
    }
}

/// Process player attacks using moveset data from ContentRegistry
pub(crate) fn process_player_attacks_from_moveset(
    mut commands: Commands,
    input: Res<CombatInput>,
    move_input: Res<MovementInput>,
    registry: Option<Res<ContentRegistry>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &MovementState,
            &Weapon,
            &crate::combat::components::PlayerMoveset,
            &mut ComboState,
            &Stagger,
        ),
        With<Player>,
    >,
) {
    let Some(registry) = registry else {
        return;
    };

    for (entity, transform, movement, weapon, player_moveset, mut combo, stagger) in &mut query {
        // Can't attack while staggered or during active attack
        if stagger.is_staggered() || combo.active_attack.is_some() {
            continue;
        }

        // Can't attack during cooldown
        if combo.cooldown_timer > 0.0 {
            continue;
        }

        // Get the moveset from registry
        let Some(moveset) = registry.movesets.get(&player_moveset.moveset_id) else {
            warn!(
                "Moveset '{}' not found in registry",
                player_moveset.moveset_id
            );
            continue;
        };

        // Determine which attack type was input
        let attack_input = if input.light_attack {
            Some(ActiveAttackType::Light)
        } else if input.heavy_attack {
            Some(ActiveAttackType::Heavy)
        } else if input.special_attack {
            Some(ActiveAttackType::Special)
        } else {
            None
        };

        let Some(attack_type) = attack_input else {
            continue;
        };

        // Get the strike data based on attack type and combo position
        let (strike, new_combo_index, is_light, is_heavy) = match attack_type {
            ActiveAttackType::Light => {
                let combo_def = &moveset.light_combo;
                let strike = &combo_def.strikes[combo.light_index];
                let next_index = if combo.light_index + 1 >= combo_def.strikes.len() {
                    combo_def.loop_from
                } else {
                    combo.light_index + 1
                };
                (strike, next_index, true, false)
            }
            ActiveAttackType::Heavy => {
                let combo_def = &moveset.heavy_combo;
                let strike = &combo_def.strikes[combo.heavy_index];
                let next_index = if combo.heavy_index + 1 >= combo_def.strikes.len() {
                    combo_def.loop_from
                } else {
                    combo.heavy_index + 1
                };
                (strike, next_index, false, true)
            }
            ActiveAttackType::Special => {
                // Special doesn't have a combo chain
                (&moveset.special, 0, false, false)
            }
            ActiveAttackType::Parry => continue, // Parry handled separately
        };

        // Determine attack direction based on input
        let direction = if move_input.axis.y > 0.5 {
            AttackDirection::Up
        } else if move_input.axis.y < -0.5 {
            AttackDirection::Down
        } else {
            match movement.facing {
                Facing::Right => AttackDirection::Right,
                Facing::Left => AttackDirection::Left,
            }
        };

        // Apply weapon multipliers
        let damage = strike.damage * weapon.damage_multiplier;
        let knockback = 150.0 * weapon.knockback_multiplier; // Base knockback
        let stance_damage = strike.stance_damage;

        // Set combo state
        combo.active_attack = Some(attack_type);
        combo.attack_timer = strike.cooldown * 0.8; // Attack animation time
        combo.cooldown_timer = strike.cooldown;
        combo.combo_window = 0.5; // Half second to continue combo

        // Update combo index
        if is_light {
            combo.light_index = new_combo_index;
        } else if is_heavy {
            combo.heavy_index = new_combo_index;
        }

        // Calculate hitbox position and size from strike data
        let hitbox_offset = direction.to_offset(strike.hitbox.offset);
        let hitbox_pos = transform.translation.truncate() + hitbox_offset;
        let hitbox_size = direction.hitbox_size(strike.hitbox.length, strike.hitbox.width);

        // Visual color based on attack type
        let color = match attack_type {
            ActiveAttackType::Light => Color::srgba(1.0, 1.0, 0.0, 0.5),
            ActiveAttackType::Heavy => Color::srgba(1.0, 0.6, 0.0, 0.5),
            ActiveAttackType::Special => Color::srgba(0.8, 0.2, 1.0, 0.5),
            ActiveAttackType::Parry => Color::srgba(0.2, 0.8, 1.0, 0.5),
        };

        // Spawn hitbox
        commands.spawn((
            Hitbox {
                damage,
                knockback,
                stance_damage,
                owner: entity,
                hit_entities: Vec::new(),
            },
            Team::Player,
            HitboxLifetime(strike.cooldown * 0.6),
            Sprite {
                color,
                custom_size: Some(hitbox_size),
                ..default()
            },
            Transform::from_xyz(hitbox_pos.x, hitbox_pos.y, 1.0),
            Collider::rectangle(hitbox_size.x, hitbox_size.y),
            Sensor,
            CollisionEventsEnabled,
            CollisionLayers::new(GameLayer::PlayerHitbox, [GameLayer::Enemy]),
        ));

        debug!(
            "Player attack: {:?} with strike '{}', damage={}, stance_damage={}",
            attack_type, strike.id, damage, stance_damage
        );
    }
}

/// Process parry input
pub(crate) fn process_parry_input(
    input: Res<CombatInput>,
    registry: Option<Res<ContentRegistry>>,
    mut query: Query<
        (
            &crate::combat::components::PlayerMoveset,
            &mut ParryState,
            &mut ComboState,
            &Stagger,
        ),
        With<Player>,
    >,
) {
    if !input.parry {
        return;
    }

    let Some(registry) = registry else {
        return;
    };

    for (player_moveset, mut parry, mut combo, stagger) in &mut query {
        // Can't parry while staggered or during other actions
        if stagger.is_staggered() || combo.active_attack.is_some() {
            continue;
        }

        // Get parry window from moveset
        let Some(moveset) = registry.movesets.get(&player_moveset.moveset_id) else {
            continue;
        };

        if !moveset.parry.enabled {
            continue;
        }

        // Set parry window duration from moveset
        parry.window_duration = moveset.parry.window_seconds;
        parry.start_parry();

        // Set combo state to parry
        combo.active_attack = Some(ActiveAttackType::Parry);
        combo.attack_timer = moveset.parry.window_seconds + 0.1;

        debug!(
            "Player parry started with window {}s",
            moveset.parry.window_seconds
        );
    }
}

/// Check for parry collisions with incoming enemy attacks
pub(crate) fn check_parry_collisions(
    mut collision_events: MessageReader<CollisionStart>,
    mut parry_events: MessageWriter<ParrySuccessEvent>,
    mut stance_break_events: MessageWriter<StanceBreakEvent>,
    gameplay_defaults: Option<Res<GameplayDefaults>>,
    parry_query: Query<(Entity, &ParryState), With<Player>>,
    hitbox_query: Query<(&Hitbox, &Team)>,
    mut enemy_stance_query: Query<&mut Stance, With<Enemy>>,
) {
    // Parry deals 8 heavies worth of stance damage by default
    let parry_stance_damage = gameplay_defaults
        .map(|d| {
            let heavy_value = d.stance_defaults.light_break_count as f32
                / d.stance_defaults.heavy_break_count as f32;
            heavy_value * d.stance_defaults.parry_heavy_equivalent as f32
        })
        .unwrap_or(14.0); // 8 * 1.75 = 14

    for event in collision_events.read() {
        let pairs = [
            (event.collider1, event.collider2),
            (event.collider2, event.collider1),
        ];

        for (player_entity, hitbox_entity) in pairs {
            // Check if this is a player with active parry
            let Ok((player, parry)) = parry_query.get(player_entity) else {
                continue;
            };

            if !parry.can_parry() {
                continue;
            }

            // Check if colliding with enemy hitbox
            let Ok((hitbox, team)) = hitbox_query.get(hitbox_entity) else {
                continue;
            };

            if *team != Team::Enemy {
                continue;
            }

            // Parry successful! Apply massive stance damage to attacker
            if let Ok(mut stance) = enemy_stance_query.get_mut(hitbox.owner) {
                let broke = stance.take_damage(parry_stance_damage);
                if broke {
                    stance_break_events.write(StanceBreakEvent {
                        entity: hitbox.owner,
                        breaker: player,
                    });
                }
            }

            parry_events.write(ParrySuccessEvent {
                parrier: player,
                attacker: hitbox.owner,
            });

            info!(
                "Parry successful! Applied {} stance damage to attacker",
                parry_stance_damage
            );
        }
    }
}

/// Apply stance damage from damage events
pub(crate) fn apply_stance_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut stance_break_events: MessageWriter<StanceBreakEvent>,
    mut stance_query: Query<&mut Stance>,
) {
    for event in damage_events.read() {
        if event.stance_damage <= 0.0 {
            continue;
        }

        if let Ok(mut stance) = stance_query.get_mut(event.target) {
            let broke = stance.take_damage(event.stance_damage);
            if broke {
                stance_break_events.write(StanceBreakEvent {
                    entity: event.target,
                    breaker: event.source,
                });
                info!(
                    "Stance broken on entity {:?} by {:?}!",
                    event.target, event.source
                );
            }
        }
    }
}

/// Process stance break events - apply stagger to broken enemies
pub(crate) fn process_stance_breaks(
    mut stance_break_events: MessageReader<StanceBreakEvent>,
    mut query: Query<(&mut Stagger, &Stance)>,
) {
    for event in stance_break_events.read() {
        if let Ok((mut stagger, stance)) = query.get_mut(event.entity) {
            // Apply long stagger equal to break duration
            stagger.timer = stance.break_duration;
            info!(
                "Entity {:?} staggered for {}s from stance break",
                event.entity, stance.break_duration
            );
        }
    }
}

#[allow(dead_code)]
pub(crate) fn process_player_attacks(
    mut commands: Commands,
    input: Res<CombatInput>,
    move_input: Res<MovementInput>,
    tuning: Res<AttackTuning>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &MovementState,
            &Weapon,
            &mut AttackState,
            &Stagger,
        ),
        With<Player>,
    >,
) {
    for (entity, transform, movement, weapon, mut attack_state, stagger) in &mut query {
        if stagger.is_staggered() || attack_state.current_attack.is_some() {
            continue;
        }

        if attack_state.cooldown_timer > 0.0 {
            continue;
        }

        let attack_type = if input.light_attack {
            Some(AttackType::Light)
        } else if input.heavy_attack {
            Some(AttackType::Heavy)
        } else if input.special_attack {
            Some(AttackType::Special)
        } else {
            None
        };

        if let Some(attack) = attack_type {
            let direction = if move_input.axis.y > 0.5 {
                AttackDirection::Up
            } else if move_input.axis.y < -0.5 {
                AttackDirection::Down
            } else {
                match movement.facing {
                    Facing::Right => AttackDirection::Right,
                    Facing::Left => AttackDirection::Left,
                }
            };

            let config = match attack {
                AttackType::Light => &tuning.light,
                AttackType::Heavy => &tuning.heavy,
                AttackType::Special => &tuning.special,
            };

            let damage = config.damage * weapon.damage_multiplier;
            let knockback = config.knockback * weapon.knockback_multiplier;

            attack_state.current_attack = Some(attack);
            attack_state.attack_direction = direction;
            attack_state.attack_timer = config.duration;
            attack_state.cooldown_timer = config.cooldown;

            if attack == AttackType::Light {
                attack_state.combo_count = (attack_state.combo_count + 1) % 3;
                attack_state.combo_window = tuning.combo_window;
            }

            let hitbox_offset = direction.to_offset(config.hitbox_offset);
            let hitbox_pos = transform.translation.truncate() + hitbox_offset;
            let hitbox_size = direction.hitbox_size(config.hitbox_length, config.hitbox_width);

            let color = match attack {
                AttackType::Light => Color::srgba(1.0, 1.0, 0.0, 0.5),
                AttackType::Heavy => Color::srgba(1.0, 0.6, 0.0, 0.5),
                AttackType::Special => Color::srgba(0.8, 0.2, 1.0, 0.5),
            };

            // Default stance damage based on attack type
            let stance_damage = match attack {
                AttackType::Light => 1.0,
                AttackType::Heavy => 1.75,
                AttackType::Special => 3.0,
            };

            commands.spawn((
                Hitbox {
                    damage,
                    knockback,
                    stance_damage,
                    owner: entity,
                    hit_entities: Vec::new(),
                },
                Team::Player,
                HitboxLifetime(config.hitbox_duration),
                Sprite {
                    color,
                    custom_size: Some(hitbox_size),
                    ..default()
                },
                Transform::from_xyz(hitbox_pos.x, hitbox_pos.y, 1.0),
                Collider::rectangle(hitbox_size.x, hitbox_size.y),
                Sensor,
                CollisionEventsEnabled,
                CollisionLayers::new(GameLayer::PlayerHitbox, [GameLayer::Enemy]),
            ));
        }
    }
}

pub(crate) fn cleanup_expired_hitboxes(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HitboxLifetime)>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifetime) in &mut query {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn detect_hitbox_collisions(
    mut collision_events: MessageReader<CollisionStart>,
    mut damage_events: MessageWriter<DamageEvent>,
    mut hitbox_query: Query<(&mut Hitbox, &Team, &Transform)>,
    target_query: Query<(Entity, &Team, &Invulnerable, &Transform), With<Combatant>>,
) {
    for event in collision_events.read() {
        let pairs = [
            (event.collider1, event.collider2),
            (event.collider2, event.collider1),
        ];

        for (hitbox_entity, target_entity) in pairs {
            if let Ok((mut hitbox, hitbox_team, hitbox_transform)) =
                hitbox_query.get_mut(hitbox_entity)
            {
                if let Ok((target, target_team, invuln, target_transform)) =
                    target_query.get(target_entity)
                {
                    if hitbox_team == target_team {
                        continue;
                    }

                    if hitbox.hit_entities.contains(&target) {
                        continue;
                    }

                    if invuln.is_invulnerable() {
                        continue;
                    }

                    if hitbox.owner == target {
                        continue;
                    }

                    hitbox.hit_entities.push(target);

                    // Calculate knockback direction from hitbox to target
                    let hitbox_pos = hitbox_transform.translation.truncate();
                    let target_pos = target_transform.translation.truncate();
                    let knockback_dir = (target_pos - hitbox_pos).normalize_or_zero();
                    // If direction is zero (same position), default to pushing right
                    let knockback_dir = if knockback_dir == Vec2::ZERO {
                        Vec2::X
                    } else {
                        knockback_dir
                    };

                    damage_events.write(DamageEvent {
                        source: hitbox.owner,
                        target,
                        amount: hitbox.damage,
                        knockback: knockback_dir * hitbox.knockback,
                        stance_damage: hitbox.stance_damage,
                    });
                }
            }
        }
    }
}

pub(crate) fn apply_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    tuning: Res<CombatTuning>,
    mut query: Query<(
        &mut crate::combat::components::Health,
        &mut Stagger,
        &mut Invulnerable,
        &mut Sprite,
    )>,
) {
    for event in damage_events.read() {
        if let Ok((mut health, mut stagger, mut invuln, mut sprite)) = query.get_mut(event.target) {
            health.take_damage(event.amount);

            stagger.timer = tuning.stagger_duration;
            invuln.timer = tuning.iframes_duration;

            sprite.color = Color::srgb(1.0, 0.5, 0.5);

            if health.is_dead() {
                death_events.write(DeathEvent {
                    entity: event.target,
                });
            }
        }
    }
}

/// Maximum velocity an entity can have after knockback
const MAX_KNOCKBACK_VELOCITY: f32 = 800.0;
/// Minimum upward knockback to give a small lift
const MIN_VERTICAL_KNOCKBACK: f32 = 100.0;

pub(crate) fn apply_knockback(
    mut damage_events: MessageReader<DamageEvent>,
    mut query: Query<&mut LinearVelocity>,
) {
    for event in damage_events.read() {
        if let Ok(mut velocity) = query.get_mut(event.target) {
            // Apply knockback
            velocity.x += event.knockback.x;
            velocity.y += event.knockback.y.max(MIN_VERTICAL_KNOCKBACK);

            // Clamp final velocity to prevent extreme values
            let speed = (velocity.x * velocity.x + velocity.y * velocity.y).sqrt();
            if speed > MAX_KNOCKBACK_VELOCITY {
                let scale = MAX_KNOCKBACK_VELOCITY / speed;
                velocity.x *= scale;
                velocity.y *= scale;
            }

            debug!(
                "Knockback applied: knockback={:?}, final_velocity=({:.1}, {:.1})",
                event.knockback, velocity.x, velocity.y
            );
        }
    }
}

pub(crate) fn check_boss_phase_transitions(
    mut phase_events: MessageWriter<BossPhaseChangeEvent>,
    query: Query<(Entity, &BossAI), Changed<BossAI>>,
) {
    for (entity, ai) in &query {
        if ai.state == BossState::PhaseTransition {
            phase_events.write(BossPhaseChangeEvent {
                boss: entity,
                new_phase: ai.phase,
            });
        }
    }
}

pub(crate) fn process_deaths(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    mut boss_defeated_events: MessageWriter<BossDefeatedEvent>,
    mut boss_state: ResMut<crate::combat::resources::BossEncounterState>,
    mut segment_progress: ResMut<SegmentProgress>,
    enemy_query: Query<(Entity, Option<&BossAI>, Option<&EnemyIdentity>), With<Enemy>>,
) {
    for event in death_events.read() {
        if let Ok((entity, boss_ai, identity)) = enemy_query.get(event.entity) {
            // Track significant enemy deaths for no-repeat logic
            if let Some(id) = identity {
                if id.is_significant {
                    segment_progress
                        .encountered_significant_enemies
                        .insert(id.def_id.clone());
                    info!(
                        "Significant enemy '{}' defeated and tracked for no-repeat",
                        id.def_id
                    );
                }
            }

            if boss_ai.is_some() {
                // Boss defeated
                boss_state.boss_defeated();
                boss_defeated_events.write(BossDefeatedEvent { boss: entity });
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Drop coins when enemies die based on their tier
pub(crate) fn handle_enemy_coin_drops(
    mut death_events: MessageReader<DeathEvent>,
    enemy_query: Query<&EnemyTier, With<Enemy>>,
    gold_config: Res<GoldDropConfig>,
    mut coin_events: MessageWriter<CoinGainedEvent>,
) {
    for event in death_events.read() {
        if let Ok(tier) = enemy_query.get(event.entity) {
            // Roll gold from configured range for this tier
            let coins = gold_config.range_for_tier(tier).roll();

            if coins > 0 {
                coin_events.write(CoinGainedEvent {
                    amount: coins,
                    source: CoinSource::EnemyDrop,
                });
            }
        }
    }
}

pub(crate) fn cleanup_telegraphs(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TelegraphIndicator)>,
) {
    let dt = time.delta_secs();
    for (entity, mut telegraph) in &mut query {
        telegraph.timer -= dt;
        if telegraph.timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
