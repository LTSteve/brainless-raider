use crate::{map_loader::*, MAP_WIDTH_COORD};
use bevy::asset::Handle;
use bevy::prelude::*;

pub struct BRMapPlugin(pub Vec<String>);
impl Plugin for BRMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MapLoaderPlugin)
            .insert_resource(MapPaths(self.0.clone()))
            .init_state::<MapLoadState>()
            .add_systems(Startup, start_loading_maps)
            .add_systems(OnExit(MapLoadState::Loading), create_map_server)
            .add_systems(
                Update,
                (while_loading).run_if(in_state(MapLoadState::Loading)),
            );
    }
}

// States
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum MapLoadState {
    #[default]
    Loading,
    Done,
}

// Resources
#[derive(Debug, Resource)]
struct MapPaths(Vec<String>);

#[derive(Debug, Resource)]
struct MapHandleIds {
    maps: Vec<Handle<RawMapData>>,
}

#[derive(Debug, Resource)]
pub struct MapServer {
    pub tutorial_maps: Vec<MapData>,
}

// Data
#[derive(Debug)]
pub struct MapData {
    pub width: usize,
    pub height: usize,
    pub tile_width: u16,
    pub data: Vec<u8>,
    pub objects: Vec<ObjectData>,
    pub sprite_sheet: TextureAtlasData,
}

#[derive(Debug)]
pub struct ObjectData {
    pub obj_type: String,
    pub id: u16,
    pub sprite_sheet: TextureAtlasData,
    pub sprite_idx: u32,
    pub x: u16,
    pub y: u16,
    pub z: f32,
    pub properties: Vec<ObjectProperty>,
}

#[derive(Debug)]
pub struct TextureAtlasData {
    pub tile_width: u8,
    pub columns: u32,
    pub sprite: Handle<Image>,
    pub texture_atlas_layout: Handle<TextureAtlasLayout>,
}

// Systems
fn start_loading_maps(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map_paths: Res<MapPaths>,
) {
    let mut maps = Vec::<Handle<RawMapData>>::new();
    for path in map_paths.0.iter() {
        maps.push(asset_server.load(path));
    }
    commands.insert_resource(MapHandleIds { maps });
    commands.remove_resource::<MapPaths>();
}

fn create_map_server(
    mut commands: Commands,
    map_assets: Res<Assets<RawMapData>>,
    spritesheet_assets: Res<Assets<SpritesheetData>>,
    template_assets: Res<Assets<TemplateData>>,
    map_handles: Res<MapHandleIds>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut map_server = MapServer {
        tutorial_maps: Vec::<MapData>::new(),
    };

    for map_handle in map_handles.maps.iter() {
        let mut objects = Vec::<ObjectData>::new();
        let asset = map_assets.get(map_handle).unwrap();

        for object_ref in &asset.objects {
            let template = template_assets.get(&object_ref.template).unwrap();
            let mut properties = template.properties.clone();

            for property in &object_ref.properties {
                let existing_property_index_option = &properties
                    .iter()
                    .position(|prop| property.name == prop.name);
                if existing_property_index_option.is_none() {
                    properties.push(property.clone());
                    continue;
                }
                let existing_property_idx = existing_property_index_option.unwrap();
                properties[existing_property_idx] = property.clone();
            }

            let mut z: f32 = 0.0;

            if let Some(z_property) = properties.iter().find(|p| p.name == "z") {
                z = z_property.value_f as f32;
            }

            let template_sprite_sheet = spritesheet_assets.get(&template.sprite_sheet).unwrap();

            objects.push(ObjectData {
                obj_type: object_ref.obj_type.clone(),
                id: object_ref.id,
                x: object_ref.x,
                y: object_ref.y,
                z,
                sprite_idx: template.sprite_idx,
                sprite_sheet: TextureAtlasData {
                    tile_width: template_sprite_sheet.tile_width,
                    columns: template_sprite_sheet.columns,
                    sprite: template_sprite_sheet.sprite.clone(),
                    texture_atlas_layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                        Vec2::splat(template_sprite_sheet.tile_width.into()),
                        template_sprite_sheet.columns as usize,
                        1,
                        None,
                        None,
                    )),
                },
                properties,
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
                texture_atlas_layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                    Vec2::splat(map_sprite_sheet.tile_width.into()),
                    map_sprite_sheet.columns as usize,
                    1,
                    None,
                    None,
                )),
            },
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
    map_handles: Res<MapHandleIds>,
) {
    for map_handle in map_handles.maps.iter() {
        match map_assets.get(map_handle) {
            None => {
                println!("loading...");
                return;
            }
            Some(map_data) => {
                match spritesheet_assets.get(&map_data.sprite_sheet) {
                    None => {
                        println!("loading...");
                        return;
                    }
                    Some(spritesheet) => match image_assets.get(&spritesheet.sprite) {
                        None => {
                            println!("loading...");
                            return;
                        }
                        _ => {}
                    },
                }
                for o_ref in &map_data.objects {
                    match template_assets.get(&o_ref.template) {
                        None => {
                            println!("loading...");
                            return;
                        }
                        Some(template) => match spritesheet_assets.get(&template.sprite_sheet) {
                            None => {
                                println!("loading...");
                                return;
                            }
                            Some(spritesheet) => match image_assets.get(&spritesheet.sprite) {
                                None => {
                                    println!("loading...");
                                    return;
                                }
                                _ => {}
                            },
                        },
                    }
                }
            }
        }
    }
    println!("loaded!");
    next_state.set(MapLoadState::Done);
}

pub fn tile_data_from_coord(coord: IVec2, map_data: &MapData) -> u8 {
    let x = coord.x as usize;
    let y = MAP_WIDTH_COORD as usize - coord.y as usize - 1; // this is a little funky
    return *map_data
        .data
        .get(x + y * MAP_WIDTH_COORD as usize)
        .expect("tile data from coord oob!");
}
