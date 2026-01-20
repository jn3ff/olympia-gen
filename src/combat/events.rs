//! Combat domain: combat-related events.

use bevy::ecs::message::Message;
use bevy::prelude::*;

/// Event emitted when a parry successfully deflects an attack
#[derive(Debug)]
pub struct ParrySuccessEvent {
    pub parrier: Entity,
    pub attacker: Entity,
}

impl Message for ParrySuccessEvent {}

/// Event emitted when stance is broken
#[derive(Debug)]
pub struct StanceBreakEvent {
    pub entity: Entity,
    pub breaker: Entity,
}

impl Message for StanceBreakEvent {}

#[derive(Debug)]
pub struct DamageEvent {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
    pub knockback: Vec2,
    pub stance_damage: f32,
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
