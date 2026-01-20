//! Combat domain: components and combat-related state types.

use bevy::prelude::*;

use crate::combat::attacks::{AttackDirection, AttackSequence, AttackStep, AttackType};

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
    pub stance_damage: f32,
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

/// Baseline DPS assumption for significance calculation (~2 lights/sec at 8 damage)
pub const BASELINE_DPS: f32 = 20.0;
/// Enemies with health pools taking longer than this to defeat are "significant"
pub const SIGNIFICANT_THRESHOLD_SECONDS: f32 = 30.0;

/// Identifies an enemy by its definition, used for no-repeat logic and narrative hooks.
#[derive(Component, Debug, Clone)]
pub struct EnemyIdentity {
    /// The id from EnemyDef in content registry
    pub def_id: String,
    /// Whether this enemy is "significant" (health pool > 30s at baseline DPS)
    /// Significant enemies don't repeat within a run
    pub is_significant: bool,
}

impl EnemyIdentity {
    /// Create identity for a non-significant enemy
    pub fn minor(def_id: impl Into<String>) -> Self {
        Self {
            def_id: def_id.into(),
            is_significant: false,
        }
    }

    /// Create identity for a significant enemy
    pub fn significant(def_id: impl Into<String>) -> Self {
        Self {
            def_id: def_id.into(),
            is_significant: true,
        }
    }

    /// Calculate significance based on effective health pool
    pub fn from_health(def_id: impl Into<String>, effective_health: f32) -> Self {
        let time_to_kill = effective_health / BASELINE_DPS;
        Self {
            def_id: def_id.into(),
            is_significant: time_to_kill > SIGNIFICANT_THRESHOLD_SECONDS,
        }
    }
}

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

/// Per-slot skill references for the player
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

/// References a moveset by id from the ContentRegistry.
/// Player attacks will use this moveset's strike definitions.
#[derive(Component, Debug, Clone)]
pub struct PlayerMoveset {
    pub moveset_id: String,
}

impl Default for PlayerMoveset {
    fn default() -> Self {
        Self {
            moveset_id: "moveset_sword_basic".to_string(),
        }
    }
}

/// Tracks the player's current position in attack combos.
#[derive(Component, Debug, Default)]
pub struct ComboState {
    /// Current index in the light combo chain
    pub light_index: usize,
    /// Current index in the heavy combo chain
    pub heavy_index: usize,
    /// Time remaining in combo window before reset
    pub combo_window: f32,
    /// Time remaining in current attack animation
    pub attack_timer: f32,
    /// Cooldown before next attack can be input
    pub cooldown_timer: f32,
    /// Which attack type is currently active
    pub active_attack: Option<ActiveAttackType>,
}

/// The type of attack currently being executed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveAttackType {
    Light,
    Heavy,
    Special,
    Parry,
}

/// Stance meter for enemies - when depleted, enemy becomes vulnerable.
/// Based on gameplay_defaults.ron: 4 heavies or 7 lights to break.
#[derive(Component, Debug, Clone)]
pub struct Stance {
    /// Current stance points remaining
    pub current: f32,
    /// Maximum stance points
    pub max: f32,
    /// Timer for stance regeneration (regen 1 light worth per 10s)
    pub regen_timer: f32,
    /// Stance damage value of 1 light attack
    pub light_stance_value: f32,
    /// Stance damage value of 1 heavy attack
    pub heavy_stance_value: f32,
    /// If true, stance is broken and entity is vulnerable
    pub is_broken: bool,
    /// Duration of the stance break vulnerable state
    pub break_duration: f32,
    /// Timer for stance break recovery
    pub break_timer: f32,
}

impl Stance {
    /// Create stance with default game values.
    /// Max stance = 7 lights = 4 heavies (from gameplay_defaults)
    pub fn new() -> Self {
        // 7 lights to break, so light value = 1, max = 7
        // 4 heavies to break, so heavy value = 7/4 = 1.75
        Self {
            current: 7.0,
            max: 7.0,
            regen_timer: 0.0,
            light_stance_value: 1.0,
            heavy_stance_value: 1.75,
            is_broken: false,
            break_duration: 2.0, // 2 second vulnerability window
            break_timer: 0.0,
        }
    }

    /// Create stance from gameplay defaults
    pub fn from_defaults(light_break: u32, heavy_break: u32, break_duration: f32) -> Self {
        let max = light_break as f32;
        let heavy_value = max / heavy_break as f32;
        Self {
            current: max,
            max,
            regen_timer: 0.0,
            light_stance_value: 1.0,
            heavy_stance_value: heavy_value,
            is_broken: false,
            break_duration,
            break_timer: 0.0,
        }
    }

    /// Apply stance damage. Returns true if stance was just broken.
    pub fn take_damage(&mut self, amount: f32) -> bool {
        if self.is_broken {
            return false;
        }
        self.current = (self.current - amount).max(0.0);
        self.regen_timer = 0.0; // Reset regen timer on hit
        if self.current <= 0.0 {
            self.is_broken = true;
            self.break_timer = self.break_duration;
            true
        } else {
            false
        }
    }

    /// Regenerate stance over time
    pub fn regenerate(&mut self, dt: f32, regen_rate: f32) {
        if self.is_broken {
            return;
        }
        self.regen_timer += dt;
        // regen_rate is seconds per 1 light worth of stance (default 10.0)
        if self.regen_timer >= regen_rate && self.current < self.max {
            self.current = (self.current + 1.0).min(self.max);
            self.regen_timer = 0.0;
        }
    }

    /// Update break state timer
    pub fn update_break(&mut self, dt: f32) {
        if self.is_broken {
            self.break_timer -= dt;
            if self.break_timer <= 0.0 {
                self.is_broken = false;
                self.current = self.max; // Fully restore stance on recovery
                self.regen_timer = 0.0;
            }
        }
    }

    /// Get stance as a percentage for UI
    pub fn percent(&self) -> f32 {
        self.current / self.max
    }
}

impl Default for Stance {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks parry state and timing window.
#[derive(Component, Debug, Default)]
pub struct ParryState {
    /// Is the parry window currently active?
    pub is_active: bool,
    /// Time remaining in the parry window
    pub window_timer: f32,
    /// Cooldown before parry can be used again
    pub cooldown_timer: f32,
    /// Duration of the parry window (from moveset)
    pub window_duration: f32,
    /// Cooldown after parry attempt
    pub cooldown_duration: f32,
}

impl ParryState {
    pub fn new(window_duration: f32) -> Self {
        Self {
            is_active: false,
            window_timer: 0.0,
            cooldown_timer: 0.0,
            window_duration,
            cooldown_duration: 0.5, // Half second cooldown between parry attempts
        }
    }

    /// Start a parry attempt
    pub fn start_parry(&mut self) {
        if self.cooldown_timer <= 0.0 {
            self.is_active = true;
            self.window_timer = self.window_duration;
        }
    }

    /// Update parry timers
    pub fn update(&mut self, dt: f32) {
        if self.is_active {
            self.window_timer -= dt;
            if self.window_timer <= 0.0 {
                self.is_active = false;
                self.cooldown_timer = self.cooldown_duration;
            }
        } else if self.cooldown_timer > 0.0 {
            self.cooldown_timer -= dt;
        }
    }

    /// Check if parry is currently active and can deflect
    pub fn can_parry(&self) -> bool {
        self.is_active && self.window_timer > 0.0
    }
}

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

/// Entity lifetime for temporary hitboxes
#[derive(Component)]
pub struct HitboxLifetime(pub f32);
