//! Core domain: character selection UI and input handling.

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::content::ContentRegistry;
use crate::core::{CharacterSelectedEvent, GameState, RunState, SelectedCharacter};

/// Marker for the character selection UI root
#[derive(Component, Debug)]
pub struct CharacterSelectUI;

/// Button for selecting a specific character
#[derive(Component, Debug)]
pub struct CharacterSelectButton {
    pub character_id: String,
}

/// UI component showing character name
#[derive(Component, Debug)]
pub struct CharacterNameText;

/// UI component showing character description
#[derive(Component, Debug)]
pub struct CharacterDescText;

pub(crate) fn spawn_character_select_ui(
    mut commands: Commands,
    registry: Option<Res<ContentRegistry>>,
) {
    let bg_color = Color::srgba(0.05, 0.05, 0.1, 0.98);
    let panel_color = Color::srgb(0.12, 0.12, 0.18);
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let muted_text = Color::srgb(0.6, 0.6, 0.7);
    let title_color = Color::srgb(0.9, 0.75, 0.3);

    // Define character colors based on their parent god
    let character_colors = [
        ("character_ares_sword", Color::srgb(0.85, 0.25, 0.25)), // Ares - Red
        ("character_demeter_sword", Color::srgb(0.3, 0.7, 0.35)), // Demeter - Green
        ("character_poseidon_spear", Color::srgb(0.25, 0.5, 0.85)), // Poseidon - Blue
        ("character_zeus_spear", Color::srgb(0.85, 0.75, 0.25)), // Zeus - Gold
    ];

    // Get characters from registry if available
    let characters: Vec<(String, String, String, Color)> = if let Some(reg) = registry {
        character_colors
            .iter()
            .filter_map(|(id, color)| {
                reg.characters.get(*id).map(|char_def| {
                    let god_name = reg
                        .gods
                        .get(&char_def.parent_god_id)
                        .map(|g| g.name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    (
                        char_def.id.clone(),
                        char_def.name.clone(),
                        format!("Child of {}", god_name),
                        *color,
                    )
                })
            })
            .collect()
    } else {
        // Fallback if registry not loaded
        vec![
            (
                "character_ares_sword".to_string(),
                "Child of Ares".to_string(),
                "Sword - Aggression".to_string(),
                character_colors[0].1,
            ),
            (
                "character_demeter_sword".to_string(),
                "Child of Demeter".to_string(),
                "Sword - Frost".to_string(),
                character_colors[1].1,
            ),
            (
                "character_poseidon_spear".to_string(),
                "Child of Poseidon".to_string(),
                "Spear - Tides".to_string(),
                character_colors[2].1,
            ),
            (
                "character_zeus_spear".to_string(),
                "Child of Zeus".to_string(),
                "Spear - Lightning".to_string(),
                character_colors[3].1,
            ),
        ]
    };

    // Root container
    commands
        .spawn((
            CharacterSelectUI,
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
            ZIndex(100),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("OLYMPIA"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(title_color),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("Choose Your Champion"),
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

            // Character selection container
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(20.0),
                    ..default()
                },))
                .with_children(|chars_parent| {
                    for (index, (char_id, char_name, char_desc, char_color)) in
                        characters.iter().enumerate()
                    {
                        spawn_character_card(
                            chars_parent,
                            index,
                            char_id,
                            char_name,
                            char_desc,
                            *char_color,
                            panel_color,
                            text_color,
                            muted_text,
                        );
                    }
                });

            // Instructions
            parent.spawn((
                Text::new("Press 1-4 or click to select"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_character_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    char_id: &str,
    char_name: &str,
    char_desc: &str,
    char_color: Color,
    panel_color: Color,
    text_color: Color,
    muted_text: Color,
) {
    let key_hint = format!("[{}]", index + 1);

    parent
        .spawn((
            CharacterSelectButton {
                character_id: char_id.to_string(),
            },
            Button,
            Node {
                width: Val::Px(180.0),
                min_height: Val::Px(240.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BorderColor::all(char_color.with_alpha(0.6)),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            // Key hint
            card.spawn((
                Text::new(key_hint),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Character icon (colored square)
            card.spawn((
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    margin: UiRect::bottom(Val::Px(15.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(char_color),
                BackgroundColor(char_color.with_alpha(0.3)),
            ));

            // Character name
            card.spawn((
                CharacterNameText,
                Text::new(char_name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Character description
            card.spawn((
                CharacterDescText,
                Text::new(char_desc),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(muted_text),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

pub(crate) fn cleanup_character_select_ui(
    mut commands: Commands,
    query: Query<Entity, With<CharacterSelectUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn handle_character_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut selected_character: ResMut<SelectedCharacter>,
    mut char_events: MessageWriter<CharacterSelectedEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut run_state: ResMut<NextState<RunState>>,
) {
    let character_ids = [
        "character_ares_sword",
        "character_demeter_sword",
        "character_poseidon_spear",
        "character_zeus_spear",
    ];

    let selected = if keyboard.just_pressed(KeyCode::Digit1)
        || keyboard.just_pressed(KeyCode::Numpad1)
    {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        Some(2)
    } else if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4) {
        Some(3)
    } else {
        None
    };

    if let Some(index) = selected {
        let char_id = character_ids[index];
        selected_character.select(char_id);
        char_events.write(CharacterSelectedEvent {
            character_id: char_id.to_string(),
        });
        info!("Character selected via keyboard: {}", char_id);

        // Transition to run
        game_state.set(GameState::Run);
        run_state.set(RunState::Arena);
    }
}

pub(crate) fn handle_character_select_click(
    mut button_query: Query<
        (
            &CharacterSelectButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
    mut selected_character: ResMut<SelectedCharacter>,
    mut char_events: MessageWriter<CharacterSelectedEvent>,
    mut game_state: ResMut<NextState<GameState>>,
    mut run_state: ResMut<NextState<RunState>>,
) {
    for (button, interaction, mut bg_color, mut border_color) in &mut button_query {
        match interaction {
            Interaction::Pressed => {
                selected_character.select(&button.character_id);
                char_events.write(CharacterSelectedEvent {
                    character_id: button.character_id.clone(),
                });
                info!("Character selected via click: {}", button.character_id);

                // Transition to run
                game_state.set(GameState::Run);
                run_state.set(RunState::Arena);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.18, 0.18, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.7, 0.7, 0.8));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.18));
                // Border color will be reset based on character
            }
        }
    }
}
