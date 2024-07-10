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
        .add_systems(Update, hello_data)
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

fn hello_data(
    map_handles: Res<MapHandleIds>,
    map_assets: Res<Assets<RawMapData>>,
    spritesheet_assets: Res<Assets<SpritesheetData>>,
    template_assets: Res<Assets<TemplateData>>,
    image_assets: Res<Assets<Image>>
) {
    let map = match map_assets.get(&map_handles.tutorial_maps[0]) {
        Some(map) => map,
        None => {
            println!("map not loaded yet...");
            return
        }
    };
    println!("map loaded!");

    let spritesheet = match spritesheet_assets.get(&map.sprite_sheet) {
        Some(spritesheet) => spritesheet,
        None => {
            println!("spritesheet not loaded yet...");
            return
        }
    };
    println!("spritesheet loaded!");

    let sprite = match image_assets.get(&spritesheet.sprite) {
        Some(sprite) => sprite,
        None => {
            println!("sprite not loaded yet...");
            return
        }
    };
    println!("sprite loaded!");

    let template = match template_assets.get(&map.objects[0].template) {
        Some(template) => template,
        None => {
            println!("template not loaded yet...");
            return
        }
    };
    println!("template loaded!");

    let template_spritesheet = match spritesheet_assets.get(&template.sprite_sheet) {
        Some(template_spritesheet) => template_spritesheet,
        None => {
            println!("template spritesheet not loaded yet...");
            return
        }
    };
    let template_sprite = match image_assets.get(&template_spritesheet.sprite) {
        Some(template_sprite) => template_sprite,
        None => {
            println!("template spritesheet sprite not loaded yet...");
            return
        }
    };
    println!("template spritesheet sprite loaded!");

    println!("spritesheet tile width: {:?}", spritesheet.tile_width);
    println!("sprite height: {:?}", sprite.height());
    println!("template sprite idx: {:?}", template.sprite_idx);
    println!("template spritesheet sprite is the same as the spritesheet sprite?: {:?}", std::ptr::eq(template_sprite, sprite));
}