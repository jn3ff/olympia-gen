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
use crate::movement::{Ground, Player, Wall};

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

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: String,
    pub name: String,
    pub exits: Vec<Direction>,
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
            boss_room: false,
            width: 800.0,
            height: 500.0,
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
            .add_message::<RoomClearedEvent>()
            .add_message::<BossDefeatedEvent>()
            .add_message::<EnterRoomEvent>()
            .add_message::<ExitRoomEvent>()
            .add_systems(Startup, setup_room_registry)
            .add_systems(OnEnter(RunState::Arena), spawn_arena_hub)
            .add_systems(OnExit(RunState::Arena), cleanup_arena)
            .add_systems(OnEnter(RunState::Room), spawn_current_room)
            .add_systems(OnExit(RunState::Room), cleanup_room)
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
                    detect_exit_collision,
                    process_room_transitions,
                    handle_boss_defeated,
                )
                    .chain()
                    .run_if(in_state(RunState::Room)),
            );
    }
}

// ============================================================================
// Setup Systems
// ============================================================================

fn setup_room_registry(mut registry: ResMut<RoomRegistry>) {
    // Register default rooms - in a full implementation these would come from RON files
    registry.rooms = vec![
        RoomData {
            id: "room_left_1".to_string(),
            name: "Western Chamber".to_string(),
            exits: vec![Direction::Right, Direction::Up],
            boss_room: false,
            width: 800.0,
            height: 500.0,
        },
        RoomData {
            id: "room_right_1".to_string(),
            name: "Eastern Hall".to_string(),
            exits: vec![Direction::Left, Direction::Down],
            boss_room: false,
            width: 900.0,
            height: 450.0,
        },
        RoomData {
            id: "room_up_1".to_string(),
            name: "Upper Sanctum".to_string(),
            exits: vec![Direction::Down, Direction::Left, Direction::Right],
            boss_room: false,
            width: 1000.0,
            height: 600.0,
        },
        RoomData {
            id: "room_down_1".to_string(),
            name: "Lower Depths".to_string(),
            exits: vec![Direction::Up],
            boss_room: false,
            width: 700.0,
            height: 400.0,
        },
        RoomData {
            id: "boss_room".to_string(),
            name: "Champion's Arena".to_string(),
            exits: vec![Direction::Down],
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
    portal_query: Query<&ArenaPortal>,
    player_query: Query<Entity, With<Player>>,
    mut room_graph: ResMut<RoomGraph>,
    registry: Res<RoomRegistry>,
    mut next_state: ResMut<NextState<RunState>>,
) {
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

        if let Ok(portal) = portal_query.get(portal_entity) {
            // Find a room for this direction
            let target_room = find_room_for_direction(&registry, portal.direction);

            if let Some(room_id) = target_room {
                room_graph.pending_transition = Some(RoomTransition {
                    from_room: None,
                    to_room: room_id,
                    entry_direction: opposite_direction(portal.direction),
                });
                next_state.set(RunState::Room);
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
    mut room_graph: ResMut<RoomGraph>,
    registry: Res<RoomRegistry>,
    mut next_state: ResMut<NextState<RunState>>,
) {
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
            room_graph.pending_transition = Some(RoomTransition {
                from_room: None,
                to_room: room_id,
                entry_direction: opposite_direction(dir),
            });
            next_state.set(RunState::Room);
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
    run_config: Res<RunConfig>,
    difficulty: Res<DifficultyScaling>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Some(transition) = &room_graph.pending_transition else {
        // No transition pending, spawn default room
        spawn_room_geometry(&mut commands, &RoomData::default(), None);
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
    spawn_room_geometry(&mut commands, &room_data, Some(transition.entry_direction));

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
) {
    let wall_color = Color::srgb(0.3, 0.3, 0.4);
    let ground_color = Color::srgb(0.4, 0.5, 0.4);
    let exit_color = Color::srgb(0.3, 0.7, 0.4);
    let boss_exit_color = Color::srgb(0.7, 0.3, 0.3);

    let half_width = room.width / 2.0;
    let half_height = room.height / 2.0;
    let wall_thickness = 40.0;

    // Room instance marker
    commands.spawn((
        RoomInstance {
            id: room.id.clone(),
            boss_room: room.boss_room,
        },
        Transform::default(),
        Visibility::default(),
    ));

    // Ground
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
    ));

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
        ));

        // Up exit trigger
        let color = if room.boss_room {
            boss_exit_color
        } else {
            exit_color
        };
        commands.spawn((
            RoomExit {
                direction: Direction::Up,
                target_room_id: None,
            },
            ExitTrigger,
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
        ));

        // Left exit trigger
        let color = if room.boss_room {
            boss_exit_color
        } else {
            exit_color
        };
        commands.spawn((
            RoomExit {
                direction: Direction::Left,
                target_room_id: None,
            },
            ExitTrigger,
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
        ));

        // Right exit trigger
        let color = if room.boss_room {
            boss_exit_color
        } else {
            exit_color
        };
        commands.spawn((
            RoomExit {
                direction: Direction::Right,
                target_room_id: None,
            },
            ExitTrigger,
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
        ));
    }

    // Down exit (in the ground)
    if room.exits.contains(&Direction::Down) {
        let gap_width = 100.0;
        let color = if room.boss_room {
            boss_exit_color
        } else {
            exit_color
        };

        commands.spawn((
            RoomExit {
                direction: Direction::Down,
                target_room_id: None,
            },
            ExitTrigger,
            Sprite {
                color,
                custom_size: Some(Vec2::new(gap_width, wall_thickness / 2.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -half_height - wall_thickness / 4.0, 0.5),
            Collider::rectangle(gap_width, wall_thickness / 2.0),
            Sensor,
            CollisionEventsEnabled,
        ));
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
        ));
    }
}

