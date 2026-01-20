//! UI domain: coin display HUD element.

use bevy::prelude::*;

use crate::rewards::PlayerWallet;
use crate::ui::hud_player::{PLAYER_HEALTHBAR_HEIGHT, PLAYER_HEALTHBAR_PADDING};

/// Marker for the coin display UI container
#[derive(Component)]
pub struct CoinDisplayUI;

/// Marker for the coin amount text
#[derive(Component)]
pub struct CoinAmountText;

pub(crate) fn spawn_coin_display_ui(mut commands: Commands) {
    // Position below the health bar
    commands
        .spawn((
            CoinDisplayUI,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(PLAYER_HEALTHBAR_PADDING),
                top: Val::Px(PLAYER_HEALTHBAR_PADDING + PLAYER_HEALTHBAR_HEIGHT + 8.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Coin icon (gold square)
            parent.spawn((
                Node {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.9, 0.75, 0.2)),
            ));

            // Coin amount text
            parent.spawn((
                CoinAmountText,
                Text::new("0"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.85, 0.5)),
            ));
        });
}

pub(crate) fn update_coin_display(
    wallet: Res<PlayerWallet>,
    mut query: Query<&mut Text, With<CoinAmountText>>,
) {
    if wallet.is_changed() {
        for mut text in &mut query {
            **text = format!("{}", wallet.coins);
        }
    }
}
