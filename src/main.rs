mod combat;
mod content;
mod core;
mod movement;
mod rewards;
mod rooms;
mod ui;

use avian2d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Olympia".to_string(),
                resolution: (1280, 720).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins((
            core::CorePlugin,
            content::ContentPlugin,
            movement::MovementPlugin,
            combat::CombatPlugin,
            rooms::RoomsPlugin,
            rewards::RewardsPlugin,
            ui::UiPlugin,
        ))
        .run();
}
