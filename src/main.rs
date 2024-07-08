mod map_loader;

use map_loader::*;
use bevy::prelude::*;
use roxmltree::*;

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
    tutorial_maps: Vec<Handle<Map>>
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(MapHandleIds {
        tutorial_maps: vec![assets.load("res/maps/tutorial/0.tmx")]
    });
}

fn hello_assets(
    map_handles: Res<MapHandleIds>,
    map_assets: Res<Assets<Map>>,
) {
    match map_assets.get(&map_handles.tutorial_maps[0]) {
        Some(file_data) => {
            let doc = Document::parse(&file_data.0).expect("can't parse document");
            let elem = doc.descendants().find(|n| n.tag_name() == "data".into()).expect("can't find data");
            
            println!("{:?}", elem.text())
        },
        None => {}
    }
}