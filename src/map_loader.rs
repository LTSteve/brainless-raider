use bevy::asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext};
use bevy::prelude::*;
use bevy_utils::BoxedFuture;
use roxmltree::*;
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::io::ErrorKind;
use std::u8;

pub struct MapLoaderPlugin;
impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RawMapData>()
            .init_asset::<SpritesheetData>()
            .init_asset::<TemplateData>()
            .register_asset_loader(MapLoader)
            .register_asset_loader(SpriteSheetLoader)
            .register_asset_loader(TemplateLoader);
    }
}

// Map Data
#[derive(Debug)]
pub struct ObjectReference {
    pub name: String,
    pub id: u16,
    pub template: Handle<TemplateData>,
    pub x: u16,
    pub y: u16,
    pub obj_type: String,
    pub properties: Vec<ObjectProperty>,
}

#[derive(Asset, TypePath, Debug)]
pub struct RawMapData {
    pub width: usize,
    pub height: usize,
    pub tile_width: u16,
    pub data: Vec<u8>,
    pub objects: Vec<ObjectReference>,
    pub sprite_sheet: Handle<SpritesheetData>,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct SpritesheetData {
    pub tile_width: u8,
    pub columns: u32,
    pub sprite: Handle<Image>,
}

#[derive(Asset, TypePath, Debug)]
pub struct TemplateData {
    pub sprite_sheet: Handle<SpritesheetData>,
    pub sprite_idx: u32,
    pub properties: Vec<ObjectProperty>,
}
#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub value_b: bool,
    pub value_c: Color,
    pub value_f: f64,
    pub value_s: String,
    pub value_i: i64,
    pub value_type: ObjectPropertyValueType,
    pub name: String,
}
impl Default for ObjectProperty {
    fn default() -> Self {
        Self {
            value_b: false,
            value_c: Color::WHITE,
            value_f: 0.0,
            value_s: String::new(),
            value_i: 0,
            value_type: ObjectPropertyValueType::Int,
            name: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ObjectPropertyValueType {
    Bool,
    Color,
    Float,
    File,
    Int,
    Obj,
    Str,
}

// Map Loader
struct MapLoader;
#[derive(Clone, Default, Serialize, Deserialize)]
struct MapLoadSettings {}
impl AssetLoader for MapLoader {
    type Asset = RawMapData;
    type Settings = MapLoadSettings;
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a MapLoadSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<'a, Result<RawMapData, Error>> {
        return Box::pin(async move {
            let mut bytes = Vec::new();

            reader.read_to_end(&mut bytes).await?;

            let parse_result = String::from_utf8(bytes);
            if parse_result.is_err() {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    parse_result.unwrap_err(),
                ));
            }

            let file_data = parse_result.unwrap();

            let doc = Document::parse(&file_data).expect("can't parse document");
            let tile_width = str::parse::<u16>(
                doc.descendants()
                    .find(|n| n.tag_name() == "map".into())
                    .expect("can't parse map")
                    .attribute("tilewidth")
                    .expect("can't parse tilewidth"),
            )
            .expect("can't convert tilewith into u16");

            // Snag Floor Data
            let layer_elem = doc
                .descendants()
                .find(|n| n.tag_name() == "layer".into())
                .expect("can't find layer");
            let mut floor_str = doc
                .descendants()
                .find(|n| n.tag_name() == "data".into())
                .expect("can't find data")
                .text()
                .expect("couldn't unwrap data str")
                .to_string();
            floor_str.retain(|c| return c != '\n' && !c.is_whitespace());
            let floor_data = floor_str.split(',');

            let w = layer_elem
                .attribute("width")
                .expect("can't find width")
                .parse::<usize>()
                .expect("failed unwrapping width value");
            let h = layer_elem
                .attribute("height")
                .expect("can't find height")
                .parse::<usize>()
                .expect("failed unwrapping height value");

            let mut data = vec![0; w * h];
            let mut idx = 0;
            for num in floor_data {
                data[idx] = num.parse::<u8>().expect("failed at parsing a number");
                idx += 1;
            }

            // Snag Objects Data
            let mut objects = Vec::<ObjectReference>::new();
            let object_group_elm = doc
                .descendants()
                .find(|n| n.tag_name() == "objectgroup".into())
                .expect("can't find objectgroup");
            for object_elm in object_group_elm.children() {
                if !object_elm.is_element() {
                    continue;
                }
                let mut properties = Vec::<ObjectProperty>::new();

                let properties_elm = object_elm
                    .descendants()
                    .find(|n| n.tag_name() == "properties".into());
                match properties_elm {
                    Some(elm) => {
                        for property_elm in elm.children() {
                            if !property_elm.is_element() {
                                continue;
                            }

                            properties.push(object_property_from_property_element(property_elm));
                        }
                    }
                    None => {}
                }

                //snag type
                let obj_type = match object_elm.attribute("type") {
                    Some(val) => val,
                    None => "none",
                };
                //snag name
                let obj_name = match object_elm.attribute("name") {
                    Some(val) => val,
                    None => "none",
                };

                objects.push(ObjectReference {
                    name: String::from(obj_name),
                    id: str::parse::<u16>(object_elm.attribute("id").expect("can't find id"))
                        .expect("can't convert id into u16"),
                    template: load_context.load(local_path_to_project_path(
                        object_elm
                            .attribute("template")
                            .expect("can't parse template"),
                        &load_context.asset_path().to_string(),
                    )),
                    x: (str::parse::<f64>(object_elm.attribute("x").expect("can't find x"))
                        .expect("can't convert x into f64")
                        / f64::from(tile_width))
                    .round() as u16,
                    y: floop_y(
                        str::parse::<f64>(object_elm.attribute("y").expect("can't find y"))
                            .expect("can't convert y into f64"),
                        tile_width,
                        h,
                    ),
                    properties,
                    obj_type: String::from(obj_type),
                });
            }

            // Snag SpriteSheet Data
            let sprite_sheet_path = doc
                .descendants()
                .find(|n| n.tag_name() == "tileset".into())
                .expect("can't find tileset")
                .attribute("source")
                .expect("can't find tileset source");

            return Ok(RawMapData {
                width: w,
                height: h,
                data,
                objects,
                tile_width,
                sprite_sheet: load_context.load(local_path_to_project_path(
                    sprite_sheet_path,
                    &load_context.asset_path().to_string(),
                )),
            });
        });
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}

fn floop_y(y: f64, tile_width: u16, map_height: usize) -> u16 {
    let y = (y / (tile_width as f64)).round() as u16;
    return map_height as u16 - y;
}

// Spritesheet Loader
struct SpriteSheetLoader;
#[derive(Clone, Default, Serialize, Deserialize)]
struct SpriteSheetLoadSettings {}
impl AssetLoader for SpriteSheetLoader {
    type Asset = SpritesheetData;
    type Settings = SpriteSheetLoadSettings;
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a SpriteSheetLoadSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        return Box::pin(async move {
            let mut bytes = Vec::new();

            reader.read_to_end(&mut bytes).await?;

            let parse_result = String::from_utf8(bytes);
            if parse_result.is_err() {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    parse_result.unwrap_err(),
                ));
            }

