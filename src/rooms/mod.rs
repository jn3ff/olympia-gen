use avian2d::prelude::*;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};
use bevy::prelude::*;
use rand::Rng;

use crate::combat::{
    ArenaLock, BossAttackSlots, BossConfig, BossDefeatedEvent as CombatBossDefeatedEvent,
    BossEncounterState, Enemy, EnemyBundle, EnemyTier, EnemyTuning, spawn_boss_scaled,
};
use crate::content::Direction;
use crate::core::{DifficultyScaling, RunConfig, RunState};
use crate::movement::{GameLayer, Ground, MovementTuning, Player, Wall};

// ============================================================================
// Components
// ============================================================================

/// Marker for the current room entity
#[derive(Component, Debug)]
pub struct RoomInstance {
    pub id: String,
    pub boss_room: bool,
}

/// An exit point in a room that leads to another room
#[derive(Component, Debug)]
pub struct RoomExit {
    pub direction: Direction,
    pub target_room_id: Option<String>,
}

/// Marker for exit collision trigger
#[derive(Component, Debug)]
pub struct ExitTrigger;

/// Marker for player spawn point
#[derive(Component, Debug)]
pub struct SpawnPoint {
    pub from_direction: Option<Direction>,
}

/// Marker for arena hub entity
#[derive(Component, Debug)]
pub struct ArenaHub;

/// Marker for directional portal in arena
#[derive(Component, Debug)]
pub struct ArenaPortal {
    pub direction: Direction,
}

/// UI marker for direction choice
#[derive(Component, Debug)]
pub struct DirectionChoiceUI;

/// Button for a specific direction
#[derive(Component, Debug)]
pub struct DirectionButton {
    pub direction: Direction,
}

/// Marker indicating a portal/exit is enabled and can be used for transitions
#[derive(Component, Debug, Default)]
pub struct PortalEnabled;

/// Marker indicating a portal/exit is disabled and cannot be used for transitions
#[derive(Component, Debug, Default)]
pub struct PortalDisabled;

/// Marker for the solid floor/platform under a portal that allows the player to stand on it
#[derive(Component, Debug)]
pub struct PortalFloor;

/// Tracks that the player is currently within a portal's interaction zone
#[derive(Component, Debug)]
pub struct PlayerInPortalZone {
    pub portal_entity: Entity,
}

/// Marker for the "Press [E] to enter" tooltip UI
#[derive(Component, Debug)]
pub struct PortalTooltipUI;

/// Condition that determines when a portal/exit becomes enabled
#[derive(Debug, Clone, PartialEq)]
pub enum PortalEnableCondition {
    /// Portal is always enabled from the start
    AlwaysEnabled,
    /// Portal enables when no enemies remain in the room
    NoEnemiesRemaining,
    /// All sub-conditions must be met
    All(Vec<PortalEnableCondition>),
    /// Any sub-condition must be met
    Any(Vec<PortalEnableCondition>),
}

impl Default for PortalEnableCondition {
    fn default() -> Self {
        Self::AlwaysEnabled
    }
}

/// Configuration for a single exit in a room
#[derive(Debug, Clone)]
pub struct RoomExitConfig {
    pub direction: Direction,
    pub condition: PortalEnableCondition,
}

impl RoomExitConfig {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            condition: PortalEnableCondition::AlwaysEnabled,
        }
    }

    pub fn with_condition(mut self, condition: PortalEnableCondition) -> Self {
        self.condition = condition;
        self
    }

    pub fn always_enabled(direction: Direction) -> Self {
        Self::new(direction)
    }

    pub fn when_cleared(direction: Direction) -> Self {
        Self {
            direction,
            condition: PortalEnableCondition::NoEnemiesRemaining,
        }
    }
}

/// Component that holds the enable condition for a portal/exit
#[derive(Component, Debug, Clone)]
pub struct PortalCondition {
    pub condition: PortalEnableCondition,
}

impl PortalCondition {
    pub fn new(condition: PortalEnableCondition) -> Self {
        Self { condition }
    }
}

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource, Debug, Default)]
pub struct RoomGraph {
    pub current_room_id: Option<String>,
    pub rooms_cleared: Vec<String>,
    pub pending_transition: Option<RoomTransition>,
}

#[derive(Debug, Clone)]
pub struct RoomTransition {
    pub from_room: Option<String>,
    pub to_room: String,
    pub entry_direction: Direction,
}

/// Available rooms loaded from definitions
#[derive(Resource, Debug, Default)]
pub struct RoomRegistry {
    pub rooms: Vec<RoomData>,
}

/// Cooldown timer to prevent rapid/double transitions between rooms
#[derive(Resource, Debug)]
pub struct TransitionCooldown {
    pub timer: Timer,
}

impl Default for TransitionCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

impl TransitionCooldown {
    pub fn reset(&mut self) {
        self.timer.reset();
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        self.timer.tick(delta);
    }

    pub fn can_transition(&self) -> bool {
        self.timer.remaining_secs() == 0.0
    }
}

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: String,
    pub name: String,
    pub exits: Vec<Direction>,
    /// Optional per-exit configuration. If provided, overrides the default condition for exits.
    /// Exits listed in `exits` but not in `exit_configs` use AlwaysEnabled by default.
    pub exit_configs: Option<Vec<RoomExitConfig>>,
    pub boss_room: bool,
    pub width: f32,
    pub height: f32,
}

impl Default for RoomData {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Room".to_string(),
            exits: vec![Direction::Left, Direction::Right],
            exit_configs: None,
            boss_room: false,
            width: 800.0,
            height: 500.0,
        }
    }
}

impl RoomData {
    /// Get the condition for a specific exit direction.
    /// Returns the configured condition if exit_configs is set, otherwise defaults based on room type.
    pub fn get_exit_condition(&self, direction: Direction) -> PortalEnableCondition {
        // Check if we have explicit exit configs
        if let Some(configs) = &self.exit_configs {
            if let Some(config) = configs.iter().find(|c| c.direction == direction) {
                return config.condition.clone();
            }
        }

        // Default behavior: boss rooms require clearing, regular rooms are always enabled
        if self.boss_room {
            PortalEnableCondition::NoEnemiesRemaining
        } else {
            PortalEnableCondition::AlwaysEnabled
        }
    }
}

