//! UI domain: in-run HUD elements and death flow.

mod death;
mod hud_boss;
mod hud_enemy;
mod hud_player;
mod hud_wallet;

pub use death::PlayerDeathState;

use bevy::prelude::*;

use crate::ui::death::{detect_player_death, handle_retry_button};
use crate::ui::hud_boss::{cleanup_boss_healthbar, spawn_boss_healthbar, update_boss_healthbar};
use crate::ui::hud_enemy::{
    cleanup_enemy_healthbars, spawn_enemy_healthbars, update_enemy_healthbars,
};
use crate::ui::hud_player::{spawn_player_healthbar_ui, update_player_healthbar};
use crate::ui::hud_wallet::{spawn_coin_display_ui, update_coin_display};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerDeathState>()
            .add_systems(Startup, (spawn_player_healthbar_ui, spawn_coin_display_ui))
            .add_systems(
                Update,
                (
                    update_player_healthbar,
                    update_coin_display,
                    spawn_enemy_healthbars,
                    update_enemy_healthbars,
                    cleanup_enemy_healthbars,
                    spawn_boss_healthbar,
                    update_boss_healthbar,
                    cleanup_boss_healthbar,
                    detect_player_death,
                    handle_retry_button,
                ),
            );
    }
}
