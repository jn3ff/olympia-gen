use avian2d::prelude::*;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;

use crate::movement::{Facing, GameLayer, MovementInput, MovementState, Player};

// ============================================================================
// Components
// ============================================================================

/// Marks an entity as a combat participant
#[derive(Component, Debug)]
pub struct Combatant;

/// Health component for damageable entities
#[derive(Component, Debug, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn take_damage(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.current);
        self.current -= actual;
        actual
    }

    pub fn heal(&mut self, amount: f32) -> f32 {
        let actual = amount.min(self.max - self.current);
        self.current += actual;
        actual
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0.0
    }

    pub fn percent(&self) -> f32 {
        self.current / self.max
    }
}

/// Stagger state - entity cannot act while staggered
#[derive(Component, Debug, Default)]
pub struct Stagger {
    pub timer: f32,
}

impl Stagger {
    pub fn is_staggered(&self) -> bool {
        self.timer > 0.0
    }
}

/// Invulnerability frames - entity cannot take damage
#[derive(Component, Debug, Default)]
pub struct Invulnerable {
    pub timer: f32,
}

impl Invulnerable {
    pub fn is_invulnerable(&self) -> bool {
        self.timer > 0.0
    }
}

/// Hitbox - deals damage on contact with hurtboxes
#[derive(Component, Debug)]
pub struct Hitbox {
    pub damage: f32,
    pub knockback: f32,
    pub owner: Entity,
    pub hit_entities: Vec<Entity>,
}

/// Hurtbox - receives damage from hitboxes
#[derive(Component, Debug)]
pub struct Hurtbox {
    pub owner: Entity,
}

/// Team affiliation to prevent friendly fire
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
}

// ============================================================================
// Enemy Tier System
// ============================================================================

/// Enemy tier classification - determines stats, behavior, and rewards
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnemyTier {
    /// Basic fodder enemies with simple behavior
    #[default]
    Minor,
    /// Tougher enemies with more health and varied attacks
    Major,
    /// Unique enemies with special mechanics
    Special,
    /// Boss enemies with complex attack patterns and phases
    Boss,
}

impl EnemyTier {
    /// Get stat multipliers for this tier (health, damage, speed)
    pub fn stat_multipliers(&self) -> (f32, f32, f32) {
        match self {
            EnemyTier::Minor => (1.0, 1.0, 1.0),
            EnemyTier::Major => (2.5, 1.5, 1.1),
            EnemyTier::Special => (4.0, 2.0, 1.2),
            EnemyTier::Boss => (10.0, 2.5, 0.9),
        }
    }

    /// Get visual scale for this tier
    pub fn scale(&self) -> f32 {
        match self {
            EnemyTier::Minor => 1.0,
            EnemyTier::Major => 1.3,
            EnemyTier::Special => 1.5,
            EnemyTier::Boss => 2.0,
        }
    }

    /// Get color tint for this tier
    pub fn color(&self) -> Color {
        match self {
            EnemyTier::Minor => Color::srgb(0.8, 0.3, 0.3),
            EnemyTier::Major => Color::srgb(0.9, 0.5, 0.2),
            EnemyTier::Special => Color::srgb(0.7, 0.3, 0.8),
            EnemyTier::Boss => Color::srgb(0.9, 0.1, 0.1),
        }
    }
}

// ============================================================================
// Weapon System
// ============================================================================