// ============================================================================
// Events
// ============================================================================

#[derive(Debug)]
pub struct RoomClearedEvent {
    pub room_id: String,
}

impl Message for RoomClearedEvent {}

#[derive(Debug)]
pub struct BossDefeatedEvent {
    pub boss_id: String,
}

impl Message for BossDefeatedEvent {}

#[derive(Debug)]
pub struct EnterRoomEvent {
    pub room_id: String,
    pub entry_direction: Direction,
}

impl Message for EnterRoomEvent {}

#[derive(Debug)]
pub struct ExitRoomEvent {
    pub direction: Direction,
}

impl Message for ExitRoomEvent {}

// ============================================================================
// Plugin
// ============================================================================

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomGraph>()
            .init_resource::<RoomRegistry>()
            .init_resource::<TransitionCooldown>()
            .add_message::<RoomClearedEvent>()
            .add_message::<BossDefeatedEvent>()
            .add_message::<EnterRoomEvent>()
            .add_message::<ExitRoomEvent>()
            .add_systems(Startup, setup_room_registry)
            .add_systems(OnEnter(RunState::Arena), (reset_transition_cooldown, drain_stale_collision_events, spawn_arena_hub).chain())
            .add_systems(OnExit(RunState::Arena), cleanup_arena)
            .add_systems(OnEnter(RunState::Room), (reset_transition_cooldown, drain_stale_collision_events, spawn_current_room).chain())
            .add_systems(OnExit(RunState::Room), cleanup_room)
            .add_systems(Update, tick_transition_cooldown)
            .add_systems(
                Update,
                (
                    handle_arena_portal_interaction,
                    update_direction_choice_ui,
                    handle_direction_button_click,
                )
                    .run_if(in_state(RunState::Arena)),
            )
            .add_systems(
                Update,
                (
                    evaluate_portal_conditions,
                    track_player_portal_zone,
                    confirm_portal_entry,
                    process_room_transitions,
                    handle_boss_defeated,
                    update_portal_tooltip,
                )
                    .chain()
                    .run_if(in_state(RunState::Room)),
            );
    }
}

// ============================================================================
// Setup Systems
// ============================================================================

fn reset_transition_cooldown(mut cooldown: ResMut<TransitionCooldown>) {
    cooldown.reset();
    info!("[TRANSITION] Cooldown reset on state enter");
}

fn tick_transition_cooldown(mut cooldown: ResMut<TransitionCooldown>, time: Res<Time>) {
    cooldown.tick(time.delta());
}

fn drain_stale_collision_events(
    mut collision_start_events: MessageReader<CollisionStart>,
) {
    let count = collision_start_events.read().count();
    if count > 0 {
        info!("[TRANSITION] Drained {} stale collision events on state enter", count);
    }
}

fn setup_room_registry(mut registry: ResMut<RoomRegistry>) {
    // Register default rooms - in a full implementation these would come from RON files
    registry.rooms = vec![
        RoomData {
            id: "room_left_1".to_string(),
            name: "Western Chamber".to_string(),
            exits: vec![Direction::Right, Direction::Up],
            exit_configs: Some(vec![
                // Right exit requires clearing enemies first
                RoomExitConfig::when_cleared(Direction::Right),
                // Up exit is always enabled (escape route)
                RoomExitConfig::always_enabled(Direction::Up),
            ]),
            boss_room: false,
            width: 800.0,
            height: 500.0,
        },
        RoomData {
            id: "room_right_1".to_string(),
            name: "Eastern Hall".to_string(),
            exits: vec![Direction::Left, Direction::Down],
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::always_enabled(Direction::Down),
            ]),
            boss_room: false,
            width: 900.0,
            height: 450.0,
        },
        RoomData {
            id: "room_up_1".to_string(),
            name: "Upper Sanctum".to_string(),
            exits: vec![Direction::Down, Direction::Left, Direction::Right],
            exit_configs: Some(vec![
                RoomExitConfig::always_enabled(Direction::Down),
                RoomExitConfig::when_cleared(Direction::Left),
                RoomExitConfig::when_cleared(Direction::Right),
            ]),
            boss_room: false,
            width: 1000.0,
            height: 600.0,
        },
        RoomData {
            id: "room_down_1".to_string(),
            name: "Lower Depths".to_string(),
            exits: vec![Direction::Up],
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Up),
            ]),
            boss_room: false,
            width: 700.0,
            height: 400.0,
        },
        RoomData {
            id: "boss_room".to_string(),
            name: "Champion's Arena".to_string(),
            exits: vec![Direction::Down],
            // Boss room exit requires defeating the boss (no enemies remaining)
            exit_configs: Some(vec![
                RoomExitConfig::when_cleared(Direction::Down),
            ]),
            boss_room: true,
            width: 1200.0,
            height: 700.0,
        },
    ];
}

// ============================================================================
// Arena Hub Systems
// ============================================================================

/// Marker for arena segment info UI
#[derive(Component, Debug)]
pub struct ArenaSegmentInfo;

