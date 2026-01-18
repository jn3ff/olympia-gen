use bevy::prelude::*;

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum GameState {
    #[default]
    Boot,
    MainMenu,
    Run,
    Reward,
    Paused,
}

#[derive(States, Debug, Hash, Eq, PartialEq, Clone, Default)]
pub enum RunState {
    #[default]
    Arena,
    Room,
    Boss,
    Reward,
}

#[derive(Resource, Debug, Default)]
pub struct RunConfig {
    pub seed: u64,
    pub segment_index: u32,
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<RunState>()
            .init_resource::<RunConfig>()
            .add_systems(Startup, setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