            let file_data = parse_result.unwrap();

            let doc = Document::parse(&file_data).expect("can't parse document");
            let tileset_elm = doc
                .descendants()
                .find(|n| n.tag_name() == "tileset".into())
                .expect("can't load tileset");

            let tile_width = str::parse::<u8>(
                tileset_elm
                    .attribute("tilewidth")
                    .expect("can't find tilewidth"),
            )
            .expect("can't parse tilewidth");
            let columns = str::parse::<u32>(
                tileset_elm
                    .attribute("columns")
                    .expect("can't find columns"),
            )
            .expect("can't parse columns");

            let source = tileset_elm
                .descendants()
                .find(|n| n.tag_name() == "image".into())
                .expect("can't find image element")
                .attribute("source")
                .expect("can't find image source");

            return Ok(SpritesheetData {
                tile_width,
                columns,
                sprite: load_context.load(local_path_to_project_path(
                    source,
                    &load_context.asset_path().to_string(),
                )),
            });
        });
    }

    fn extensions(&self) -> &[&str] {
        &["tsx"]
    }
}

// Template Loader
struct TemplateLoader;
#[derive(Clone, Default, Serialize, Deserialize)]
struct TemplateLoadSettings {}
impl AssetLoader for TemplateLoader {
    type Asset = TemplateData;
    type Settings = TemplateLoadSettings;
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a TemplateLoadSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        return Box::pin(async move {
            let mut bytes = Vec::new();

            reader.read_to_end(&mut bytes).await?;

            let parse_result = String::from_utf8(bytes);
            if parse_result.is_err() {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    parse_result.unwrap_err(),
                ));
            }

            let file_data = parse_result.unwrap();

            let doc = Document::parse(&file_data).expect("can't parse document");