fn spawn_arena_hub(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    run_config: Res<RunConfig>,
    mut room_graph: ResMut<RoomGraph>,
) {
    // Reset rooms cleared for the new segment
    room_graph.rooms_cleared.clear();
    room_graph.current_room_id = None;
    room_graph.pending_transition = None;

    info!(
        "Entering Arena Hub - Segment {} (seed: {})",
        run_config.segment_index, run_config.seed
    );
    let wall_color = Color::srgb(0.25, 0.25, 0.35);
    let ground_color = Color::srgb(0.35, 0.4, 0.35);
    let portal_color = Color::srgb(0.4, 0.6, 0.9);

    let ground_layers = CollisionLayers::new(GameLayer::Ground, [GameLayer::Player, GameLayer::Enemy]);
    let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Player, GameLayer::Enemy]);

    // Spawn arena container
    commands.spawn((ArenaHub, Transform::default(), Visibility::default()));

    // Ground platform
    commands.spawn((
        Ground,
        Sprite {
            color: ground_color,
            custom_size: Some(Vec2::new(600.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -150.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(600.0, 40.0),
        ground_layers,
    ));

    // Left wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 400.0)),
            ..default()
        },
        Transform::from_xyz(-320.0, 30.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 400.0),
        wall_layers,
    ));

    // Right wall
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(40.0, 400.0)),
            ..default()
        },
        Transform::from_xyz(320.0, 30.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(40.0, 400.0),
        wall_layers,
    ));

    // Ceiling
    commands.spawn((
        Wall,
        Sprite {
            color: wall_color,
            custom_size: Some(Vec2::new(600.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 250.0, 0.0),
        RigidBody::Static,
        Collider::rectangle(600.0, 40.0),
        wall_layers,
    ));

    // Directional portals
    let portal_positions = [
        (Direction::Up, Vec2::new(0.0, 200.0)),
        (Direction::Down, Vec2::new(0.0, -100.0)),
        (Direction::Left, Vec2::new(-250.0, 0.0)),
        (Direction::Right, Vec2::new(250.0, 0.0)),
    ];

    for (direction, pos) in portal_positions {
        let size = match direction {
            Direction::Up | Direction::Down => Vec2::new(80.0, 30.0),
            Direction::Left | Direction::Right => Vec2::new(30.0, 80.0),
        };

        commands.spawn((
            ArenaPortal { direction },
            ExitTrigger,
            PortalEnabled,
            Sprite {
                color: portal_color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.5),
            Collider::rectangle(size.x, size.y),
            Sensor,
            CollisionEventsEnabled,
        ));
    }

    // Reset player position if exists
    for entity in player_query.iter() {
        commands
            .entity(entity)
            .insert(Transform::from_xyz(0.0, 0.0, 0.0));
    }

    // Spawn direction choice UI
    spawn_direction_choice_ui(&mut commands);

    // Spawn segment info UI
    spawn_segment_info_ui(&mut commands, run_config.segment_index);
}

fn spawn_direction_choice_ui(commands: &mut Commands) {
    commands
        .spawn((
            DirectionChoiceUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Direction labels
            let directions = [
                (Direction::Left, "← West [A]"),
                (Direction::Up, "↑ North [W]"),
                (Direction::Down, "↓ South [S]"),
                (Direction::Right, "→ East [D]"),
            ];

            for (direction, label) in directions {
                parent
                    .spawn((
                        DirectionButton { direction },
                        Button,
                        Node {
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.5, 0.5, 0.6)),
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    ))
                    .with_child((
                        Text::new(label),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    ));
            }
        });
}

