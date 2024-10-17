use bevy::prelude::*;

use crate::{ComponentHydrators, MapLoadState, MapServer, SceneState};

// Plugin

pub struct YouWinPlugin;
impl Plugin for YouWinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators).add_systems(
            Update,
            track_clickable_areas.run_if(in_state(MapLoadState::Done)),
        );
    }
}

// Components

#[derive(Default, Component)]
pub struct YouWin;

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators.register_tag::<YouWin>("YouWin");
}

fn track_clickable_areas(
    you_win_q: Query<Entity, With<YouWin>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut next_state: ResMut<NextState<SceneState>>,
    mut map_server: ResMut<MapServer>,
) {
    if let Ok(_) = you_win_q.get_single() {
        if buttons.just_pressed(MouseButton::Left) {
            map_server.map_idx = std::cmp::max((map_server.map_idx + 1) % map_server.maps.len(), 1);
            next_state.set(SceneState::Transitioning);
        }
    }
}
