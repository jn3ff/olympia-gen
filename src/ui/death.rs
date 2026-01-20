//! UI domain: death screen presentation and retry flow.

use bevy::prelude::*;

use crate::combat::Health;
use crate::core::RunState;
use crate::movement::Player;

/// Marker for the death screen overlay
#[derive(Component)]
pub struct DeathScreenUI;

/// Marker for the retry button on death screen
#[derive(Component)]
pub struct RetryButton;

/// Resource to track if player has died (prevents multiple death screens)
#[derive(Resource, Default)]
pub struct PlayerDeathState {
    pub is_dead: bool,
}

pub(crate) fn detect_player_death(
    mut commands: Commands,
    player_query: Query<&Health, With<Player>>,
    mut death_state: ResMut<PlayerDeathState>,
    existing_death_screen: Query<Entity, With<DeathScreenUI>>,
) {
    // Skip if death screen already shown
    if death_state.is_dead {
        return;
    }

    // Check if player health is zero or below
    let Ok(health) = player_query.single() else {
        return;
    };

    if health.is_dead() {
        death_state.is_dead = true;

        // Only spawn if not already showing
        if existing_death_screen.is_empty() {
            spawn_death_screen(&mut commands);
        }
    }
}

fn spawn_death_screen(commands: &mut Commands) {
    // Full screen dark overlay
    commands
        .spawn((
            DeathScreenUI,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            // High z-index to be on top of everything
            ZIndex(100),
        ))
        .with_children(|parent| {
            // "YOU DIED" text
            parent.spawn((
                Text::new("YOU DIED"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.15, 0.15)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Subtext - leaving room for narrative
            parent.spawn((
                Text::new("Your journey ends here... for now."),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    margin: UiRect::bottom(Val::Px(60.0)),
                    ..default()
                },
            ));

            // Retry button
            parent
                .spawn((
                    RetryButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(40.0), Val::Px(16.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                ))
                .with_child((
                    Text::new("RETRY"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));

            // Hint text
            parent.spawn((
                Text::new("Press [Enter] or click to retry"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.45)),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

pub(crate) fn handle_retry_button(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<&Interaction, (With<RetryButton>, Changed<Interaction>)>,
    death_screen_query: Query<Entity, With<DeathScreenUI>>,
    mut death_state: ResMut<PlayerDeathState>,
    mut player_query: Query<&mut Health, With<Player>>,
    mut next_state: ResMut<NextState<RunState>>,
) {
    // Check if we should retry
    let should_retry = keyboard.just_pressed(KeyCode::Enter)
        || keyboard.just_pressed(KeyCode::NumpadEnter)
        || button_query
            .iter()
            .any(|interaction| *interaction == Interaction::Pressed);

    if !should_retry || !death_state.is_dead {
        return;
    }

    // Reset death state
    death_state.is_dead = false;

    // Despawn death screen
    for entity in &death_screen_query {
        commands.entity(entity).despawn();
    }

    // Reset player health
    for mut health in &mut player_query {
        health.current = health.max;
    }

    // Return to arena for a fresh start
    next_state.set(RunState::Arena);
}
