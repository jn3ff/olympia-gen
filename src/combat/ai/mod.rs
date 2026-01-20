//! Combat domain: AI system modules for enemies and bosses.

pub(crate) mod boss;
pub(crate) mod enemy;

pub(crate) use boss::{apply_boss_movement, process_boss_attacks, update_boss_ai};
pub(crate) use enemy::{apply_enemy_movement, process_enemy_attacks, update_enemy_ai};
