use bevy::prelude::*;
use roxmltree::*;
use serde::{Deserialize, Serialize};
use bevy::asset::{
    io::Reader,
    AssetLoader, AsyncReadExt, LoadContext,
};
use std::io::ErrorKind;
use std::io::Error;
use std::u8;

//Snagged partly from https://github.com/bevyengine/bevy/blob/latest/examples/asset/processing/asset_processing.rs

#[derive(Asset, TypePath, Debug)]
pub struct Map(pub String);

#[derive(Asset, TypePath, Debug)]
pub struct RawMapData {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>
}

struct MapLoader;

#[derive(Clone, Default, Serialize, Deserialize)]
struct MapLoadSettings {}

impl AssetLoader for MapLoader {
    type Asset = RawMapData;
    type Settings = MapLoadSettings;
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a MapLoadSettings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<RawMapData, Error> {

        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let parse_result = String::from_utf8(bytes);
        if parse_result.is_err() {
            return Err(std::io::Error::new(ErrorKind::Other, parse_result.unwrap_err()));
        }

        let file_data = parse_result.unwrap();

        let doc = Document::parse(&file_data).expect("can't parse document");
        let layer_elem = doc.descendants().find(|n| n.tag_name() == "layer".into()).expect("can't find layer");
        let mut floor_str = doc.descendants().find(|n| n.tag_name() == "data".into()).expect("can't find data").text().expect("couldn't unwrap data str").to_string();
        floor_str.retain(|c| return c != '\n' && !c.is_whitespace());
        let floor_data = floor_str.split(',');

        let w = layer_elem.attribute("width").expect("can't find width").parse::<usize>().expect("failed unwrapping width value");
        let h = layer_elem.attribute("height").expect("can't find height").parse::<usize>().expect("failed unwrapping height value");

        let mut data = vec![0; w * h];
        let mut idx = 0;
        for num in floor_data {
            data[idx] = num.parse::<u8>().expect("failed at parsing a number");
            idx += 1;
        }
        
        return Ok(RawMapData{
            width: w,
            height: h,
            data
        });
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}

pub struct MapLoaderPlugin;

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RawMapData>()
            .register_asset_loader(MapLoader);
    }
}
