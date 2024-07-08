use bevy::prelude::*;
use roxmltree::*;
use serde::{Deserialize, Serialize};
use bevy::asset::{
    io::Reader,
    AssetLoader, AsyncReadExt, LoadContext,
};
use std::io::ErrorKind;
use std::io::Error;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                mode: AssetMode::Unprocessed,
                file_path: "src".to_string(),
                ..default()
            }),
        ))
        .init_asset::<Map>()
        .register_asset_loader(MapLoader)
        .add_systems(Startup, setup)
        .add_systems(Update, hello_assets)
        .run();
}

//Snagged from https://github.com/bevyengine/bevy/blob/latest/examples/asset/processing/asset_processing.rs

#[derive(Asset, TypePath, Debug)]
struct Map(String);

struct MapLoader;

#[derive(Clone, Default, Serialize, Deserialize)]
struct MapLoadSettings;

impl AssetLoader for MapLoader {
    type Asset = Map;
    type Settings = MapLoadSettings;
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _: &'a MapLoadSettings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Map, Error> {

        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        match String::from_utf8(bytes) {
            Ok(value) => {
                return Ok(Map(value));
            },
            Err(e) => {
                return Err(std::io::Error::new(ErrorKind::Other, e));
            }
        }
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
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