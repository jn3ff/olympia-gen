//! Combat domain: enemy and boss spawning helpers.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::combat::attacks::{AttackSequence, AttackStep};
use crate::combat::components::{
    BossAI, BossAttackCooldowns, BossAttackSlots, Combatant, Enemy, EnemyAI, EnemyIdentity,
    EnemyTier, Health, Invulnerable, Stagger, Stance, Team,
};
use crate::combat::resources::EnemyTuning;
use crate::movement::GameLayer;

/// Bundle for spawning enemies with proper tier scaling
#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub tier: EnemyTier,
    pub identity: EnemyIdentity,
    pub combatant: Combatant,
    pub team: Team,
    pub health: Health,
    pub stance: Stance,
    pub stagger: Stagger,
    pub invulnerable: Invulnerable,
    pub ai: EnemyAI,
    pub sprite: Sprite,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub collision_events: CollisionEventsEnabled,
    pub collision_layers: CollisionLayers,
    pub velocity: LinearVelocity,
    pub damping: LinearDamping,
    pub locked_axes: LockedAxes,
    pub gravity_scale: GravityScale,
}

impl EnemyBundle {
    pub fn new(
        tier: EnemyTier,
        position: Vec2,
        base_health: f32,
        _tuning: &EnemyTuning,
        def_id: impl Into<String>,
    ) -> Self {
        let (health_mult, _damage_mult, _speed_mult) = tier.stat_multipliers();
        let effective_health = base_health * health_mult;
        let scale = tier.scale();
        let size = Vec2::new(32.0 * scale, 32.0 * scale);

        // Scale stance based on tier - higher tiers have more stance
        let stance = match tier {
            EnemyTier::Minor => Stance::new(),
            EnemyTier::Major => {
                let mut s = Stance::new();
                s.max *= 1.5;
                s.current = s.max;
                s
            }
            EnemyTier::Special => {
                let mut s = Stance::new();
                s.max *= 2.0;
                s.current = s.max;
                s.break_duration = 2.5;
                s
            }
            EnemyTier::Boss => {
                let mut s = Stance::new();
                s.max *= 3.0;
                s.current = s.max;
                s.break_duration = 3.0;
                s
            }
        };

        Self {
            enemy: Enemy,
            tier,
            identity: EnemyIdentity::from_health(def_id, effective_health),
            combatant: Combatant,
            team: Team::Enemy,
            health: Health::new(effective_health),
            stance,
            stagger: Stagger::default(),
            invulnerable: Invulnerable::default(),
            ai: EnemyAI {
                patrol_origin: position,
                detection_range: 200.0 + (scale - 1.0) * 50.0,
                attack_range: 40.0 + (scale - 1.0) * 20.0,
                ..default()
            },
            sprite: Sprite {
                color: tier.color(),
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_xyz(position.x, position.y, 0.0),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::rectangle(size.x, size.y),
            collision_events: CollisionEventsEnabled,
            collision_layers: CollisionLayers::new(
                GameLayer::Enemy,
                [GameLayer::Ground, GameLayer::Wall, GameLayer::PlayerHitbox],
            ),
            velocity: LinearVelocity::default(),
            damping: LinearDamping(5.0), // High damping to quickly decay knockback velocity
            locked_axes: LockedAxes::ROTATION_LOCKED,
            gravity_scale: GravityScale(1.0), // Normal gravity so enemies fall after knockback
        }
    }
}

/// Spawn a boss enemy with full boss components
pub fn spawn_boss(
    commands: &mut Commands,
    position: Vec2,
    base_health: f32,
    attack_slots: BossAttackSlots,
) -> Entity {
    spawn_boss_scaled(commands, position, base_health, attack_slots, 1.0, 1.0)
}

/// Spawn a boss enemy with difficulty scaling applied
pub fn spawn_boss_scaled(
    commands: &mut Commands,
    position: Vec2,
    base_health: f32,
    mut attack_slots: BossAttackSlots,
    health_multiplier: f32,
    damage_multiplier: f32,
) -> Entity {
    let tier = EnemyTier::Boss;
    let (tier_health_mult, _tier_damage_mult, _speed_mult) = tier.stat_multipliers();
    let scale = tier.scale();
    let size = Vec2::new(32.0 * scale, 32.0 * scale);

    // Apply difficulty scaling to attack damage
    scale_attack_sequence(&mut attack_slots.primary, damage_multiplier);
    scale_attack_sequence(&mut attack_slots.secondary, damage_multiplier);
    scale_attack_sequence(&mut attack_slots.signature, damage_multiplier);
    if let Some(ref mut enraged) = attack_slots.enraged {
        scale_attack_sequence(enraged, damage_multiplier);
    }

    // Calculate final health with tier and difficulty multipliers
    let final_health = base_health * tier_health_mult * health_multiplier;

    // Boss has triple stance
    let mut stance = Stance::new();
    stance.max *= 3.0;
    stance.current = stance.max;
    stance.break_duration = 3.0;

    commands
        .spawn((
            // Identity & Combat
            (
                Enemy,
                tier,
                Combatant,
                Team::Enemy,
                Health::new(final_health),
                stance,
                Stagger::default(),
                Invulnerable::default(),
            ),
            // Boss AI
            (
                BossAI::default(),
                attack_slots,
                BossAttackCooldowns::default(),
            ),
            // Rendering
            (
                Sprite {
                    color: tier.color(),
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_xyz(position.x, position.y, 0.0),
            ),
            // Physics
            (
                RigidBody::Dynamic,
                Collider::rectangle(size.x, size.y),
                CollisionEventsEnabled,
                CollisionLayers::new(
                    GameLayer::Enemy,
                    [GameLayer::Ground, GameLayer::Wall, GameLayer::PlayerHitbox],
                ),
                LinearVelocity::default(),
                LinearDamping(3.0), // Moderate damping for bosses
                LockedAxes::ROTATION_LOCKED,
                GravityScale(1.0), // Normal gravity so boss falls after knockback
            ),
        ))
        .id()
}

/// Scale damage values in an attack sequence
fn scale_attack_sequence(sequence: &mut AttackSequence, damage_multiplier: f32) {
    for step in &mut sequence.steps {
        if let AttackStep::Hitbox { damage, .. } = step {
            *damage *= damage_multiplier;
        }
        if let AttackStep::Projectile { damage, .. } = step {
            *damage *= damage_multiplier;
        }
    }
}
