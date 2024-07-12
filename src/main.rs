mod map_loader;

use map_loader::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                file_path: "res".to_string(),
                ..default()
            }).set(ImagePlugin::default_nearest()),
            MapLoaderPlugin(vec![
                String::from("maps/tutorial/0.tmx"),
                String::from("maps/tutorial/1.tmx"),
                String::from("maps/tutorial/2.tmx"),
                String::from("maps/tutorial/3.tmx"),
                String::from("maps/tutorial/4.tmx")
            ]),
        ))
        .add_systems(OnEnter(MapLoadState::Done), setup_scene)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    map_server: Res<MapServer>
){
    let map = &map_server.tutorial_maps[0];
    let texture = &map.sprite_sheet.sprite;

    let half_tile_width = map.tile_width as f32 / 2.0;
    let half_map_width = map.width as f32 * half_tile_width;

    let scale: f32 = 4.0;

    commands.spawn(Camera2dBundle::default());

    for idx in 0..map.data.len() {
        if map.data[idx] == 0 { continue; }

        let x = idx % map.width;
        let y = idx / map.width;

        let x_pos = (x as f32 * map.tile_width as f32 - half_map_width + half_tile_width) * scale;
        let y_pos = (y as f32 * map.tile_width as f32 - half_map_width + half_tile_width) * scale;

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3{x:x_pos, y:y_pos, z:0.0},
                    scale: Vec3::splat(scale),
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
}