#[derive(Component, Debug, Clone)]
pub struct Weapon {
    pub damage_multiplier: f32,
    pub knockback_multiplier: f32,
    pub speed_multiplier: f32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            knockback_multiplier: 1.0,
            speed_multiplier: 1.0,
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct AttackState {
    pub current_attack: Option<AttackType>,
    pub attack_direction: AttackDirection,
    pub attack_timer: f32,
    pub cooldown_timer: f32,
    pub combo_count: u8,
    pub combo_window: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    Light,
    Heavy,
    Special,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttackDirection {
    Up,
    Down,
    #[default]
    Left,
    Right,
}

impl AttackDirection {
    /// Get the offset vector for hitbox placement
    pub fn to_offset(self, distance: f32) -> Vec2 {
        match self {
            AttackDirection::Up => Vec2::new(0.0, distance),
            AttackDirection::Down => Vec2::new(0.0, -distance),
            AttackDirection::Left => Vec2::new(-distance, 0.0),
            AttackDirection::Right => Vec2::new(distance, 0.0),
        }
    }

    /// Get hitbox dimensions (width, height) - elongated in attack direction
    pub fn hitbox_size(self, length: f32, width: f32) -> Vec2 {
        match self {
            AttackDirection::Up | AttackDirection::Down => Vec2::new(width, length),
            AttackDirection::Left | AttackDirection::Right => Vec2::new(length, width),
        }
    }
}

/// Per-attack-type configuration
#[derive(Debug, Clone)]
pub struct AttackConfig {
    pub damage: f32,
    pub knockback: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub hitbox_length: f32,
    pub hitbox_width: f32,
    pub hitbox_offset: f32,
    pub hitbox_duration: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct AttackTuning {
    pub light: AttackConfig,
    pub heavy: AttackConfig,
    pub special: AttackConfig,
    pub combo_window: f32,
}

impl Default for AttackTuning {
    fn default() -> Self {
        Self {
            light: AttackConfig {
                damage: 8.0,
                knockback: 150.0,
                duration: 0.12,
                cooldown: 0.05,
                hitbox_length: 45.0,
                hitbox_width: 35.0,
                hitbox_offset: 28.0,
                hitbox_duration: 0.08,
            },
            heavy: AttackConfig {
                damage: 25.0,
                knockback: 400.0,
                duration: 0.5,
                cooldown: 0.4,
                hitbox_length: 50.0,
                hitbox_width: 45.0,
                hitbox_offset: 32.0,
                hitbox_duration: 0.15,
            },
            special: AttackConfig {
                damage: 40.0,
                knockback: 600.0,
                duration: 0.6,
                cooldown: 0.8,
                hitbox_length: 60.0,
                hitbox_width: 50.0,
                hitbox_offset: 35.0,
                hitbox_duration: 0.2,
            },
            combo_window: 0.3,
        }
    }
}

// ============================================================================
// Skill System
// ============================================================================

#[derive(Component, Debug, Default, Clone)]
pub struct SkillSlots {
    pub passive: Option<String>,
    pub common: Option<String>,
    pub heavy: Option<String>,
}

#[derive(Component, Debug, Default)]
pub struct SkillCooldowns {
    pub common_timer: f32,
    pub heavy_timer: f32,
}

// ============================================================================
// Enemy AI
// ============================================================================

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct EnemyAI {
    pub state: AIState,
    pub patrol_origin: Vec2,
    pub patrol_range: f32,
    pub detection_range: f32,
    pub attack_range: f32,
    pub state_timer: f32,
    pub patrol_direction: f32,
}

impl Default for EnemyAI {
    fn default() -> Self {
        Self {
            state: AIState::Patrol,
            patrol_origin: Vec2::ZERO,
            patrol_range: 100.0,
            detection_range: 200.0,
            attack_range: 40.0,
            state_timer: 0.0,
            patrol_direction: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AIState {
    #[default]
    Patrol,
    Chase,
    Attack,
    Staggered,
}

#[derive(Resource, Debug, Clone)]
pub struct EnemyTuning {
    pub move_speed: f32,
    pub chase_speed: f32,
    pub attack_damage: f32,
    pub attack_knockback: f32,
    pub attack_duration: f32,
    pub attack_cooldown: f32,
    pub stagger_duration: f32,
    pub patrol_pause_time: f32,
}

impl Default for EnemyTuning {
    fn default() -> Self {
        Self {
            move_speed: 80.0,
            chase_speed: 150.0,
            attack_damage: 15.0,
            attack_knockback: 300.0,
            attack_duration: 0.4,
            attack_cooldown: 1.0,
            stagger_duration: 0.3,
            patrol_pause_time: 1.0,
        }
    }
}

// ============================================================================
// Boss System
// ============================================================================

/// Configuration for boss encounter spawning
#[derive(Resource, Debug, Clone)]
pub struct BossConfig {
    /// Probability of a boss encounter per room (0.0 to 1.0)
    pub encounter_rate: f32,
    /// Minimum rooms between boss encounters
    pub min_rooms_between: u32,
    /// Maximum rooms without a boss before forcing one
    pub max_rooms_between: u32,
}

impl Default for BossConfig {
    fn default() -> Self {
        Self {
            encounter_rate: 0.15,
            min_rooms_between: 3,
            max_rooms_between: 8,
        }
    }
}

/// Tracks boss encounter state across the run
#[derive(Resource, Debug, Default)]
pub struct BossEncounterState {
    /// Rooms cleared since last boss
    pub rooms_since_boss: u32,
    /// Total bosses defeated this run
    pub bosses_defeated: u32,
}

impl BossEncounterState {
    /// Determine if the next room should have a boss
    pub fn should_spawn_boss(&self, config: &BossConfig, rng_roll: f32) -> bool {
        // Force boss if we've hit max rooms
        if self.rooms_since_boss >= config.max_rooms_between {
            return true;
        }

        // Never spawn boss before min rooms
        if self.rooms_since_boss < config.min_rooms_between {
            return false;
        }

        // Roll for boss based on encounter rate
        rng_roll < config.encounter_rate
    }

    /// Record a room cleared (no boss)
    pub fn room_cleared(&mut self) {
        self.rooms_since_boss += 1;
    }

    /// Record a boss defeated
    pub fn boss_defeated(&mut self) {
        self.rooms_since_boss = 0;
        self.bosses_defeated += 1;
    }
}

/// Boss-specific AI state machine
#[derive(Component, Debug)]
pub struct BossAI {
    pub state: BossState,
    pub state_timer: f32,
    pub phase: u8,
    pub current_sequence_index: usize,
    pub sequence_step: usize,
    pub telegraph_timer: f32,
    pub recovery_timer: f32,
}

impl Default for BossAI {
    fn default() -> Self {
        Self {
            state: BossState::Idle,
            state_timer: 0.0,
            phase: 1,
            current_sequence_index: 0,
            sequence_step: 0,
            telegraph_timer: 0.0,
            recovery_timer: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BossState {
    #[default]
    Idle,
    /// Showing attack telegraph (warning indicator)
    Telegraph,
    /// Executing attack sequence
    Attacking,
    /// Recovery period after attack
    Recovery,
    /// Moving/repositioning
    Moving,
    /// Phase transition (invulnerable)
    PhaseTransition,
    /// Staggered/vulnerable window
    Staggered,
    /// Defeated
    Defeated,
}

/// Slots for boss special attack sequences
#[derive(Component, Debug, Clone)]
pub struct BossAttackSlots {
    /// Primary attack sequence (most common)
    pub primary: AttackSequence,
    /// Secondary attack sequence
    pub secondary: AttackSequence,
    /// Signature/ultimate attack sequence
    pub signature: AttackSequence,
    /// Phase 2+ enhanced attacks (optional)
    pub enraged: Option<AttackSequence>,
}

impl Default for BossAttackSlots {
    fn default() -> Self {
        Self {
            primary: AttackSequence::default(),
            secondary: AttackSequence {
                name: "Sweep".to_string(),
                steps: vec![
                    AttackStep::Telegraph { duration: 0.4 },
                    AttackStep::Hitbox {
                        damage: 20.0,
                        knockback: 400.0,
                        size: Vec2::new(120.0, 60.0),
                        offset: Vec2::new(0.0, 0.0),
                        duration: 0.3,
                    },
                    AttackStep::Recovery { duration: 0.5 },
                ],
                cooldown: 2.5,
            },
            signature: AttackSequence {
                name: "Slam".to_string(),
                steps: vec![
                    AttackStep::Telegraph { duration: 0.8 },
                    AttackStep::Jump {
                        height: 200.0,
                        duration: 0.5,
                    },
                    AttackStep::Hitbox {
                        damage: 40.0,
                        knockback: 600.0,
                        size: Vec2::new(200.0, 40.0),
                        offset: Vec2::new(0.0, -30.0),
                        duration: 0.2,
                    },
                    AttackStep::Recovery { duration: 1.0 },
                ],
                cooldown: 5.0,
            },
            enraged: None,
        }
    }
}

/// A sequence of attack steps that a boss executes
#[derive(Debug, Clone)]
pub struct AttackSequence {
    pub name: String,
    pub steps: Vec<AttackStep>,
    pub cooldown: f32,
}

impl Default for AttackSequence {
    fn default() -> Self {
        Self {
            name: "Basic Attack".to_string(),
            steps: vec![
                AttackStep::Telegraph { duration: 0.3 },
                AttackStep::Hitbox {
                    damage: 15.0,
                    knockback: 300.0,
                    size: Vec2::new(50.0, 50.0),
                    offset: Vec2::new(40.0, 0.0),
                    duration: 0.2,
                },
                AttackStep::Recovery { duration: 0.4 },
            ],
            cooldown: 1.5,
        }
    }
}

/// Individual step in an attack sequence
#[derive(Debug, Clone)]
pub enum AttackStep {
    /// Show warning indicator
    Telegraph { duration: f32 },
    /// Spawn a hitbox
    Hitbox {
        damage: f32,
        knockback: f32,
        size: Vec2,
        offset: Vec2,
        duration: f32,
    },
    /// Move/dash in a direction
    Move {
        direction: Vec2,
        speed: f32,
        duration: f32,
    },
    /// Jump to height
    Jump { height: f32, duration: f32 },
    /// Wait/pause
    Wait { duration: f32 },
    /// Recovery period (vulnerable)
    Recovery { duration: f32 },
    /// Spawn projectile
    Projectile {
        damage: f32,
        speed: f32,
        direction: Vec2,
    },
}

/// Tracks cooldowns for boss attack sequences
#[derive(Component, Debug, Default)]
pub struct BossAttackCooldowns {
    pub primary: f32,
    pub secondary: f32,
    pub signature: f32,
}

/// Marker for arena lock during boss fight
#[derive(Component, Debug)]
pub struct ArenaLock;

/// Visual telegraph indicator
#[derive(Component, Debug)]
pub struct TelegraphIndicator {
    pub timer: f32,
}

// ============================================================================
// Combat Tuning
// ============================================================================

#[derive(Resource, Debug, Clone)]
pub struct CombatTuning {
    pub iframes_duration: f32,
    pub stagger_duration: f32,
    pub damage_flash_duration: f32,
}

impl Default for CombatTuning {
    fn default() -> Self {
        Self {
            iframes_duration: 0.5,
            stagger_duration: 0.2,
            damage_flash_duration: 0.1,
        }
    }
}

// ============================================================================
// Input
// ============================================================================

#[derive(Resource, Debug, Default)]
pub struct CombatInput {
    pub light_attack: bool,
    pub heavy_attack: bool,
    pub special_attack: bool,
}

// ============================================================================
// Events
// ============================================================================

#[derive(Debug)]
pub struct DamageEvent {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
    pub knockback: Vec2,
}

impl Message for DamageEvent {}

#[derive(Debug)]
pub struct DeathEvent {
    pub entity: Entity,
}

impl Message for DeathEvent {}

#[derive(Debug)]
pub struct BossPhaseChangeEvent {
    pub boss: Entity,
    pub new_phase: u8,
}

impl Message for BossPhaseChangeEvent {}

#[derive(Debug)]
pub struct BossDefeatedEvent {
    pub boss: Entity,
}

impl Message for BossDefeatedEvent {}

// ============================================================================
// Plugin
// ============================================================================

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatTuning>()
            .init_resource::<AttackTuning>()
            .init_resource::<EnemyTuning>()
            .init_resource::<CombatInput>()
            .init_resource::<BossConfig>()
            .init_resource::<BossEncounterState>()
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_message::<BossPhaseChangeEvent>()
            .add_message::<BossDefeatedEvent>()
            .add_systems(
                Update,
                (
                    read_combat_input,
                    update_combat_timers,
                    process_player_attacks,
                    update_enemy_ai,
                    update_boss_ai,
                    apply_enemy_movement,
                    apply_boss_movement,
                    process_enemy_attacks,
                    process_boss_attacks,
                    detect_hitbox_collisions,
                    apply_damage,
                    apply_knockback,
                    check_boss_phase_transitions,
                    process_deaths,
                    cleanup_expired_hitboxes,
                    cleanup_telegraphs,
                )
                    .chain(),
            );
    }
}

// ============================================================================
// Enemy Spawning Helpers
// ============================================================================

/// Bundle for spawning enemies with proper tier scaling
#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub tier: EnemyTier,
    pub combatant: Combatant,
    pub team: Team,
    pub health: Health,
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
    pub fn new(tier: EnemyTier, position: Vec2, base_health: f32, _tuning: &EnemyTuning) -> Self {
        let (health_mult, _damage_mult, _speed_mult) = tier.stat_multipliers();
        let scale = tier.scale();
        let size = Vec2::new(32.0 * scale, 32.0 * scale);

        Self {
            enemy: Enemy,
            tier,
            combatant: Combatant,
            team: Team::Enemy,
            health: Health::new(base_health * health_mult),
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
            collision_layers: CollisionLayers::new(GameLayer::Enemy, [GameLayer::Ground, GameLayer::Wall]),
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

    commands
        .spawn((
            // Identity & Combat
            (
                Enemy,
                tier,
                Combatant,
                Team::Enemy,
                Health::new(final_health),
                Stagger::default(),
                Invulnerable::default(),
            ),
            // Boss AI
            (BossAI::default(), attack_slots, BossAttackCooldowns::default()),
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
                CollisionLayers::new(GameLayer::Enemy, [GameLayer::Ground, GameLayer::Wall]),
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

// ============================================================================
// Systems
// ============================================================================

fn read_combat_input(keyboard: Res<ButtonInput<KeyCode>>, mut input: ResMut<CombatInput>) {
    input.light_attack =
        keyboard.just_pressed(KeyCode::KeyZ) || keyboard.just_pressed(KeyCode::KeyU);
    input.heavy_attack =
        keyboard.just_pressed(KeyCode::KeyX) || keyboard.just_pressed(KeyCode::KeyI);
    input.special_attack =
        keyboard.just_pressed(KeyCode::KeyC) || keyboard.just_pressed(KeyCode::KeyO);
}

fn update_combat_timers(
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

fn process_player_attacks(
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

            commands.spawn((
                Hitbox {
                    damage,
                    knockback,
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
            ));
        }
    }
}

#[derive(Component)]
pub struct HitboxLifetime(pub f32);

fn cleanup_expired_hitboxes(
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

fn update_enemy_ai(
    time: Res<Time>,
    tuning: Res<EnemyTuning>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut EnemyAI, &Stagger, &EnemyTier),
        (With<Enemy>, Without<BossAI>),
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

fn update_boss_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut boss_query: Query<(
        &Transform,
        &mut BossAI,
        &BossAttackSlots,
        &mut BossAttackCooldowns,
        &Health,
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

fn apply_enemy_movement(
    tuning: Res<EnemyTuning>,
    player_query: Query<&Transform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut LinearVelocity, &EnemyAI, &EnemyTier),
        (With<Enemy>, Without<Player>, Without<BossAI>),
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

fn apply_boss_movement(
    player_query: Query<&Transform, With<Player>>,
    mut boss_query: Query<(&Transform, &mut LinearVelocity, &BossAI), (With<Enemy>, Without<Player>)>,
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

fn process_enemy_attacks(
    mut commands: Commands,
    tuning: Res<EnemyTuning>,
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(Entity, &Transform, &EnemyAI, &EnemyTier), (With<Enemy>, Without<BossAI>)>,
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
            ));
        }
    }
}

fn process_boss_attacks(
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
            ));
        }
    }
}

fn detect_hitbox_collisions(
    mut collision_events: MessageReader<CollisionStart>,
    mut damage_events: MessageWriter<DamageEvent>,
    mut hitbox_query: Query<(&mut Hitbox, &Team)>,
    target_query: Query<(Entity, &Team, &Invulnerable), With<Combatant>>,
) {
    for event in collision_events.read() {
        let pairs = [
            (event.collider1, event.collider2),
            (event.collider2, event.collider1),
        ];

        for (hitbox_entity, target_entity) in pairs {
            if let Ok((mut hitbox, hitbox_team)) = hitbox_query.get_mut(hitbox_entity) {
                if let Ok((target, target_team, invuln)) = target_query.get(target_entity) {
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

                    let knockback_dir = Vec2::X;

                    damage_events.write(DamageEvent {
                        source: hitbox.owner,
                        target,
                        amount: hitbox.damage,
                        knockback: knockback_dir * hitbox.knockback,
                    });
                }
            }
        }
    }
}

fn apply_damage(
    mut damage_events: MessageReader<DamageEvent>,
    mut death_events: MessageWriter<DeathEvent>,
    tuning: Res<CombatTuning>,
    mut query: Query<(&mut Health, &mut Stagger, &mut Invulnerable, &mut Sprite)>,
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

fn apply_knockback(
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

fn check_boss_phase_transitions(
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

fn process_deaths(
    mut commands: Commands,
    mut death_events: MessageReader<DeathEvent>,
    mut boss_defeated_events: MessageWriter<BossDefeatedEvent>,
    mut boss_state: ResMut<BossEncounterState>,
    enemy_query: Query<(Entity, Option<&BossAI>), With<Enemy>>,
) {
    for event in death_events.read() {
        if let Ok((entity, boss_ai)) = enemy_query.get(event.entity) {
            if boss_ai.is_some() {
                // Boss defeated
                boss_state.boss_defeated();
                boss_defeated_events.write(BossDefeatedEvent { boss: entity });
            }
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_telegraphs(
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
