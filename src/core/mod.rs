use bevy::prelude::*;
use rand::Rng;

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
            .add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Boot), transition_to_run)
            .add_systems(OnEnter(GameState::Run), initialize_run);
    }
}

fn transition_to_run(
    mut game_state: ResMut<NextState<GameState>>,
    mut run_state: ResMut<NextState<RunState>>,
) {
    // For now, skip directly to Run state and Arena
    // In a full implementation, this would go through MainMenu
    game_state.set(GameState::Run);
    run_state.set(RunState::Arena);
}

/// Initialize a new run with a fresh seed and reset segment
fn initialize_run(mut run_config: ResMut<RunConfig>) {
    // Generate a new random seed for this run
    run_config.seed = rand::rng().random();
    run_config.segment_index = 0;

    info!(
        "Starting new run with seed: {}, segment: {}",
        run_config.seed, run_config.segment_index
    );
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
