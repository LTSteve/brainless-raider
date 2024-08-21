mod audio_server;
mod brmap;
mod collision;
mod collision_events;
mod hydrate_components;
mod map_loader;
mod movement;
mod scene;
mod treasure_train;

use audio_server::*;
use bevy::prelude::*;
use brmap::*;
use collision::*;
use collision_events::*;
use hydrate_components::*;
use movement::*;
use scene::*;
use treasure_train::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    mode: AssetMode::Unprocessed,
                    file_path: "res".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            BRMapPlugin(vec![
                String::from("maps/tutorial/0.tmx"),
                String::from("maps/tutorial/1.tmx"),
                String::from("maps/tutorial/2.tmx"),
                String::from("maps/tutorial/3.tmx"),
                String::from("maps/tutorial/4.tmx"),
            ]),
            CollisionPlugin {
                debug_collisions: false,
            },
            CollisionEventsPlugin,
            AudioServerPlugin,
            TreasureTrainPlugin,
            ScenePlugin,
        ))
        .add_systems(Update, move_movers.run_if(in_state(MapLoadState::Done)))
        .run();
}

// Components

#[derive(Default, Component)]
pub struct Goblinoid;
#[derive(Default, Component)]
pub struct Adventurer;
#[derive(Default, Component)]
pub struct Exit;

// Helpers

pub fn clamp(val: f32, min: f32, max: f32) -> f32 {
    if val > max {
        return max;
    }
    if val < min {
        return min;
    }
    return val;
}

pub fn coord_to_pos(val: f32) -> f32 {
    return (val * TILE_WIDTH - HALF_MAP_WIDTH + HALF_TILE_WIDTH) * SCALE;
}

pub fn pos_to_coord(val: f32) -> f32 {
    return ((val / SCALE) + HALF_MAP_WIDTH - HALF_TILE_WIDTH) / TILE_WIDTH;
}
