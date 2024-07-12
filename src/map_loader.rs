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

// Map Data Final

#[derive(Debug,Resource)]
pub struct MapServer {
    pub tutorial_maps: Vec<MapData>
}

#[derive(Debug)]
pub struct MapData {
    pub width: usize,
    pub height: usize,
    pub tile_width: u16,
    pub data: Vec<u8>,
    pub objects: Vec<ObjectData>,
    pub sprite_sheet: TextureAtlasData
}

#[derive(Debug)]
pub struct ObjectData {
    pub obj_type: String,
    pub id: u16,
    pub sprite_sheet: TextureAtlasData,
    pub sprite_idx: u32,
    pub x: u16,
    pub y: u16,
    pub properties: Vec<ObjectProperty>
}

#[derive(Debug)]
pub struct TextureAtlasData {
    pub tile_width: u8,
    pub columns: u32,
    pub sprite: Handle<Image>,
    pub texture_atlas_layout: Handle<TextureAtlasLayout>
}

// Map Data Phase 2
#[derive(Asset, TypePath, Debug, Clone)]
pub struct SpritesheetData {
    pub tile_width: u8,
    pub columns: u32,
    pub sprite: Handle<Image>
}

#[derive(Asset, TypePath, Debug)]
pub struct TemplateData {
    pub sprite_sheet: Handle<SpritesheetData>,
    pub sprite_idx: u32,
    pub properties: Vec<ObjectProperty>
}

// Map Data Phase 1
#[derive(Debug)]
pub struct ObjectReference {
    pub id: u16,
    pub template: Handle<TemplateData>,
    pub x: u16,
    pub y: u16,
    pub obj_type: String,
    pub properties: Vec<ObjectProperty>
}

#[derive(Asset, TypePath, Debug)]
pub struct RawMapData {
    pub width: usize,
    pub height: usize,
    pub tile_width: u16,
    pub data: Vec<u8>,
    pub objects: Vec<ObjectReference>,
    pub sprite_sheet: Handle<SpritesheetData>
}

// Map Loader
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
        _settings: &'a MapLoadSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<RawMapData, Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let parse_result = String::from_utf8(bytes);
        if parse_result.is_err() {
            return Err(std::io::Error::new(ErrorKind::Other, parse_result.unwrap_err()));
        }

        let file_data = parse_result.unwrap();

        let doc = Document::parse(&file_data).expect("can't parse document");
        let tile_width = str::parse::<u16>(
            doc.descendants()
                .find(|n| n.tag_name() == "map".into())
                .expect("can't parse map")
                .attribute("tilewidth")
                .expect("can't parse tilewidth")).expect("can't convert tilewith into u16");

        // Snag Floor Data
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

        // Snag Objects Data
        let mut objects = Vec::<ObjectReference>::new();
        let object_group_elm = doc.descendants().find(|n| n.tag_name() == "objectgroup".into()).expect("can't find objectgroup");
        for object_elm in object_group_elm.children() {
            if !object_elm.is_element() {
                continue;
            }
            let mut properties = Vec::<ObjectProperty>::new();

            let properties_elm = object_elm.descendants().find(|n| n.tag_name() == "properties".into());
            match properties_elm {
                Some(elm) => {
                    for property_elm in elm.children() {
                        if !property_elm.is_element() { continue; }

                        properties.push(object_property_from_property_element(property_elm));
                    }
                }
                None => {}
            }

            //snag type
            let obj_type = match object_elm.attribute("type") { Some(val) => val, None => "none" };

            objects.push(ObjectReference {
                id: str::parse::<u16>(object_elm.attribute("id").expect("can't find id")).expect("can't convert id into u16"),
                template: load_context.load(local_path_to_project_path(object_elm.attribute("template").expect("can't parse template"),&load_context.asset_path().to_string())),
                x: (str::parse::<f64>(object_elm.attribute("x").expect("can't find x")).expect("can't convert x into f64") / f64::from(tile_width)) as u16,
                y: (str::parse::<f64>(object_elm.attribute("y").expect("can't find y")).expect("can't convert y into f64") / f64::from(tile_width)) as u16,
                properties,
                obj_type: String::from(obj_type)
            });
        }

        // Snag SpriteSheet Data
        let sprite_sheet_path = doc.descendants().find(|n| n.tag_name() == "tileset".into()).expect("can't find tileset").attribute("source").expect("can't find tileset source");
        
        return Ok(RawMapData{
            width: w,
            height: h,
            data,
            objects,
            tile_width,
            sprite_sheet: load_context.load(local_path_to_project_path(sprite_sheet_path, &load_context.asset_path().to_string()))
        });
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}

// Spritesheet Loader

struct SpriteSheetLoader;

#[derive(Clone, Default, Serialize, Deserialize)]
struct SpriteSheetLoadSettings {}

