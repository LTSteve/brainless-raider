mod map_loader;
mod brmap;

use brmap::*;
use bevy::prelude::*;

const FLOOR_Z:f32 = 0.0;
const ENTITY_Z:f32 = 1.0;
const SCALE:f32 = 4.0;

const TILE_WIDTH:f32 = 16.0;
const HALF_TILE_WIDTH:f32 = 8.0;
const MAP_WIDTH:f32 = 30.0 * TILE_WIDTH;
const HALF_MAP_WIDTH:f32 = 15.0 * TILE_WIDTH;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                file_path: "res".to_string(),
                ..default()
            }).set(ImagePlugin::default_nearest()),
            BRMapPlugin(vec![
                String::from("maps/tutorial/0.tmx"),
                String::from("maps/tutorial/1.tmx"),
                String::from("maps/tutorial/2.tmx"),
                String::from("maps/tutorial/3.tmx"),
                String::from("maps/tutorial/4.tmx")
            ]),
        ))
        .add_systems(OnEnter(MapLoadState::Done), setup_scene)
        .add_systems(Update, (move_movers).run_if(in_state(MapLoadState::Done)))
        .run();
}

// Components

#[derive(Debug, Component)]
struct Mover {
    data: String
}

// Systems

fn setup_scene(
    mut commands: Commands,
    map_server: Res<MapServer>
){
    let map = &map_server.tutorial_maps[0];
    let texture = &map.sprite_sheet.sprite;

    commands.spawn(Camera2dBundle::default());

    for idx in 0..map.data.len() {
        if map.data[idx] == 0 { continue; }

        let x = idx % map.width;
        let y = idx / map.width;
        
        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3{x:coord_to_pos(x as f32), y:-coord_to_pos(y as f32), z:FLOOR_Z},
                    scale: Vec3::splat(SCALE),
                    ..default()
                },
                texture: texture.clone(),
                ..default()
            },
            TextureAtlas {
                layout: map.sprite_sheet.texture_atlas_layout.clone(),
                index: (map.data[idx] as usize) - 1
            }
        ));
    }
    
    for obj in map.objects.iter() {
        let sprite_bundle = SpriteBundle {
            transform: Transform {
                translation: Vec3{x: coord_to_pos(obj.x as f32), y: -coord_to_pos(obj.y as f32 - 1.0), z: ENTITY_Z},
                scale: Vec3::splat(SCALE),
                ..default()
            },
            texture: texture.clone(),
            ..default()
        };
        let texture_atlas = TextureAtlas {
            layout: obj.sprite_sheet.texture_atlas_layout.clone(),
            index: obj.sprite_idx as usize - 1
        };

        if obj.obj_type == "Adventurer" {
            commands.spawn((
                sprite_bundle,
                texture_atlas,
                Mover {
                    data: String::from("asdf")
                }
            ));
        }
        else {
            commands.spawn((
                sprite_bundle,
                texture_atlas
            ));
        }
    }
}

fn move_movers(
    mut movers: Query<(&mut Transform, &Mover)>,
    time: Res<Time>,
    map_server: Res<MapServer>
) {
    for (mut transform, _) in movers.iter_mut() {
        transform.translation.x += 10.0 * time.delta_seconds();
    }
}

// Helpers

fn coord_to_pos(val: f32) -> f32 {
    return (val * TILE_WIDTH - HALF_MAP_WIDTH + HALF_TILE_WIDTH) * SCALE;
}