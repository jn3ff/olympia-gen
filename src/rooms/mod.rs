use bevy::prelude::*;

use crate::content::Direction;

#[derive(Component, Debug)]
pub struct RoomInstance {
    pub id: String,
    pub boss_room: bool,
}

#[derive(Component, Debug)]
pub struct RoomExit {
    pub direction: Direction,
}

#[derive(Resource, Debug, Default)]
pub struct RoomGraph {
    pub current_room_id: Option<String>,
}

#[derive(Event, Debug)]
pub struct RoomClearedEvent {
    pub room_id: String,
}

#[derive(Event, Debug)]
pub struct BossDefeatedEvent {
    pub boss_id: String,
}

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomGraph>()
            .add_event::<RoomClearedEvent>()
            .add_event::<BossDefeatedEvent>()
            .add_systems(Update, room_flow_stub);
    }
}

fn room_flow_stub() {}
