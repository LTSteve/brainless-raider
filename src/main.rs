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
            MapLoaderPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, hello_assets)
        .run();
}

#[derive(Resource)]
struct MapHandleIds {
    tutorial_maps: Vec<Handle<RawMapData>>
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(MapHandleIds {
        tutorial_maps: vec![assets.load("res/maps/tutorial/0.tmx")]
    });
}

fn hello_assets(
    map_handles: Res<MapHandleIds>,
    map_assets: Res<Assets<RawMapData>>,
) {
    match map_assets.get(&map_handles.tutorial_maps[0]) {
        Some(map_data) => {
            for y in 0..map_data.height {
                for x in 0..map_data.width {
                    print!("{:?} ", map_data.data[y * map_data.width + x]);
                }
                println!();
            }
        },
        None => {}
    }
}