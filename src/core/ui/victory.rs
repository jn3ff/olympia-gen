//! Core domain: victory screen UI and input handling.

use bevy::prelude::*;

use crate::content::ContentRegistry;
use crate::core::{GameState, SegmentProgress};
use crate::rewards::RunFaith;

/// Marker for victory screen UI root
#[derive(Component, Debug)]
pub struct VictoryScreenUI;

pub(crate) fn spawn_victory_screen(
    mut commands: Commands,
    segment_progress: Res<SegmentProgress>,
    run_faith: Res<RunFaith>,
    content_registry: Option<Res<ContentRegistry>>,
) {
    let bg_color = Color::srgba(0.02, 0.05, 0.1, 0.98);
    let title_color = Color::srgb(0.95, 0.85, 0.3);
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let muted_text = Color::srgb(0.6, 0.6, 0.7);
    let positive_color = Color::srgb(0.4, 0.9, 0.4);
    let negative_color = Color::srgb(0.9, 0.4, 0.4);

    // Get faith summary
    let faith_summary = run_faith.get_faith_summary();

    commands
        .spawn((
            VictoryScreenUI,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg_color),
            ZIndex(200),
        ))
        .with_children(|parent| {
            // Victory title
            parent.spawn((
                Text::new("VICTORY"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(title_color),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("The Gods Smile Upon You"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Stats
            parent.spawn((
                Text::new(format!(
                    "Bosses Defeated: {}",
                    segment_progress.total_bosses_defeated
                )),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Faith section header (only if there are changes)
            if !faith_summary.is_empty() {
                parent.spawn((
                    Text::new("Divine Favor"),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(title_color),
                    Node {
                        margin: UiRect::new(
                            Val::Px(0.0),
                            Val::Px(0.0),
                            Val::Px(20.0),
                            Val::Px(10.0),
                        ),
                        ..default()
                    },
                ));

                // Faith container
                parent
                    .spawn((Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(30.0)),
                        ..default()
                    },))
                    .with_children(|faith_container| {
                        for summary in &faith_summary {
                            // Get god name from registry or use ID
                            let god_name = content_registry
                                .as_ref()
                                .and_then(|reg| reg.gods.get(&summary.god_id))
                                .map(|g| g.name.clone())
                                .unwrap_or_else(|| summary.god_id.clone());

                            let delta_str = if summary.total_delta >= 0 {
                                format!("+{}", summary.total_delta)
                            } else {
                                format!("{}", summary.total_delta)
                            };

                            let color = if summary.total_delta >= 0 {
                                positive_color
                            } else {
                                negative_color
                            };

                            faith_container.spawn((
                                Text::new(format!(
                                    "{}: {} ({})",
                                    god_name, summary.final_value, delta_str
                                )),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(color),
                                Node {
                                    margin: UiRect::bottom(Val::Px(5.0)),
                                    ..default()
                                },
                            ));
                        }
                    });

                // Show adversarial events that were triggered
                let triggered_count = run_faith.triggered_adversarial.len();
                if triggered_count > 0 {
                    parent.spawn((
                        Text::new(format!("Adversarial Events Faced: {}", triggered_count)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(negative_color),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            ..default()
                        },
                    ));
                }
            }

            // Instructions
            parent.spawn((
                Text::new("Press ENTER to return to character select"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(muted_text),
            ));
        });
}

pub(crate) fn cleanup_victory_screen(
    mut commands: Commands,
    query: Query<Entity, With<VictoryScreenUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn handle_victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        game_state.set(GameState::CharacterSelect);
    }
}
