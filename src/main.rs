mod brmap;
mod hydrate_components;
mod map_loader;

use std::{env, f32::consts::PI};

use bevy::{ecs::system::EntityCommands, prelude::*};
use brmap::*;
use hydrate_components::*;
use map_loader::ObjectPropertyValue;

const FLOOR_Z: f32 = 0.0;
const ENTITY_Z: f32 = 1.0;
const SCALE: f32 = 4.0;

const TILE_WIDTH: f32 = 16.0;
const HALF_TILE_WIDTH: f32 = 8.0;
const MAP_WIDTH_COORD: f32 = 30.0;
const HALF_MAP_WIDTH_COORD: f32 = 15.0;
const MAP_WIDTH: f32 = MAP_WIDTH_COORD * TILE_WIDTH;
const HALF_MAP_WIDTH: f32 = HALF_MAP_WIDTH_COORD * TILE_WIDTH;

const MOVER_SPEED: f32 = 1.95;

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
        ))
        .add_systems(OnEnter(MapLoadState::Done), setup_scene)
        .add_systems(Update, (move_movers).run_if(in_state(MapLoadState::Done)))
        .run();
}

// Components

#[derive(Debug, Component)]
struct Mover {
    dir: IVec2,
    target: IVec2,
    coord: IVec2,
    move_percent: f32,
    clockwise: bool,
}

fn hydrate_mover(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let x = object_data.properties.iter().find(|p| p.name == "dir_x");
    let y = object_data.properties.iter().find(|p| p.name == "dir_y");
    let clockwise = object_data
        .properties
        .iter()
        .find(|p| p.name == "clockwise");

    let x = if x.is_none() {
        0
    } else {
        if let ObjectPropertyValue::Int { value } = x.unwrap().value {
            value
        } else {
            0
        }
    };
    let y = if y.is_none() {
        0
    } else {
        if let ObjectPropertyValue::Int { value } = y.unwrap().value {
            value
        } else {
            0
        }
    };

    let clockwise = if let Some(prop) = clockwise {
        if let ObjectPropertyValue::Bool { value } = prop.value {
            value
        } else {
            false
        }
    } else {
        false
    };

    entity_commands.insert(Mover {
        dir: IVec2::new(x as i32, y as i32),
        target: IVec2::new(
            object_data.x as i32 + x as i32,
            object_data.y as i32 + y as i32,
        ),
        coord: IVec2::new(object_data.x as i32, object_data.y as i32),
        move_percent: 0.0,
        clockwise,
    });
}

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
        let y = idx / map.width;

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3 {
                        x: coord_to_pos(x as f32),
                        y: -coord_to_pos(y as f32),
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

    let entity_hydrator = &ComponentHydrators::new().register_hydrator("Mover", hydrate_mover);

    for obj in map.objects.iter() {
        let sprite_bundle = SpriteBundle {
            transform: Transform {
                translation: Vec3 {
                    x: coord_to_pos(obj.x as f32),
                    y: -coord_to_pos(obj.y as f32 - 1.0),
                    z: ENTITY_Z,
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

        match components_property {
            Some(components_property) => match &components_property.value {
                map_loader::ObjectPropertyValue::Str {
                    value: components_str,
                } => {
                    for component_name in String::from(components_str).split("|") {
                        entity_hydrator.hydrate_entity(&mut entity_commands, &obj, component_name)
                    }
                }
                _ => {
                    println!("tried to parse a non-str Components property!");
                }
            },
            None => {}
        }
    }
}

fn move_movers(
    mut movers: Query<(&mut Transform, &mut Mover)>,
    time: Res<Time>,
    map_server: Res<MapServer>,
) {
    let active_map = &map_server.tutorial_maps[0];

    for (mut transform, mut mover) in movers.iter_mut() {
        mover.move_percent = clamp(
            mover.move_percent + MOVER_SPEED * time.delta_seconds(),
            0.0,
            1.0,
        );
        let destination = Vec3::new(
            coord_to_pos(mover.target.x as f32),
            coord_to_pos(mover.target.y as f32),
            ENTITY_Z,
        );
        let previous_position = Vec3::new(
            coord_to_pos(mover.coord.x as f32),
            coord_to_pos(mover.coord.y as f32),
            ENTITY_Z,
        );
        transform.translation = cerp_v3(previous_position, destination, mover.move_percent);

        if mover.move_percent == 1.0 {
            mover.move_percent = 0.0;
            mover.coord = mover.coord + mover.dir;

            let forward = mover.dir;
            let side = rotate_dir(mover.dir, mover.clockwise);
            let back = -mover.dir;
            let target: IVec2;

            if tile_data_from_coord(mover.coord + forward, active_map) == 1 {
                target = mover.coord + forward;
            } else if tile_data_from_coord(mover.coord + side, active_map) == 1 {
                target = mover.coord + side;
                mover.dir = side;
            } else if tile_data_from_coord(mover.coord + back, active_map) == 1 {
                target = mover.coord + back;
                mover.dir = back;
            } else {
                target = mover.coord;
                mover.dir = IVec2::ZERO;
            }

            mover.target = target;
        }
    }
}

const RIGHT: IVec2 = IVec2::new(1, 0);
const LEFT: IVec2 = IVec2::new(-1, 0);
const UP: IVec2 = IVec2::new(0, 1);
const DOWN: IVec2 = IVec2::new(0, -1);

// Helpers
fn tile_data_from_coord(coord: IVec2, map_data: &MapData) -> u8 {
    let x = coord.x as usize;
    let y = MAP_WIDTH_COORD as usize - coord.y as usize - 1; // this is a little funky
    return *map_data
        .data
        .get(x + y * MAP_WIDTH_COORD as usize)
        .expect("tile data from coord oob!");
}

fn rotate_dir(dir: IVec2, cw: bool) -> IVec2 {
    if dir == RIGHT {
        return if cw { DOWN } else { UP };
    } else if dir == DOWN {
        return if cw { LEFT } else { RIGHT };
    } else if dir == LEFT {
        return if cw { UP } else { DOWN };
    } else if dir == UP {
        return if cw { RIGHT } else { LEFT };
    }
    return IVec2::ZERO;
}

fn cerp_v3(start: Vec3, end: Vec3, percent: f32) -> Vec3 {
    return lerp_v3(start, end, (percent * PI * 0.5).sin());
}

fn lerp_v3(start: Vec3, end: Vec3, percent: f32) -> Vec3 {
    return start * (1.0 - percent) + end * percent;
}

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
