use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum GameState {
    #[default]
    Boot,
    MainMenu,
    CharacterSelect,
    Run,
    Reward,
    Paused,
    Victory,
}

// ============================================================================
// Character Selection
// ============================================================================

/// Resource tracking the currently selected character
#[derive(Resource, Debug, Default)]
pub struct SelectedCharacter {
    pub character_id: Option<String>,
}

impl SelectedCharacter {
    pub fn select(&mut self, character_id: impl Into<String>) {
        self.character_id = Some(character_id.into());
    }

    pub fn is_selected(&self) -> bool {
        self.character_id.is_some()
    }
}

/// Event fired when a character is selected
#[derive(Debug)]
pub struct CharacterSelectedEvent {
    pub character_id: String,
}

impl Message for CharacterSelectedEvent {}

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum RunState {
    #[default]
    Arena,
    Room,
    Boss,
    Reward,
}

#[derive(Resource, Debug)]
pub struct RunConfig {
    pub seed: u64,
    pub segment_index: u32,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            seed: rand::rng().random(),
            segment_index: 0,
        }
    }
}

// ============================================================================
// Segment Progress Tracking
// ============================================================================

/// Tracks progress within the current segment and across the entire run.
#[derive(Resource, Debug, Default)]
pub struct SegmentProgress {
    /// Rooms cleared in current segment (excludes boss rooms)
    pub rooms_cleared_this_segment: u32,
    /// Bosses defeated in current segment
    pub bosses_defeated_this_segment: u32,
    /// Total bosses defeated across entire run (for win condition)
    pub total_bosses_defeated: u32,
    /// The biome_id for this segment (selected at segment start)
    pub current_biome_id: Option<String>,
    /// Pool of available room_ids for this segment (pre-selected at segment start)
    pub room_pool: Vec<String>,
    /// Pool of available boss room_ids for this segment
    pub boss_room_pool: Vec<String>,
    /// IDs of significant enemies already encountered this run (for no-repeat logic)
    pub encountered_significant_enemies: HashSet<String>,
    /// Flag indicating segment pools need initialization
    pub needs_pool_init: bool,
}

impl SegmentProgress {
    /// Reset for a new run
    pub fn reset(&mut self) {
        self.rooms_cleared_this_segment = 0;
        self.bosses_defeated_this_segment = 0;
        self.total_bosses_defeated = 0;
        self.current_biome_id = None;
        self.room_pool.clear();
        self.boss_room_pool.clear();
        self.encountered_significant_enemies.clear();
        self.needs_pool_init = true;
    }

    /// Reset segment-specific counters for next segment (preserves run-wide tracking)
    pub fn advance_segment(&mut self) {
        self.rooms_cleared_this_segment = 0;
        self.bosses_defeated_this_segment = 0;
        self.current_biome_id = None;
        self.room_pool.clear();
        self.boss_room_pool.clear();
        self.needs_pool_init = true;
        // NOTE: encountered_significant_enemies persists across segments
    }
}

/// Event fired when a segment is completed
#[derive(Debug)]
pub struct SegmentCompletedEvent {
    pub segment_index: u32,
}

impl Message for SegmentCompletedEvent {}

/// Event fired when the run is won (boss_target reached)
#[derive(Debug)]
pub struct RunVictoryEvent {
    pub total_bosses_defeated: u32,
}

impl Message for RunVictoryEvent {}

// ============================================================================
// Difficulty Scaling
// ============================================================================

/// Configuration for how difficulty scales with segment progression
#[derive(Resource, Debug, Clone)]
pub struct DifficultyScaling {
    /// Base multiplier applied to all scaling (adjust for overall difficulty)
    pub base_multiplier: f32,
    /// How much enemy health increases per segment (e.g., 0.15 = +15% per segment)
    pub enemy_health_per_segment: f32,
    /// How much enemy damage increases per segment
    pub enemy_damage_per_segment: f32,
    /// How much enemy count increases per segment (additive)
    pub enemy_count_per_segment: f32,
    /// How much boss health increases per segment
    pub boss_health_per_segment: f32,
    /// How much boss damage increases per segment
    pub boss_damage_per_segment: f32,
    /// Bonus to higher tier reward drop rates per segment
    pub reward_tier_bonus_per_segment: f32,
    /// Maximum scaling multiplier (caps the difficulty growth)
    pub max_scaling_multiplier: f32,
}

impl Default for DifficultyScaling {
    fn default() -> Self {
        Self {
            base_multiplier: 1.0,
            enemy_health_per_segment: 0.20,
            enemy_damage_per_segment: 0.15,
            enemy_count_per_segment: 0.5,
            boss_health_per_segment: 0.25,
            boss_damage_per_segment: 0.20,
            reward_tier_bonus_per_segment: 0.05,
            max_scaling_multiplier: 5.0,
        }
    }
}

impl DifficultyScaling {
    /// Calculate health multiplier for enemies at the given segment
    pub fn enemy_health_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.enemy_health_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate damage multiplier for enemies at the given segment
    pub fn enemy_damage_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.enemy_damage_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate additional enemy count for the given segment
    pub fn bonus_enemy_count(&self, segment: u32) -> usize {
        (segment as f32 * self.enemy_count_per_segment).floor() as usize
    }

