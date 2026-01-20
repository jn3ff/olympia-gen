//! Combat domain: attacks, damage handling, and AI systems.

mod ai;
mod attacks;
mod components;
mod events;
mod resources;
mod spawn;
mod systems;

pub use components::{
    ArenaLock, AttackState, BASELINE_DPS, BossAI, BossAttackSlots, Combatant, ComboState, Enemy,
    EnemyTier, Health, Invulnerable, ParryState, PlayerMoveset, SIGNIFICANT_THRESHOLD_SECONDS,
    SkillSlots, Stagger, Team, Weapon,
};
pub use events::{
    BossDefeatedEvent, BossPhaseChangeEvent, DamageEvent, DeathEvent, ParrySuccessEvent,
    StanceBreakEvent,
};
pub use resources::{
    AttackTuning, BossConfig, BossEncounterState, CombatInput, CombatTuning, EnemyTuning,
    GoldDropConfig,
};
pub use spawn::{EnemyBundle, spawn_boss_scaled};

use bevy::prelude::*;

use crate::combat::ai::{
    apply_boss_movement, apply_enemy_movement, process_boss_attacks, process_enemy_attacks,
    update_boss_ai, update_enemy_ai,
};
use crate::combat::systems::{
    apply_damage, apply_knockback, apply_stance_damage, check_boss_phase_transitions,
    check_parry_collisions, cleanup_expired_hitboxes, cleanup_telegraphs, detect_hitbox_collisions,
    handle_enemy_coin_drops, process_deaths, process_parry_input,
    process_player_attacks_from_moveset, process_stance_breaks, read_combat_input,
    update_combat_timers, update_combo_state, update_parry_state, update_stance,
};
use crate::core::gameplay_active;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CombatTuning>()
            .init_resource::<AttackTuning>()
            .init_resource::<EnemyTuning>()
            .init_resource::<CombatInput>()
            .init_resource::<BossConfig>()
            .init_resource::<BossEncounterState>()
            .init_resource::<GoldDropConfig>()
            .add_message::<DamageEvent>()
            .add_message::<DeathEvent>()
            .add_message::<BossPhaseChangeEvent>()
            .add_message::<BossDefeatedEvent>()
            .add_message::<ParrySuccessEvent>()
            .add_message::<StanceBreakEvent>()
            // Input and timer updates
            .add_systems(
                Update,
                (
                    read_combat_input,
                    update_combat_timers,
                    update_combo_state,
                    update_parry_state,
                    update_stance,
                )
                    .chain()
                    .run_if(gameplay_active),
            )
            // Player combat systems
            .add_systems(
                Update,
                (process_player_attacks_from_moveset, process_parry_input)
                    .chain()
                    .after(read_combat_input)
                    .run_if(gameplay_active),
            )
            // Enemy AI systems
            .add_systems(
                Update,
                (
                    update_enemy_ai,
                    update_boss_ai,
                    apply_enemy_movement,
                    apply_boss_movement,
                    process_enemy_attacks,
                    process_boss_attacks,
                )
                    .chain()
                    .after(update_combat_timers)
                    .run_if(gameplay_active),
            )
            // Collision and damage systems
            .add_systems(
                Update,
                (
                    detect_hitbox_collisions,
                    check_parry_collisions,
                    apply_damage,
                    apply_stance_damage,
                    apply_knockback,
                    check_boss_phase_transitions,
                    process_stance_breaks,
                    process_deaths,
                )
                    .chain()
                    .after(process_enemy_attacks)
                    .after(process_boss_attacks),
            )
            // Cleanup systems
            .add_systems(
                Update,
                (cleanup_expired_hitboxes, cleanup_telegraphs).after(process_deaths),
            )
            // Coin drops on enemy death
            .add_systems(Update, handle_enemy_coin_drops.after(process_deaths));
    }
}
