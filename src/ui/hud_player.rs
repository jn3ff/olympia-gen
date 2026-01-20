//! UI domain: player HUD health bar.

use bevy::prelude::*;

use crate::combat::Health;
use crate::movement::Player;

pub(crate) const PLAYER_HEALTHBAR_WIDTH: f32 = 200.0;
pub(crate) const PLAYER_HEALTHBAR_HEIGHT: f32 = 20.0;
pub(crate) const PLAYER_HEALTHBAR_PADDING: f32 = 16.0;

/// Marker for the player's HUD health bar container
#[derive(Component)]
pub struct PlayerHealthBarUI;

/// Marker for the player's health bar fill element
#[derive(Component)]
pub struct PlayerHealthBarFill;

pub(crate) fn spawn_player_healthbar_ui(mut commands: Commands) {
    // Root container positioned at top-left
    commands
        .spawn((
            PlayerHealthBarUI,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PLAYER_HEALTHBAR_PADDING),
                top: Val::Px(PLAYER_HEALTHBAR_PADDING),
                width: Val::Px(PLAYER_HEALTHBAR_WIDTH),
                height: Val::Px(PLAYER_HEALTHBAR_HEIGHT),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|parent| {
            // Health bar fill
            parent.spawn((
                PlayerHealthBarFill,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.3)),
            ));
        });
}

pub(crate) fn update_player_healthbar(
    player_query: Query<&Health, With<Player>>,
    mut fill_query: Query<(&mut Node, &mut BackgroundColor), With<PlayerHealthBarFill>>,
) {
    let Ok(health) = player_query.single() else {
        return;
    };

    for (mut node, mut bg_color) in &mut fill_query {
        let percent = health.percent();
        node.width = Val::Percent(percent * 100.0);

        // Color gradient: green -> yellow -> red
        let color = if percent > 0.5 {
            let t = (percent - 0.5) * 2.0;
            Color::srgb(1.0 - t * 0.8, 0.8, 0.3 * (1.0 - t))
        } else {
            let t = percent * 2.0;
            Color::srgb(0.9, 0.2 + t * 0.6, 0.2)
        };
        bg_color.0 = color;
    }
}