    /// Calculate health multiplier for bosses at the given segment
    pub fn boss_health_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.boss_health_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate damage multiplier for bosses at the given segment
    pub fn boss_damage_multiplier(&self, segment: u32) -> f32 {
        let raw = self.base_multiplier + (segment as f32 * self.boss_damage_per_segment);
        raw.min(self.max_scaling_multiplier)
    }

    /// Calculate tier drop bonus for rewards at the given segment
    /// Returns a value to shift tier probabilities toward higher tiers
    pub fn reward_tier_bonus(&self, segment: u32) -> f32 {
        let raw = segment as f32 * self.reward_tier_bonus_per_segment;
        raw.min(0.5) // Cap at 50% bonus to preserve some randomness
    }
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<RunState>()
            .init_resource::<RunConfig>()
            .init_resource::<DifficultyScaling>()
            .init_resource::<SelectedCharacter>()
            .init_resource::<SegmentProgress>()
            .add_message::<CharacterSelectedEvent>()
            .add_message::<SegmentCompletedEvent>()
            .add_message::<RunVictoryEvent>()
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Boot), transition_to_character_select)
            .add_systems(
                OnEnter(GameState::CharacterSelect),
                spawn_character_select_ui,
            )
            .add_systems(
                OnExit(GameState::CharacterSelect),
                cleanup_character_select_ui,
            )
            .add_systems(
                Update,
                (handle_character_select_input, handle_character_select_click)
                    .run_if(in_state(GameState::CharacterSelect)),
            )
            .add_systems(OnEnter(GameState::Run), initialize_run)
            .add_systems(
                Update,
                (handle_segment_completed, handle_victory).run_if(in_state(GameState::Run)),
            )
            .add_systems(OnEnter(GameState::Victory), spawn_victory_screen)
            .add_systems(OnExit(GameState::Victory), cleanup_victory_screen)
            .add_systems(
                Update,
                handle_victory_input.run_if(in_state(GameState::Victory)),
            );
    }
}

fn transition_to_character_select(mut game_state: ResMut<NextState<GameState>>) {
    // Go to character selection before starting the run
    game_state.set(GameState::CharacterSelect);
}

/// Transition from character select to run state
fn transition_to_run(
    mut game_state: ResMut<NextState<GameState>>,
    mut run_state: ResMut<NextState<RunState>>,
) {
    game_state.set(GameState::Run);
    run_state.set(RunState::Arena);
}

/// Initialize a new run with a fresh seed and reset segment
fn initialize_run(
    mut run_config: ResMut<RunConfig>,
    mut segment_progress: ResMut<SegmentProgress>,
    mut run_faith: ResMut<crate::rewards::RunFaith>,
) {
    // Generate a new random seed for this run
    run_config.seed = rand::rng().random();
    run_config.segment_index = 0;

    // Reset segment progress for new run
    segment_progress.reset();

    // Reset faith tracking for new run
    run_faith.reset();

    info!(
        "Starting new run with seed: {}, segment: {}",
        run_config.seed, run_config.segment_index
    );
}

/// Handle segment completion - increment segment and return to hub
fn handle_segment_completed(
    mut events: MessageReader<SegmentCompletedEvent>,
    mut run_config: ResMut<RunConfig>,
    mut segment_progress: ResMut<SegmentProgress>,
    mut next_run_state: ResMut<NextState<RunState>>,
) {
    for event in events.read() {
        info!("Segment {} completed!", event.segment_index);

        // Increment segment
        run_config.segment_index += 1;

        // Reset segment-specific counters (preserves run-wide tracking)
        segment_progress.advance_segment();

        // Return to hub
        next_run_state.set(RunState::Arena);
    }
}

/// Handle victory - transition to victory state
fn handle_victory(
    mut events: MessageReader<RunVictoryEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for event in events.read() {
        info!(
            "Victory! Defeated {} bosses total.",
            event.total_bosses_defeated
        );
        game_state.set(GameState::Victory);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ============================================================================
// Character Selection UI
// ============================================================================

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

fn spawn_character_select_ui(
    mut commands: Commands,
    registry: Option<Res<crate::content::ContentRegistry>>,
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

fn cleanup_character_select_ui(
    mut commands: Commands,
    query: Query<Entity, With<CharacterSelectUI>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_character_select_input(
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

fn handle_character_select_click(
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

// ============================================================================
// Victory Screen
// ============================================================================

/// Marker for victory screen UI root
#[derive(Component, Debug)]
pub struct VictoryScreenUI;

fn spawn_victory_screen(
    mut commands: Commands,
    segment_progress: Res<SegmentProgress>,
    run_faith: Res<crate::rewards::RunFaith>,
    content_registry: Option<Res<crate::content::ContentRegistry>>,
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

fn cleanup_victory_screen(mut commands: Commands, query: Query<Entity, With<VictoryScreenUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        game_state.set(GameState::CharacterSelect);
    }
}
