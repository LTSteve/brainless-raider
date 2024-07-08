use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use bevy::asset::{
    io::Reader,
    AssetLoader, AsyncReadExt, LoadContext,
};
use std::io::ErrorKind;
use std::io::Error;

//Snagged partly from https://github.com/bevyengine/bevy/blob/latest/examples/asset/processing/asset_processing.rs

#[derive(Asset, TypePath, Debug)]
pub struct Map(pub String);

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

pub struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<Map>()
            .register_asset_loader(MapLoader);
    }
}
