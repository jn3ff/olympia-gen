use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug, Default)]
pub struct MovementState {
    pub on_ground: bool,
    pub facing: Facing,
    pub coyote_timer: f32,
    pub jump_buffer_timer: f32,
    pub dash_cooldown_timer: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct MovementTuning {
    pub max_speed: f32,
    pub accel: f32,
    pub decel: f32,
    pub jump_velocity: f32,
    pub gravity: f32,
    pub coyote_time: f32,
    pub jump_buffer_time: f32,
    pub dash_speed: f32,
    pub dash_time: f32,
    pub dash_cooldown: f32,
    pub ground_only_dash: bool,
}

impl Default for MovementTuning {
    fn default() -> Self {
        Self {
            max_speed: 320.0,
            accel: 3000.0,
            decel: 2600.0,
            jump_velocity: 680.0,
            gravity: 1800.0,
            coyote_time: 0.12,
            jump_buffer_time: 0.12,
            dash_speed: 900.0,
            dash_time: 0.16,
            dash_cooldown: 0.35,
            ground_only_dash: true,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct MovementInput {
    pub axis: Vec2,
    pub jump_pressed: bool,
    pub dash_pressed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Facing {
    #[default]
    Right,
    Left,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementTuning>()
            .init_resource::<MovementInput>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, read_move_input)
            .add_systems(Update, apply_movement);
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        MovementState::default(),
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.9, 0.9, 0.9),
                custom_size: Some(Vec2::new(24.0, 48.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
    ));
}

fn read_move_input(mut input: ResMut<MovementInput>) {
    input.axis = Vec2::ZERO;
    input.jump_pressed = false;
    input.dash_pressed = false;
}

fn apply_movement() {}
