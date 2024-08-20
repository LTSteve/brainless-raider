mod audio_server;
mod brmap;
mod collision;
mod collision_events;
mod hydrate_components;
mod map_loader;
mod movement;

use audio_server::*;
use bevy::prelude::*;
use brmap::*;
use collision::*;
use collision_events::*;
use hydrate_components::*;
use movement::*;

pub const FLOOR_Z: f32 = 0.0;
pub const ENTITY_Z_OFFSET: f32 = 10.0;
pub const SCALE: f32 = 4.0;

pub const TILE_WIDTH: f32 = 16.0;
pub const HALF_TILE_WIDTH: f32 = 8.0;
pub const MAP_WIDTH_COORD: f32 = 30.0;
pub const HALF_MAP_WIDTH_COORD: f32 = 15.0;
pub const MAP_WIDTH: f32 = MAP_WIDTH_COORD * TILE_WIDTH;
pub const HALF_MAP_WIDTH: f32 = HALF_MAP_WIDTH_COORD * TILE_WIDTH;

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
                debug_collisions: true,
            },
            CollisionEventsPlugin,
            AudioServerPlugin,
        ))
        .add_systems(OnEnter(MapLoadState::Done), setup_scene)
        .add_systems(Update, move_movers.run_if(in_state(MapLoadState::Done)))
        .run();
}

// Components

#[derive(Default, Component)]
pub struct Goblinoid;
#[derive(Default, Component)]
pub struct Adventurer;
#[derive(Default, Component)]
pub struct Treasure;

// Systems

fn setup_scene(mut commands: Commands, map_server: Res<MapServer>) {
    let map = &map_server.tutorial_maps[0];
    let texture = &map.sprite_sheet.sprite;

    commands.spawn(Camera2dBundle::default());

    for idx in 0..map.data.len() {
        if map.data[idx] == 0 {
            continue;
        }

        let x = idx % map.width;
        let y = map.width - idx / map.width - 1;

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3 {
                        x: coord_to_pos(x as f32),
                        y: coord_to_pos(y as f32),
                        z: FLOOR_Z,
                    },
                    scale: Vec3::splat(SCALE),
                    ..default()
                },
                texture: texture.clone(),
                ..default()
            },
            TextureAtlas {
                layout: map.sprite_sheet.texture_atlas_layout.clone(),
                index: (map.data[idx] as usize) - 1,
            },
        ));
    }

    let entity_hydrator = &ComponentHydrators::new()
        .register_hydrator("Mover", hydrate_mover)
        .register_hydrator("Collider", hydrate_collider)
        .register_tag::<Goblinoid>("Goblinoid")
        .register_tag::<Adventurer>("Adventurer")
        .register_tag::<Treasure>("Treasure");

    for obj in map.objects.iter() {
        let sprite_bundle = SpriteBundle {
            transform: Transform {
                translation: Vec3 {
                    x: coord_to_pos(obj.x as f32),
                    y: coord_to_pos(obj.y as f32),
                    z: obj.z + ENTITY_Z_OFFSET,
                },
                scale: Vec3::splat(SCALE),
                ..default()
            },
            texture: texture.clone(),
            ..default()
        };
        let texture_atlas = TextureAtlas {
            layout: obj.sprite_sheet.texture_atlas_layout.clone(),
            index: obj.sprite_idx as usize - 1,
        };

        let mut entity_commands = commands.spawn((sprite_bundle, texture_atlas));

        let components_property = obj.properties.iter().find(|prop| prop.name == "Components");

        if let Some(components_property) = components_property {
            for component_name in String::from(&components_property.value_s).split("|") {
                entity_hydrator.hydrate_entity(&mut entity_commands, &obj, component_name)
            }
        }
    }
}

// Helpers

fn clamp(val: f32, min: f32, max: f32) -> f32 {
    if val > max {
        return max;
    }
    if val < min {
        return min;
    }
    return val;
}

fn coord_to_pos(val: f32) -> f32 {
    return (val * TILE_WIDTH - HALF_MAP_WIDTH + HALF_TILE_WIDTH) * SCALE;
}

fn pos_to_coord(val: f32) -> f32 {
    return ((val / SCALE) + HALF_MAP_WIDTH - HALF_TILE_WIDTH) / TILE_WIDTH;
}