impl AssetLoader for SpriteSheetLoader {
    type Asset = SpritesheetData;
    type Settings = SpriteSheetLoadSettings;
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a SpriteSheetLoadSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<SpritesheetData, Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let parse_result = String::from_utf8(bytes);
        if parse_result.is_err() {
            return Err(std::io::Error::new(ErrorKind::Other, parse_result.unwrap_err()));
        }

        let file_data = parse_result.unwrap();

        let doc = Document::parse(&file_data).expect("can't parse document");
        let tileset_elm = doc.descendants().find(|n| n.tag_name() == "tileset".into()).expect("can't load tileset");

        let tile_width = str::parse::<u8>(tileset_elm.attribute("tilewidth").expect("can't find tilewidth")).expect("can't parse tilewidth");
        let columns = str::parse::<u32>(tileset_elm.attribute("columns").expect("can't find columns")).expect("can't parse columns");

        let source = tileset_elm.descendants().find(|n| n.tag_name() == "image".into()).expect("can't find image element").attribute("source").expect("can't find image source");

        return Ok(SpritesheetData {
            tile_width,
            columns,
            sprite: load_context.load(local_path_to_project_path(source, &load_context.asset_path().to_string()))
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

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a TemplateLoadSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<TemplateData, Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let parse_result = String::from_utf8(bytes);
        if parse_result.is_err() {
            return Err(std::io::Error::new(ErrorKind::Other, parse_result.unwrap_err()));
        }

        let file_data = parse_result.unwrap();

        let doc = Document::parse(&file_data).expect("can't parse document");

        // Snag SpriteSheet Data
        let sprite_sheet_path = local_path_to_project_path(doc.descendants().find(|n| n.tag_name() == "tileset".into()).expect("can't find tileset").attribute("source").expect("can't find tileset source"), &load_context.asset_path().to_string());

        // Snag Sprite Index
        let sprite_idx = str::parse::<u32>(doc.descendants().find(|n| n.tag_name() == "object".into()).expect("can't find object").attribute("gid").expect("can't find gid")).expect("can't parse gid");

        // Snag Properties

        let mut properties = Vec::<ObjectProperty>::new();

        let properties_elm = doc.descendants().find(|n| n.tag_name() == "properties".into());
        match properties_elm {
            Some(elm) => {
                for property_elm in elm.children() {
                    if !property_elm.is_element() { continue; }
                    
                    properties.push(object_property_from_property_element(property_elm));
                }
            }
            None => {}
        }

        return Ok(TemplateData {
            sprite_sheet: load_context.load(sprite_sheet_path),
            sprite_idx,
            properties
        });
    }

    fn extensions(&self) -> &[&str] {
        &["tx"]
    }
}

#[derive(Debug, Clone)]
pub struct ObjectProperty {
    pub value: ObjectPropertyValue,
    pub name: String
}
#[derive(Debug, Clone)]
pub enum ObjectPropertyValue {
    Bool {
        value: bool
    },
    Color {
        value: Color
    },
    Float {
        value: f64
    },
    File {
        value: String
    },
    Int {
        value: i64
    },
    Obj {
        value: u32
    },
    Str {
        value: String
    }
}

// Helper fns

// EX:
// path = "../../sprites.tsx"
// local_path = "res/maps/tutorial/0.tmx"
// return = "res/sprites.tsx"
fn local_path_to_project_path(path:&str, local_path:&str) -> String {
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
        }
        else {
            new_path += &(part.to_owned() + "/");
        }
    }

    for part in project_path_parts {
        new_path = part.to_owned() + "/" + &new_path;
    }

    return new_path;
}

fn object_property_from_property_element(property_elm: roxmltree::Node) -> ObjectProperty{
    return ObjectProperty {
        name: String::from(property_elm.attribute("name").expect("can't find name")),
        value: match property_elm.attribute("type").expect("can't parse property type") {
            "bool" => {
                ObjectPropertyValue::Bool{ value: str::parse::<bool>(property_elm.attribute("value").expect("can't find value")).expect("can't parse value bool") }
            },
            "color" => {
                ObjectPropertyValue::Color{ value: Color::WHITE }
            },
            "float" => {
                ObjectPropertyValue::Float{ value: str::parse::<f64>(property_elm.attribute("value").expect("can't find value")).expect("can't parse value f64") }
            },
            "file" => {
                ObjectPropertyValue::File{ value: String::from(property_elm.attribute("value").expect("can't find value")) }
            },
            "int" => {
                ObjectPropertyValue::Int{ value: str::parse::<i64>(property_elm.attribute("value").expect("can't find value")).expect("can't parse value i64") }
            },
            "object" => {
                ObjectPropertyValue::Obj{ value: str::parse::<u32>(property_elm.attribute("value").expect("can't find value")).expect("can't parse value u32") }
            },
            "string" => {
                ObjectPropertyValue::Str{ value: String::from(property_elm.attribute("value").expect("can't find value")) }
            },
            val => {
                println!("{:?}", val);
                todo!();    
            }
        }
    };    
}

// Plugin
pub struct MapLoaderPlugin(pub Vec::<String>);