fn get_spawn_position(room: &RoomData, entry_direction: Direction) -> Vec2 {
    let half_width = room.width / 2.0;
    let half_height = room.height / 2.0;
    let offset = 60.0; // Distance from wall

    match entry_direction {
        Direction::Left => Vec2::new(-half_width + offset, -half_height + 80.0),
        Direction::Right => Vec2::new(half_width - offset, -half_height + 80.0),
        Direction::Up => Vec2::new(0.0, half_height - offset),
        Direction::Down => Vec2::new(0.0, -half_height + 80.0),
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
        )>,
    >,
    mut room_graph: ResMut<RoomGraph>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    // Clear pending transition after room is cleaned up
    if let Some(transition) = room_graph.pending_transition.take() {
        room_graph.current_room_id = Some(transition.to_room);
    }
}

fn detect_exit_collision(
    mut collision_events: MessageReader<CollisionStart>,
    exit_query: Query<&RoomExit>,
    player_query: Query<Entity, With<Player>>,
    arena_lock_query: Query<Entity, With<ArenaLock>>,
    mut exit_events: MessageWriter<ExitRoomEvent>,
) {
    // If arena is locked (boss fight in progress), don't allow exits
    if !arena_lock_query.is_empty() {
        // Consume events without processing
        for _ in collision_events.read() {}
        return;
    }

    let Some(player_entity) = player_query.iter().next() else {
        return;
    };

    for event in collision_events.read() {
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

        if let Ok(exit) = exit_query.get(exit_entity) {
            exit_events.write(ExitRoomEvent {
                direction: exit.direction,
            });
        }
    }
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
        // Find a room that has an exit in the opposite direction (so we can enter it)
        let entry_dir = opposite_direction(event.direction);

        if let Some(target_room) =
            find_room_with_entry(&registry, entry_dir, &room_graph.rooms_cleared)
        {
            room_graph.pending_transition = Some(RoomTransition {
                from_room: room_graph.current_room_id.clone(),
                to_room: target_room,
                entry_direction: entry_dir,
            });

            // Trigger room change by exiting and re-entering Room state
            // This will trigger OnExit(Room) -> cleanup -> OnEnter(Room) -> spawn
            next_state.set(RunState::Arena);
            // After a brief moment, we'll transition back to Room
            // For now, we go back to Arena and let the player choose again
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
