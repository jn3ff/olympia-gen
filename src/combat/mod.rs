use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Combatant;

#[derive(Component, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component, Debug, Default)]
pub struct Stagger {
    pub timer: f32,
}

#[derive(Component, Debug, Default)]
pub struct Invulnerable {
    pub timer: f32,
}

#[derive(Component, Debug)]
pub struct Hitbox;

#[derive(Component, Debug)]
pub struct Hurtbox;

#[derive(Event, Debug)]
pub struct DamageEvent {
    pub source: Entity,
    pub target: Entity,
    pub amount: f32,
    pub knockback: Vec2,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_systems(Update, apply_damage);
    }
}

fn apply_damage() {}