fn spawn_segment_info_ui(commands: &mut Commands, segment_index: u32) {
    let text_color = Color::srgb(0.9, 0.9, 0.9);
    let accent_color = Color::srgb(0.8, 0.7, 0.3);

    commands
        .spawn((
            ArenaSegmentInfo,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Segment title
            parent.spawn((
                Text::new(format!("SEGMENT {}", segment_index + 1)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(accent_color),
            ));

            // Difficulty indicator
            let difficulty_text = match segment_index {
                0 => "Difficulty: Normal",
                1 => "Difficulty: Moderate",
                2 => "Difficulty: Challenging",
                3..=4 => "Difficulty: Hard",
                5..=6 => "Difficulty: Very Hard",
                _ => "Difficulty: Extreme",
            };

            parent.spawn((
                Text::new(difficulty_text),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(text_color),
                Node {
                    margin: UiRect::top(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Instructions
            parent.spawn((
                Text::new("Choose a direction to begin"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                Node {
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                },
            ));
        });
}

fn cleanup_arena(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<ArenaHub>,
            With<ArenaPortal>,
            With<Ground>,
            With<Wall>,
            With<DirectionChoiceUI>,
            With<ArenaSegmentInfo>,
        )>,
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn handle_arena_portal_interaction(
    mut collision_events: MessageReader<CollisionStart>,
    portal_query: Query<(&ArenaPortal, Option<&PortalEnabled>)>,
    player_query: Query<Entity, With<Player>>,
    cooldown: Res<TransitionCooldown>,
    mut room_graph: ResMut<RoomGraph>,
    registry: Res<RoomRegistry>,
    mut next_state: ResMut<NextState<RunState>>,
) {
    // Check cooldown before allowing any transitions
    if !cooldown.can_transition() {
        // Consume events without processing during cooldown
        for _ in collision_events.read() {}
        return;
    }

    let Some(player_entity) = player_query.iter().next() else {
        return;
    };

    for event in collision_events.read() {
        let (portal_entity, other) = if portal_query.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if portal_query.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other != player_entity {
            continue;
        }

        if let Ok((portal, portal_enabled)) = portal_query.get(portal_entity) {
            // Only react to portals marked as PortalEnabled
            if portal_enabled.is_none() {
                info!("[TRANSITION] Ignoring arena portal {:?} - not PortalEnabled", portal.direction);
                continue;
            }

            // Find a room for this direction
            let target_room = find_room_for_direction(&registry, portal.direction);

            if let Some(room_id) = target_room {
                info!("[TRANSITION] Arena portal {:?} -> room '{}'", portal.direction, room_id);
                room_graph.pending_transition = Some(RoomTransition {
                    from_room: None,
                    to_room: room_id.clone(),
                    entry_direction: opposite_direction(portal.direction),
                });
                info!("[TRANSITION] pending_transition set: {:?}", room_graph.pending_transition);
                next_state.set(RunState::Room);
                info!("[TRANSITION] RunState changed to Room");
            }
        }
    }
}

fn update_direction_choice_ui(
    mut button_query: Query<(&DirectionButton, &Interaction, &mut BackgroundColor)>,
) {
    for (_button, interaction, mut bg_color) in &mut button_query {
        *bg_color = match interaction {
            Interaction::Pressed => BackgroundColor(Color::srgb(0.3, 0.4, 0.5)),
            Interaction::Hovered => BackgroundColor(Color::srgb(0.2, 0.25, 0.35)),
            Interaction::None => BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
        };
    }
}

fn handle_direction_button_click(
    keyboard: Res<ButtonInput<KeyCode>>,
    button_query: Query<(&DirectionButton, &Interaction)>,
    cooldown: Res<TransitionCooldown>,
    mut room_graph: ResMut<RoomGraph>,
    registry: Res<RoomRegistry>,
    mut next_state: ResMut<NextState<RunState>>,
) {
    // Check cooldown before allowing any transitions
    if !cooldown.can_transition() {
        return;
    }

    // Check keyboard shortcuts
    let direction = if keyboard.just_pressed(KeyCode::KeyW)
        || keyboard.just_pressed(KeyCode::ArrowUp)
    {
        Some(Direction::Up)
    } else if keyboard.just_pressed(KeyCode::KeyS) || keyboard.just_pressed(KeyCode::ArrowDown) {
        Some(Direction::Down)
    } else if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
        Some(Direction::Left)
    } else if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
        Some(Direction::Right)
    } else {
        // Check button clicks
        button_query
            .iter()
            .find(|(_, interaction)| **interaction == Interaction::Pressed)
            .map(|(button, _)| button.direction)
    };

    if let Some(dir) = direction {
        if let Some(room_id) = find_room_for_direction(&registry, dir) {
            info!("[TRANSITION] Direction button {:?} -> room '{}'", dir, room_id);
            room_graph.pending_transition = Some(RoomTransition {
                from_room: None,
                to_room: room_id.clone(),
                entry_direction: opposite_direction(dir),
            });
            info!("[TRANSITION] pending_transition set: {:?}", room_graph.pending_transition);
            next_state.set(RunState::Room);
            info!("[TRANSITION] RunState changed to Room");
        }
    }
}

// ============================================================================
// Room Systems
// ============================================================================

fn spawn_current_room(
    mut commands: Commands,
    room_graph: Res<RoomGraph>,
    registry: Res<RoomRegistry>,
    boss_config: Res<BossConfig>,
    mut boss_state: ResMut<BossEncounterState>,
    enemy_tuning: Res<EnemyTuning>,
    movement_tuning: Res<MovementTuning>,
    run_config: Res<RunConfig>,
    difficulty: Res<DifficultyScaling>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Some(transition) = &room_graph.pending_transition else {
        // No transition pending, spawn default room
        spawn_room_geometry(&mut commands, &RoomData::default(), None, &movement_tuning);
        return;
    };

    // Find room data
    let room_data = registry
        .rooms
        .iter()
        .find(|r| r.id == transition.to_room)
        .cloned()
        .unwrap_or_default();

    // Determine if this room should have a boss
    let mut rng = rand::rng();
    let rng_roll: f32 = rng.random();
    let spawn_boss_encounter =
        room_data.boss_room || boss_state.should_spawn_boss(&boss_config, rng_roll);

    // Spawn room geometry
    spawn_room_geometry(&mut commands, &room_data, Some(transition.entry_direction), &movement_tuning);

    // Spawn enemies or boss with difficulty scaling
    if spawn_boss_encounter {
        spawn_boss_room_enemies(&mut commands, &room_data, &run_config, &difficulty);
    } else {
        spawn_room_enemies(
            &mut commands,
            &room_data,
            &enemy_tuning,
            &run_config,
            &difficulty,
            &mut rng,
        );
        boss_state.room_cleared();
    }

    // Position player at spawn point based on entry direction
    let spawn_pos = get_spawn_position(&room_data, transition.entry_direction);
    for mut transform in &mut player_query {
        transform.translation = spawn_pos.extend(0.0);
    }

    info!(
        "Spawned room '{}' at segment {} (health_mult: {:.2}x, damage_mult: {:.2}x)",
        room_data.id,
        run_config.segment_index,
        difficulty.enemy_health_multiplier(run_config.segment_index),
        difficulty.enemy_damage_multiplier(run_config.segment_index)
    );
}

fn spawn_room_enemies(
    commands: &mut Commands,
    room: &RoomData,
    tuning: &EnemyTuning,
    run_config: &RunConfig,
    difficulty: &DifficultyScaling,
    rng: &mut impl Rng,
) {
    let half_width = room.width / 2.0 - 80.0;
    let half_height = room.height / 2.0 - 80.0;

    // Determine number of enemies based on room size and segment
    let base_enemies = 2 + (room.width * room.height / 200000.0) as usize;
    let bonus_enemies = difficulty.bonus_enemy_count(run_config.segment_index);
    let num_enemies = (base_enemies + bonus_enemies).min(8); // Cap at 8 enemies

    // Calculate difficulty multipliers
    let health_mult = difficulty.enemy_health_multiplier(run_config.segment_index);
    let _damage_mult = difficulty.enemy_damage_multiplier(run_config.segment_index);

    for _i in 0..num_enemies {
        // Random position within room bounds
        let x = rng.random_range(-half_width..half_width);
        let y = rng.random_range(-half_height..half_height * 0.5); // Bias toward lower half

        // Random tier with weighted probabilities (higher segments have tougher enemies)
        let tier_bonus = (run_config.segment_index as f32 * 0.05).min(0.3);
        let tier_roll: f32 = rng.random();
        let tier = if tier_roll < 0.6 - tier_bonus {
            EnemyTier::Minor
        } else if tier_roll < 0.85 - tier_bonus * 0.5 {
            EnemyTier::Major
        } else {
            EnemyTier::Special
        };

        // Base health scaled by difficulty
        let base_health = 50.0 * health_mult;
        commands.spawn(EnemyBundle::new(tier, Vec2::new(x, y), base_health, tuning));
    }
}

fn spawn_boss_room_enemies(
    commands: &mut Commands,
    _room: &RoomData,
    run_config: &RunConfig,
    difficulty: &DifficultyScaling,
) {
    // Spawn the boss in the center of the room
    let boss_pos = Vec2::new(0.0, 0.0);

    // Calculate boss scaling based on segment
    let health_mult = difficulty.boss_health_multiplier(run_config.segment_index);
    let damage_mult = difficulty.boss_damage_multiplier(run_config.segment_index);

    spawn_boss_scaled(
        commands,
        boss_pos,
        100.0, // Base health (will be multiplied by tier AND difficulty)
        BossAttackSlots::default(),
        health_mult,
        damage_mult,
    );

    info!(
        "Spawned boss at segment {} (health_mult: {:.2}x, damage_mult: {:.2}x)",
        run_config.segment_index, health_mult, damage_mult
    );

    // Lock the arena exits during boss fight
    commands.spawn((ArenaLock, Transform::default(), Visibility::default()));
}

fn spawn_room_geometry(
    commands: &mut Commands,
    room: &RoomData,
    _entry_direction: Option<Direction>,
    movement_tuning: &MovementTuning,
) {
    let wall_color = Color::srgb(0.3, 0.3, 0.4);
    let ground_color = Color::srgb(0.4, 0.5, 0.4);
    let exit_enabled_color = Color::srgb(0.3, 0.7, 0.4);
    let exit_disabled_color = Color::srgb(0.5, 0.5, 0.5);
    let boss_exit_enabled_color = Color::srgb(0.7, 0.3, 0.3);
    let boss_exit_disabled_color = Color::srgb(0.4, 0.3, 0.3);

    let half_width = room.width / 2.0;
    let half_height = room.height / 2.0;
    let wall_thickness = 40.0;

    // Calculate safe reachable height for platform placement
    let safe_jump_height = movement_tuning.safe_reachable_height();

    let ground_layers = CollisionLayers::new(GameLayer::Ground, [GameLayer::Player, GameLayer::Enemy]);
    let wall_layers = CollisionLayers::new(GameLayer::Wall, [GameLayer::Player, GameLayer::Enemy]);

    // Room instance marker
    commands.spawn((
        RoomInstance {
            id: room.id.clone(),
            boss_room: room.boss_room,
        },
        Transform::default(),
        Visibility::default(),
    ));

    // Ground (with gap if Down exit exists)
    if room.exits.contains(&Direction::Down) {
        // Split ground with exit gap
        let gap_width = 100.0;
        let side_width = (room.width - gap_width) / 2.0;

        // Left ground
        commands.spawn((
            Ground,
            Sprite {
                color: ground_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(-half_width + side_width / 2.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            ground_layers,
        ));

        // Right ground
        commands.spawn((
            Ground,
            Sprite {
                color: ground_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(half_width - side_width / 2.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            ground_layers,
        ));
    } else {
        // Full ground
        commands.spawn((
            Ground,
            Sprite {
                color: ground_color,
                custom_size: Some(Vec2::new(room.width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(room.width, wall_thickness),
            ground_layers,
        ));
    }

    // Ceiling (with gap if Up exit exists)
    if room.exits.contains(&Direction::Up) {
        // Split ceiling with exit gap
        let gap_width = 100.0;
        let side_width = (room.width - gap_width) / 2.0;

        // Left ceiling
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(-half_width + side_width / 2.0, half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            wall_layers,
        ));

        // Right ceiling
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(side_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(half_width - side_width / 2.0, half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(side_width, wall_thickness),
            wall_layers,
        ));

        // Up exit trigger
        let condition = room.get_exit_condition(Direction::Up);
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if room.boss_room {
            if is_enabled { boss_exit_enabled_color } else { boss_exit_disabled_color }
        } else {
            if is_enabled { exit_enabled_color } else { exit_disabled_color }
        };

        // Solid floor platform below the Up exit so player can stand on it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        let up_exit_platform_y = half_height - wall_thickness - platform_height / 2.0;

        // Calculate ground level and check if we need stepping stones
        let ground_level = -half_height + wall_thickness / 2.0;
        let height_to_climb = up_exit_platform_y - ground_level;

        // Add stepping stone platforms if the exit is too high to reach
        if height_to_climb > safe_jump_height {
            let step_platform_color = Color::srgb(0.4, 0.35, 0.3);
            let num_steps = (height_to_climb / safe_jump_height).ceil() as i32;
            let step_height = height_to_climb / num_steps as f32;

            // Alternate platforms left and right for interesting traversal
            for i in 1..num_steps {
                let step_y = ground_level + step_height * i as f32;
                let step_x = if i % 2 == 1 { -80.0 } else { 80.0 };

                commands.spawn((
                    Ground,
                    Sprite {
                        color: step_platform_color,
                        custom_size: Some(Vec2::new(100.0, platform_height)),
                        ..default()
                    },
                    Transform::from_xyz(step_x, step_y, 0.0),
                    RigidBody::Static,
                    Collider::rectangle(100.0, platform_height),
                    ground_layers,
                ));
            }
        }

        // Main Up exit platform
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(gap_width + 40.0, platform_height)),
                ..default()
            },
            Transform::from_xyz(0.0, up_exit_platform_y, 0.0),
            RigidBody::Static,
            Collider::rectangle(gap_width + 40.0, platform_height),
            ground_layers,
        ));

        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Up,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(gap_width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, half_height, 0.5),
            Collider::rectangle(gap_width, wall_thickness),
            Sensor,
            CollisionEventsEnabled,
        ));
        if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }
    } else {
        // Full ceiling
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(room.width, wall_thickness)),
                ..default()
            },
            Transform::from_xyz(0.0, half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(room.width, wall_thickness),
            wall_layers,
        ));
    }

    // Left wall (with gap if Left exit exists)
    if room.exits.contains(&Direction::Left) {
        let gap_height = 100.0;
        let side_height = (room.height - gap_height) / 2.0;

        // Top left wall
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(-half_width, half_height - side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Bottom left wall
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(-half_width, -half_height + side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Left exit trigger
        let condition = room.get_exit_condition(Direction::Left);
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if room.boss_room {
            if is_enabled { boss_exit_enabled_color } else { boss_exit_disabled_color }
        } else {
            if is_enabled { exit_enabled_color } else { exit_disabled_color }
        };

        // Solid floor platform at the bottom of the Left exit so player can stand on it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        let platform_width = wall_thickness + 60.0; // Extends into the room
        let left_exit_platform_y = -gap_height / 2.0 - platform_height / 2.0;

        // Calculate ground level and check if we need stepping stones
        let ground_level = -half_height + wall_thickness / 2.0;
        let height_to_climb = left_exit_platform_y - ground_level;

        // Add stepping stone platforms if the exit is too high to reach
        if height_to_climb > safe_jump_height {
            let step_platform_color = Color::srgb(0.4, 0.35, 0.3);
            let num_steps = (height_to_climb / safe_jump_height).ceil() as i32;
            let step_height = height_to_climb / num_steps as f32;

            // Stagger platforms leading to the left exit
            for i in 1..num_steps {
                let step_y = ground_level + step_height * i as f32;
                // Place steps progressively closer to the left wall
                let step_x = -half_width + 100.0 + (i as f32 * 30.0);

                commands.spawn((
                    Ground,
                    Sprite {
                        color: step_platform_color,
                        custom_size: Some(Vec2::new(100.0, platform_height)),
                        ..default()
                    },
                    Transform::from_xyz(step_x, step_y, 0.0),
                    RigidBody::Static,
                    Collider::rectangle(100.0, platform_height),
                    ground_layers,
                ));
            }
        }

        // Main Left exit platform
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(platform_width, platform_height)),
                ..default()
            },
            // Position at bottom of the gap, extending into the room
            Transform::from_xyz(-half_width + platform_width / 2.0 - wall_thickness / 2.0, left_exit_platform_y, 0.0),
            RigidBody::Static,
            Collider::rectangle(platform_width, platform_height),
            ground_layers,
        ));

        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Left,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(wall_thickness, gap_height)),
                ..default()
            },
            Transform::from_xyz(-half_width, 0.0, 0.5),
            Collider::rectangle(wall_thickness, gap_height),
            Sensor,
            CollisionEventsEnabled,
        ));
        if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }
    } else {
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, room.height)),
                ..default()
            },
            Transform::from_xyz(-half_width, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, room.height),
            wall_layers,
        ));
    }

    // Right wall (with gap if Right exit exists)
    if room.exits.contains(&Direction::Right) {
        let gap_height = 100.0;
        let side_height = (room.height - gap_height) / 2.0;

        // Top right wall
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(half_width, half_height - side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Bottom right wall
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, side_height)),
                ..default()
            },
            Transform::from_xyz(half_width, -half_height + side_height / 2.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, side_height),
            wall_layers,
        ));

        // Right exit trigger
        let condition = room.get_exit_condition(Direction::Right);
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if room.boss_room {
            if is_enabled { boss_exit_enabled_color } else { boss_exit_disabled_color }
        } else {
            if is_enabled { exit_enabled_color } else { exit_disabled_color }
        };

        // Solid floor platform at the bottom of the Right exit so player can stand on it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        let platform_width = wall_thickness + 60.0; // Extends into the room
        let right_exit_platform_y = -gap_height / 2.0 - platform_height / 2.0;

        // Calculate ground level and check if we need stepping stones
        let ground_level = -half_height + wall_thickness / 2.0;
        let height_to_climb = right_exit_platform_y - ground_level;

        // Add stepping stone platforms if the exit is too high to reach
        if height_to_climb > safe_jump_height {
            let step_platform_color = Color::srgb(0.4, 0.35, 0.3);
            let num_steps = (height_to_climb / safe_jump_height).ceil() as i32;
            let step_height = height_to_climb / num_steps as f32;

            // Stagger platforms leading to the right exit
            for i in 1..num_steps {
                let step_y = ground_level + step_height * i as f32;
                // Place steps progressively closer to the right wall
                let step_x = half_width - 100.0 - (i as f32 * 30.0);

                commands.spawn((
                    Ground,
                    Sprite {
                        color: step_platform_color,
                        custom_size: Some(Vec2::new(100.0, platform_height)),
                        ..default()
                    },
                    Transform::from_xyz(step_x, step_y, 0.0),
                    RigidBody::Static,
                    Collider::rectangle(100.0, platform_height),
                    ground_layers,
                ));
            }
        }

        // Main Right exit platform
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(platform_width, platform_height)),
                ..default()
            },
            // Position at bottom of the gap, extending into the room
            Transform::from_xyz(half_width - platform_width / 2.0 + wall_thickness / 2.0, right_exit_platform_y, 0.0),
            RigidBody::Static,
            Collider::rectangle(platform_width, platform_height),
            ground_layers,
        ));

        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Right,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(wall_thickness, gap_height)),
                ..default()
            },
            Transform::from_xyz(half_width, 0.0, 0.5),
            Collider::rectangle(wall_thickness, gap_height),
            Sensor,
            CollisionEventsEnabled,
        ));
        if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }
    } else {
        commands.spawn((
            Wall,
            Sprite {
                color: wall_color,
                custom_size: Some(Vec2::new(wall_thickness, room.height)),
                ..default()
            },
            Transform::from_xyz(half_width, 0.0, 0.0),
            RigidBody::Static,
            Collider::rectangle(wall_thickness, room.height),
            wall_layers,
        ));
    }

    // Down exit (in the ground gap)
    if room.exits.contains(&Direction::Down) {
        let gap_width = 100.0;
        let condition = room.get_exit_condition(Direction::Down);
        let is_enabled = condition == PortalEnableCondition::AlwaysEnabled;
        let color = if room.boss_room {
            if is_enabled { boss_exit_enabled_color } else { boss_exit_disabled_color }
        } else {
            if is_enabled { exit_enabled_color } else { exit_disabled_color }
        };

        // Add a visible floor/bridge over the Down exit that player can stand on
        // This bridge is slightly below ground level so player can walk onto it
        let portal_floor_color = Color::srgb(0.45, 0.4, 0.35);
        let platform_height = 20.0;
        commands.spawn((
            PortalFloor,
            Ground,
            Sprite {
                color: portal_floor_color,
                custom_size: Some(Vec2::new(gap_width, platform_height)),
                ..default()
            },
            // Position at ground level within the gap
            Transform::from_xyz(0.0, -half_height, 0.0),
            RigidBody::Static,
            Collider::rectangle(gap_width, platform_height),
            ground_layers,
        ));

        // The Down exit sensor is below the bridge platform
        // When player presses E, they'll descend through
        let mut exit_cmd = commands.spawn((
            RoomExit {
                direction: Direction::Down,
                target_room_id: None,
            },
            ExitTrigger,
            PortalCondition::new(condition),
            Sprite {
                color,
                custom_size: Some(Vec2::new(gap_width, wall_thickness / 2.0)),
                ..default()
            },
            // Position sensor at ground level so player's feet touch it while standing on the bridge
            Transform::from_xyz(0.0, -half_height + platform_height / 2.0 + 10.0, 0.5),
            Collider::rectangle(gap_width, wall_thickness),
            Sensor,
            CollisionEventsEnabled,
        ));
        if is_enabled {
            exit_cmd.insert(PortalEnabled);
        } else {
            exit_cmd.insert(PortalDisabled);
        }
    }

    // Add some platforms for gameplay variety
    let platform_color = Color::srgb(0.5, 0.4, 0.3);

    // Platform positions based on room size
    let platforms = [
        Vec2::new(-room.width * 0.25, -room.height * 0.15),
        Vec2::new(room.width * 0.25, room.height * 0.1),
        Vec2::new(0.0, room.height * 0.25),
    ];

    for pos in platforms {
        commands.spawn((
            Ground,
            Sprite {
                color: platform_color,
                custom_size: Some(Vec2::new(120.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, 0.0),
            RigidBody::Static,
            Collider::rectangle(120.0, 20.0),
            ground_layers,
        ));
    }
}

fn get_spawn_position(room: &RoomData, entry_direction: Direction) -> Vec2 {
    let half_width = room.width / 2.0;
    let half_height = room.height / 2.0;
    // Distance from wall/exit sensor - must be larger than wall_thickness (40) + sensor half-height
    // to avoid spawning inside exit sensors and triggering immediate transitions
    let wall_offset = 80.0;
    let ground_offset = 80.0; // Height above ground level

    match entry_direction {
        // Spawn near left wall, on ground level (away from Left exit which is at y=0)
        Direction::Left => Vec2::new(-half_width + wall_offset, -half_height + ground_offset),
        // Spawn near right wall, on ground level (away from Right exit which is at y=0)
        Direction::Right => Vec2::new(half_width - wall_offset, -half_height + ground_offset),
        // Spawn below ceiling, far enough from Up exit sensor
        Direction::Up => Vec2::new(0.0, half_height - wall_offset),
        // Spawn above ground, Down exit is below ground so no overlap
        Direction::Down => Vec2::new(0.0, -half_height + ground_offset),
    }
}

fn cleanup_room(
    mut commands: Commands,
    query: Query<
        Entity,
        Or<(
            With<RoomInstance>,
            With<RoomExit>,
            With<Ground>,
            With<Wall>,
            With<Enemy>,
            With<ArenaLock>,
            With<PortalFloor>,
            With<PortalTooltipUI>,
        )>,
    >,
    mut player_query: Query<Entity, With<Player>>,
    mut room_graph: ResMut<RoomGraph>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Remove PlayerInPortalZone from player when leaving room
    for player_entity in &mut player_query {
        commands.entity(player_entity).remove::<PlayerInPortalZone>();
    }

    // Clear pending transition after room is cleaned up
    if let Some(transition) = room_graph.pending_transition.take() {
        room_graph.current_room_id = Some(transition.to_room);
    }
}

/// Tracks when the player enters/exits portal interaction zones.
/// Adds PlayerInPortalZone to player when touching an enabled portal sensor.
fn track_player_portal_zone(
    mut commands: Commands,
    mut collision_start_events: MessageReader<CollisionStart>,
    mut collision_end_events: MessageReader<CollisionEnd>,
    exit_query: Query<(Entity, &RoomExit, Option<&PortalEnabled>)>,
    player_query: Query<Entity, With<Player>>,
    player_zone_query: Query<&PlayerInPortalZone, With<Player>>,
    arena_lock_query: Query<Entity, With<ArenaLock>>,
) {
    let Some(player_entity) = player_query.iter().next() else {
        // Consume events if no player
        for _ in collision_start_events.read() {}
        for _ in collision_end_events.read() {}
        return;
    };

    // If arena is locked (boss fight), don't track portal zones
    if !arena_lock_query.is_empty() {
        for _ in collision_start_events.read() {}
        for _ in collision_end_events.read() {}
        return;
    }

    // Handle collision starts - player enters portal zone
    for event in collision_start_events.read() {
        let (exit_entity, other) = if exit_query.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if exit_query.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other != player_entity {
            continue;
        }

        if let Ok((portal_entity, exit, portal_enabled)) = exit_query.get(exit_entity) {
            // Only track enabled portals
            if portal_enabled.is_some() {
                info!("[PORTAL] Player entered portal zone {:?}", exit.direction);
                commands.entity(player_entity).insert(PlayerInPortalZone {
                    portal_entity,
                });
            }
        }
    }

    // Handle collision ends - player exits portal zone
    for event in collision_end_events.read() {
        let (exit_entity, other) = if exit_query.get(event.collider1).is_ok() {
            (event.collider1, event.collider2)
        } else if exit_query.get(event.collider2).is_ok() {
            (event.collider2, event.collider1)
        } else {
            continue;
        };

        if other != player_entity {
            continue;
        }

        // Check if player is leaving the portal they're currently in
        if let Ok(player_zone) = player_zone_query.get(player_entity) {
            if player_zone.portal_entity == exit_entity {
                info!("[PORTAL] Player exited portal zone");
                commands.entity(player_entity).remove::<PlayerInPortalZone>();
            }
        }
    }
}

/// Confirms portal entry when player presses E while in a portal zone.
fn confirm_portal_entry(
    keyboard: Res<ButtonInput<KeyCode>>,
    player_query: Query<&PlayerInPortalZone, With<Player>>,
    exit_query: Query<(&RoomExit, Option<&PortalEnabled>)>,
    cooldown: Res<TransitionCooldown>,
    mut exit_events: MessageWriter<ExitRoomEvent>,
) {
    // Check if player pressed E
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    // Check cooldown
    if !cooldown.can_transition() {
        return;
    }

    // Check if player is in a portal zone
    let Ok(player_zone) = player_query.single() else {
        return;
    };

    // Verify the portal is still enabled and get its direction
    if let Ok((exit, portal_enabled)) = exit_query.get(player_zone.portal_entity) {
        if portal_enabled.is_some() {
            info!("[TRANSITION] Player confirmed exit {:?} with E key", exit.direction);
            exit_events.write(ExitRoomEvent {
                direction: exit.direction,
            });
        }
    }
}

/// Updates the portal tooltip UI - shows "Press [E] to enter" when player is in portal zone.
fn update_portal_tooltip(
    mut commands: Commands,
    player_query: Query<Option<&PlayerInPortalZone>, With<Player>>,
    exit_query: Query<(&RoomExit, Option<&PortalEnabled>)>,
    existing_tooltip: Query<Entity, With<PortalTooltipUI>>,
) {
    // Check if player is in a portal zone
    let Ok(maybe_zone) = player_query.single() else {
        // No player, cleanup any tooltip
        for entity in &existing_tooltip {
            commands.entity(entity).despawn();
        }
        return;
    };

    match maybe_zone {
        Some(player_zone) => {
            // Player is in a portal zone - check if portal is enabled
            let portal_enabled = exit_query
                .get(player_zone.portal_entity)
                .map(|(_, enabled)| enabled.is_some())
                .unwrap_or(false);

            if portal_enabled {
                // Show tooltip if not already shown
                if existing_tooltip.is_empty() {
                    spawn_portal_tooltip(&mut commands);
                }
            } else {
                // Portal not enabled, hide tooltip
                for entity in &existing_tooltip {
                    commands.entity(entity).despawn();
                }
            }
        }
        None => {
            // Player not in portal zone, hide tooltip
            for entity in &existing_tooltip {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn spawn_portal_tooltip(commands: &mut Commands) {
    commands
        .spawn((
            PortalTooltipUI,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(120.0),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
                    BorderColor::all(Color::srgb(0.3, 0.6, 0.4)),
                ))
                .with_child((
                    Text::new("Press [E] to enter"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.3, 0.9, 0.4)),
                ));
        });
}

fn handle_boss_defeated(
    mut commands: Commands,
    mut boss_defeated_events: MessageReader<CombatBossDefeatedEvent>,
    arena_lock_query: Query<Entity, With<ArenaLock>>,
) {
    for _event in boss_defeated_events.read() {
        // Unlock the arena when boss is defeated
        for lock_entity in arena_lock_query.iter() {
            commands.entity(lock_entity).despawn();
        }
    }
}

fn process_room_transitions(
    mut exit_events: MessageReader<ExitRoomEvent>,
    mut room_graph: ResMut<RoomGraph>,
    registry: Res<RoomRegistry>,
    mut next_state: ResMut<NextState<RunState>>,
) {
    for event in exit_events.read() {
        info!("[TRANSITION] Processing exit event: {:?}", event.direction);

        // Find a room that has an exit in the opposite direction (so we can enter it)
        let entry_dir = opposite_direction(event.direction);

        if let Some(target_room) =
            find_room_with_entry(&registry, entry_dir, &room_graph.rooms_cleared)
        {
            info!("[TRANSITION] Found target room '{}' with entry direction {:?}", target_room, entry_dir);
            room_graph.pending_transition = Some(RoomTransition {
                from_room: room_graph.current_room_id.clone(),
                to_room: target_room.clone(),
                entry_direction: entry_dir,
            });
            info!("[TRANSITION] pending_transition set: {:?}", room_graph.pending_transition);

            // Trigger room change by exiting and re-entering Room state
            // This will trigger OnExit(Room) -> cleanup -> OnEnter(Room) -> spawn
            next_state.set(RunState::Arena);
            info!("[TRANSITION] RunState changed to Arena");
            // After a brief moment, we'll transition back to Room
            // For now, we go back to Arena and let the player choose again
        } else {
            info!("[TRANSITION] No target room found for entry direction {:?}", entry_dir);
        }
    }
}

// ============================================================================
// Portal Condition Evaluation
// ============================================================================

/// Evaluates portal conditions and enables/disables portals accordingly.
/// This system runs every frame to check if conditions have been met.
fn evaluate_portal_conditions(
    mut commands: Commands,
    enemy_query: Query<Entity, With<Enemy>>,
    room_instance_query: Query<&RoomInstance>,
    mut portal_query: Query<
        (Entity, &PortalCondition, &RoomExit, &mut Sprite, Option<&PortalDisabled>),
        Without<PortalEnabled>,
    >,
) {
    // Only evaluate if we're in a room (have a room instance)
    let Some(room_instance) = room_instance_query.iter().next() else {
        return;
    };

    // Count enemies in the room
    let enemy_count = enemy_query.iter().count();

    // Define colors for portals (should match spawn_room_geometry)
    let exit_enabled_color = Color::srgb(0.3, 0.7, 0.4);
    let boss_exit_enabled_color = Color::srgb(0.7, 0.3, 0.3);

    for (entity, condition, exit, mut sprite, is_disabled) in portal_query.iter_mut() {
        // Skip if already enabled
        if is_disabled.is_none() {
            continue;
        }

        let should_enable = evaluate_condition(&condition.condition, enemy_count);

        if should_enable {
            // Enable the portal
            commands.entity(entity).remove::<PortalDisabled>();
            commands.entity(entity).insert(PortalEnabled);

            // Update visual
            sprite.color = if room_instance.boss_room {
                boss_exit_enabled_color
            } else {
                exit_enabled_color
            };

            info!(
                "[PORTAL] Enabled exit {:?} (condition: {:?}, enemies remaining: {})",
                exit.direction, condition.condition, enemy_count
            );
        }
    }
}

/// Recursively evaluates a portal enable condition.
fn evaluate_condition(condition: &PortalEnableCondition, enemy_count: usize) -> bool {
    match condition {
        PortalEnableCondition::AlwaysEnabled => true,
        PortalEnableCondition::NoEnemiesRemaining => enemy_count == 0,
        PortalEnableCondition::All(conditions) => {
            conditions.iter().all(|c| evaluate_condition(c, enemy_count))
        }
        PortalEnableCondition::Any(conditions) => {
            conditions.iter().any(|c| evaluate_condition(c, enemy_count))
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn opposite_direction(dir: Direction) -> Direction {
    match dir {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

fn find_room_for_direction(registry: &RoomRegistry, direction: Direction) -> Option<String> {
    // Simple mapping: direction determines which room we go to
    let target_id = match direction {
        Direction::Left => "room_left_1",
        Direction::Right => "room_right_1",
        Direction::Up => "room_up_1",
        Direction::Down => "room_down_1",
    };

    registry
        .rooms
        .iter()
        .find(|r| r.id == target_id)
        .map(|r| r.id.clone())
}

fn find_room_with_entry(
    registry: &RoomRegistry,
    entry_direction: Direction,
    cleared_rooms: &[String],
) -> Option<String> {
    // Find a room that has an exit matching the entry direction
    // (meaning we can enter from that side)
    registry
        .rooms
        .iter()
        .filter(|r| !cleared_rooms.contains(&r.id))
        .find(|r| r.exits.contains(&entry_direction))
        .map(|r| r.id.clone())
}
