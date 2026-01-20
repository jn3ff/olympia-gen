//! Rewards domain: reward selection UI components and handlers.

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::core::RunState;
use crate::rewards::choices::{CurrentRewardChoices, RewardChosenEvent};
use crate::rewards::types::RewardKind;

/// Marker for the reward selection UI root
#[derive(Component, Debug)]
pub struct RewardUI;

/// Marker for a reward choice button
#[derive(Component, Debug)]
pub struct RewardChoiceButton {
    pub index: usize,
}

/// Marker for the skip reward button
#[derive(Component, Debug)]
pub struct SkipRewardButton;

/// Spawn the reward selection UI
pub(crate) fn spawn_reward_ui(mut commands: Commands, current_choices: Res<CurrentRewardChoices>) {
    let bg_color = Color::srgba(0.1, 0.1, 0.15, 0.95);
    let panel_color = Color::srgb(0.15, 0.15, 0.2);
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let muted_text = Color::srgb(0.6, 0.6, 0.7);

    commands
        .spawn((
            RewardUI,
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
            parent.spawn((
                Text::new("VICTORY!"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("Choose Your Reward"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Stretch,
                    column_gap: Val::Px(20.0),
                    ..default()
                },))
                .with_children(|choices_parent| {
                    for (index, choice) in current_choices.choices.iter().enumerate() {
                        spawn_reward_card(
                            choices_parent,
                            index,
                            choice,
                            panel_color,
                            text_color,
                            muted_text,
                        );
                    }
                });

            parent
                .spawn((
                    SkipRewardButton,
                    Button,
                    Node {
                        margin: UiRect::top(Val::Px(30.0)),
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.4, 0.4, 0.5)),
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                ))
                .with_child((
                    Text::new("Skip [Esc]"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(muted_text),
                ));

            parent.spawn((
                Text::new("Press 1, 2, or 3 to select"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_reward_card(
    parent: &mut ChildSpawnerCommands,
    index: usize,
    reward: &RewardKind,
    panel_color: Color,
    text_color: Color,
    muted_text: Color,
) {
    let icon_color = reward.icon_color();
    let tier = reward.tier();
    let tier_accent = reward.tier_accent_color();
    let key_hint = format!("[{}]", index + 1);

    let border_thickness = 2.0 + (tier.level() as f32 - 1.0) * 0.5;

    parent
        .spawn((
            RewardChoiceButton { index },
            Button,
            Node {
                width: Val::Px(220.0),
                min_height: Val::Px(300.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(15.0)),
                border: UiRect::all(Val::Px(border_thickness)),
                ..default()
            },
            BorderColor::all(tier_accent),
            BackgroundColor(panel_color),
        ))
        .with_children(|card| {
            card.spawn((
                Text::new(key_hint),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(tier.display_name().to_uppercase()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(tier_accent),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            card.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    margin: UiRect::bottom(Val::Px(12.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(tier_accent),
                BackgroundColor(icon_color),
            ));

            let type_label = match reward {
                RewardKind::SkillTreeNode { .. } => "SKILL",
                RewardKind::Equipment { slot, .. } => slot.name(),
                RewardKind::StatUpgrade { .. } => "STAT BOOST",
            };

            card.spawn((
                Text::new(type_label.to_uppercase()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(icon_color),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(reward.name()),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(text_color),
                TextLayout::new_with_justify(Justify::Center),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            card.spawn((
                Text::new(reward.description()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(muted_text),
                TextLayout::new_with_justify(Justify::Center),
            ));
        });
}

pub(crate) fn cleanup_reward_ui(mut commands: Commands, query: Query<Entity, With<RewardUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn handle_reward_choice_interaction(
    mut choice_query: Query<
        (
            &RewardChoiceButton,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, Without<SkipRewardButton>),
    >,
    mut skip_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<SkipRewardButton>, Changed<Interaction>),
    >,
    mut reward_events: MessageWriter<RewardChosenEvent>,
    mut next_run_state: ResMut<NextState<RunState>>,
) {
    for (button, interaction, mut bg_color, mut border_color) in &mut choice_query {
        match interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.4, 0.5));
                reward_events.write(RewardChosenEvent {
                    choice_index: button.index,
                });
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.25, 0.35));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
                *border_color = BorderColor::all(Color::srgb(0.3, 0.3, 0.4));
            }
        }
    }

    for (interaction, mut bg_color, mut border_color) in &mut skip_query {
        match interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.35));
                next_run_state.set(RunState::Arena);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
                *border_color = BorderColor::all(Color::srgb(0.5, 0.5, 0.6));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
                *border_color = BorderColor::all(Color::srgb(0.4, 0.4, 0.5));
            }
        }
    }
}

pub(crate) fn handle_reward_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut reward_events: MessageWriter<RewardChosenEvent>,
    mut next_run_state: ResMut<NextState<RunState>>,
    current_choices: Res<CurrentRewardChoices>,
) {
    if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        if !current_choices.choices.is_empty() {
            reward_events.write(RewardChosenEvent { choice_index: 0 });
        }
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        if current_choices.choices.len() > 1 {
            reward_events.write(RewardChosenEvent { choice_index: 1 });
        }
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        if current_choices.choices.len() > 2 {
            reward_events.write(RewardChosenEvent { choice_index: 2 });
        }
    } else if keyboard.just_pressed(KeyCode::Escape) {
        next_run_state.set(RunState::Arena);
    }
}
