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
        .add_systems(OnEnter(MapLoadState::Done), (setup, hello_data))
        .run();
}

#[derive(Resource)]
struct MapHandleIds {
    tutorial_maps: Vec<Handle<RawMapData>>
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(MapHandleIds {
        tutorial_maps: vec![
            assets.load("res/maps/tutorial/0.tmx"),
            assets.load("res/maps/tutorial/1.tmx"),
            assets.load("res/maps/tutorial/2.tmx"),
            assets.load("res/maps/tutorial/3.tmx"),
            assets.load("res/maps/tutorial/4.tmx")
        ]
    });
}

fn hello_data(
    map_handles: Res<MapHandleIds>,
    map_assets: Res<Assets<RawMapData>>,
    spritesheet_assets: Res<Assets<SpritesheetData>>,
    template_assets: Res<Assets<TemplateData>>,
    image_assets: Res<Assets<Image>>
) {
    let map = map_assets.get(&map_handles.tutorial_maps[0]).unwrap();
    println!("map loaded!");

    let spritesheet = spritesheet_assets.get(&map.sprite_sheet).unwrap();
    println!("spritesheet loaded!");

    let sprite = image_assets.get(&spritesheet.sprite).unwrap();
    println!("sprite loaded!");

    let template = template_assets.get(&map.objects[0].template).unwrap();
    println!("template loaded!");

    let template_spritesheet = spritesheet_assets.get(&template.sprite_sheet).unwrap();
    let template_sprite = image_assets.get(&template_spritesheet.sprite).unwrap();
    println!("template spritesheet sprite loaded!");

    println!("spritesheet tile width: {:?}", spritesheet.tile_width);
    println!("sprite height: {:?}", sprite.height());
    println!("template sprite idx: {:?}", template.sprite_idx);
    println!("template spritesheet sprite is the same as the spritesheet sprite?: {:?}", std::ptr::eq(template_sprite, sprite));
}