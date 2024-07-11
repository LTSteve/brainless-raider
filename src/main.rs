mod map_loader;

use map_loader::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                file_path: "src".to_string(),
                ..default()
            }),
            MapLoaderPlugin(vec![
                String::from("res/maps/tutorial/0.tmx"),
                String::from("res/maps/tutorial/1.tmx"),
                String::from("res/maps/tutorial/2.tmx"),
                String::from("res/maps/tutorial/3.tmx"),
                String::from("res/maps/tutorial/4.tmx")
            ]),
        ))
        .add_systems(OnEnter(MapLoadState::Done), (hello_map))
        .run();
}

fn hello_map(
    map_server: Res<MapServer>,
    image_assets: Res<Assets<Image>>
) {
    let map = &map_server.tutorial_maps[0];

    let spritesheet = &map.sprite_sheet;

    let sprite = image_assets.get(&spritesheet.sprite).unwrap();

    let object = &map.objects[0];
    
    println!("spritesheet tile width: {:?}", spritesheet.tile_width);
    println!("sprite height: {:?}", sprite.height());
    println!("object sprite idx: {:?}", object.sprite_idx);
}