            // Snag SpriteSheet Data
            let sprite_sheet_path = local_path_to_project_path(
                doc.descendants()
                    .find(|n| n.tag_name() == "tileset".into())
                    .expect("can't find tileset")
                    .attribute("source")
                    .expect("can't find tileset source"),
                &load_context.asset_path().to_string(),
            );

            // Snag Sprite Index
            let sprite_idx = str::parse::<u32>(
                doc.descendants()
                    .find(|n| n.tag_name() == "object".into())
                    .expect("can't find object")
                    .attribute("gid")
                    .expect("can't find gid"),
            )
            .expect("can't parse gid");

            // Snag Properties

            let mut properties = Vec::<ObjectProperty>::new();

            let properties_elm = doc
                .descendants()
                .find(|n| n.tag_name() == "properties".into());
            match properties_elm {
                Some(elm) => {
                    for property_elm in elm.children() {
                        if !property_elm.is_element() {
                            continue;
                        }

                        properties.push(object_property_from_property_element(property_elm));
                    }
                }
                None => {}
            }

            return Ok(TemplateData {
                sprite_sheet: load_context.load(sprite_sheet_path),
                sprite_idx,
                properties,
            });
        });
    }

    fn extensions(&self) -> &[&str] {
        &["tx"]
    }
}

// Helper fns

// EX:
// path = "../../sprites.tsx"
// local_path = "res/maps/tutorial/0.tmx"
// return = "res/sprites.tsx"
fn local_path_to_project_path(path: &str, local_path: &str) -> String {
    let local_path_string = String::from(local_path);
    let mut project_path_parts = Vec::<&str>::new();
    for part in local_path_string.split('/') {
        project_path_parts.push(part);
    }
    // remove the file name
    project_path_parts.pop();

    let path_string = String::from(path);
    let mut new_path = String::new();
    for part in path_string.split('/') {
        if part == ".." {
            // remove part of local path
            project_path_parts.pop();
            continue;
        }
        if part.contains(".") {
            new_path += part;
        } else {
            new_path += &(part.to_owned() + "/");
        }
    }

    for part in project_path_parts {
        new_path = part.to_owned() + "/" + &new_path;
    }

    return new_path;
}

fn object_property_from_property_element(property_elm: roxmltree::Node) -> ObjectProperty {
    return match property_elm.attribute("type") {
        Some("bool") => ObjectProperty {
            name: String::from(property_elm.attribute("name").expect("can't find name")),
            value_b: str::parse::<bool>(property_elm.attribute("value").expect("can't find value"))
                .expect("can't parse value bool"),
            value_type: ObjectPropertyValueType::Bool,
            ..Default::default()
        },
        Some("color") => ObjectProperty {
            name: String::from(property_elm.attribute("name").expect("can't find name")),
            value_c: Color::WHITE,
            value_type: ObjectPropertyValueType::Color,
            ..Default::default()
        },
        Some("float") => ObjectProperty {
            name: String::from(property_elm.attribute("name").expect("can't find name")),
            value_f: str::parse::<f64>(property_elm.attribute("value").expect("can't find value"))
                .expect("can't parse value f64"),
            value_type: ObjectPropertyValueType::Float,
            ..Default::default()
        },
        Some("file") => ObjectProperty {
            name: String::from(property_elm.attribute("name").expect("can't find name")),
            value_s: String::from(property_elm.attribute("value").expect("can't find value")),
            value_type: ObjectPropertyValueType::File,
            ..Default::default()
        },
        Some("int") => ObjectProperty {
            name: String::from(property_elm.attribute("name").expect("can't find name")),
            value_i: str::parse::<i64>(property_elm.attribute("value").expect("can't find value"))
                .expect("can't parse value i64"),
            value_type: ObjectPropertyValueType::Int,
            ..Default::default()
        },
        Some("object") => ObjectProperty {
            name: String::from(property_elm.attribute("name").expect("can't find name")),
            value_i: str::parse::<i64>(property_elm.attribute("value").expect("can't find value"))
                .expect("can't parse value u32"),
            value_type: ObjectPropertyValueType::Obj,
            ..Default::default()
        },
        None => ObjectProperty {
            name: String::from(property_elm.attribute("name").expect("can't find name")),
            value_s: String::from(property_elm.attribute("value").expect("can't find value")),
            value_type: ObjectPropertyValueType::Str,
            ..Default::default()
        },
        val => {
            println!("{:?}", val);
            todo!();
        }
    };
}