impl Plugin for MapLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<RawMapData>()
            .init_asset::<SpritesheetData>()
            .init_asset::<TemplateData>()
            .register_asset_loader(MapLoader)
            .register_asset_loader(SpriteSheetLoader)
            .register_asset_loader(TemplateLoader)
            .insert_resource(MapPaths(self.0.clone()))
            .init_state::<MapLoadState>()
            .add_systems(Startup, start_loading_maps)
            .add_systems(Update, (while_loading).run_if(in_state(MapLoadState::Loading)))
            .add_systems(OnExit(MapLoadState::Loading), create_map_server);
    }
}

#[derive(Resource)]
struct MapPaths(Vec::<String>);

#[derive(Resource)]
struct MapHandleIds {
    maps: Vec<Handle<RawMapData>>
}

fn start_loading_maps(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    map_paths: Res<MapPaths>
) {
    let mut maps = Vec::<Handle<RawMapData>>::new();
    for path in map_paths.0.iter() {
        maps.push(asset_server.load(path));
    }
    commands.insert_resource(MapHandleIds{maps});
    commands.remove_resource::<MapPaths>();
}

fn create_map_server(
    mut commands: Commands,
    map_assets: Res<Assets<RawMapData>>,
    spritesheet_assets: Res<Assets<SpritesheetData>>,
    template_assets: Res<Assets<TemplateData>>,
    map_handles: Res<MapHandleIds>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>
) {
    let mut map_server = MapServer{
        tutorial_maps: Vec::<MapData>::new()
    };

    for map_handle in map_handles.maps.iter() {
        let mut objects = Vec::<ObjectData>::new();
        let asset = map_assets.get(map_handle).unwrap();

        for object_ref in &asset.objects {
            let template = template_assets.get(&object_ref.template).unwrap();
            let mut properties = template.properties.clone();

            for property in &object_ref.properties {
                let existing_property_index_option = &properties.iter().position(|prop| {
                    property.name == prop.name
                });
                if existing_property_index_option.is_none() {
                    properties.push(property.clone());
                    continue;
                }
                let existing_property_idx = existing_property_index_option.unwrap();
                properties[existing_property_idx].value = property.value.clone();
            }

            let template_sprite_sheet = spritesheet_assets.get(&template.sprite_sheet).unwrap();

            objects.push(ObjectData {
                obj_type: object_ref.obj_type.clone(),
                id: object_ref.id,
                x: object_ref.x,
                y: object_ref.y,
                sprite_idx: template.sprite_idx,
                sprite_sheet: TextureAtlasData {
                    tile_width: template_sprite_sheet.tile_width,
                    columns: template_sprite_sheet.columns,
                    sprite: template_sprite_sheet.sprite.clone(),
                    texture_atlas_layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(template_sprite_sheet.tile_width.into()), template_sprite_sheet.columns, 1, None, None))
                },
                properties
            });
        }

        let map_sprite_sheet = spritesheet_assets.get(&asset.sprite_sheet).unwrap();

        map_server.tutorial_maps.push(MapData {
            width: asset.width,
            height: asset.height,
            tile_width: asset.tile_width,
            data: asset.data.clone(),
            objects,
            sprite_sheet: TextureAtlasData {
                tile_width: map_sprite_sheet.tile_width,
                columns: map_sprite_sheet.columns,
                sprite: map_sprite_sheet.sprite.clone(),
                texture_atlas_layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(UVec2::splat(map_sprite_sheet.tile_width.into()), map_sprite_sheet.columns, 1, None, None))
            }
        });
    }

    commands.insert_resource(map_server)
}

fn while_loading(
    mut next_state: ResMut<NextState<MapLoadState>>,
    map_assets: Res<Assets<RawMapData>>,
    spritesheet_assets: Res<Assets<SpritesheetData>>,
    template_assets: Res<Assets<TemplateData>>,
    image_assets: Res<Assets<Image>>,
    map_handles: Res<MapHandleIds>
) {
    for map_handle in map_handles.maps.iter() {
        match map_assets.get(map_handle) {
            None => {
                println!("loading...");
                return;
            },
            Some(map_data) => {
                match spritesheet_assets.get(&map_data.sprite_sheet) {
                    None => {
                        println!("loading...");
                        return;
                    },
                    Some(spritesheet) => {
                        match image_assets.get(&spritesheet.sprite) {
                            None => {
                                println!("loading...");
                                return;
                            },
                            _ => {}
                        }
                    }
                }
                for o_ref in &map_data.objects {
                    match template_assets.get(&o_ref.template) {
                        None => {
                            println!("loading...");
                            return;
                        },
                        Some(template) => {
                            match spritesheet_assets.get(&template.sprite_sheet) {
                                None => {
                                    println!("loading...");
                                    return;
                                },
                                Some(spritesheet) => {
                                    match image_assets.get(&spritesheet.sprite) {
                                        None => {
                                            println!("loading...");
                                            return;
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("loaded!");
    next_state.set(MapLoadState::Done);
}

// States
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MapLoadState {
    #[default]
    Loading,
    Done
}