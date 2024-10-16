mod audio_server;
mod brmap;
mod clickable_area;
mod collision;
mod collision_events;
mod death;
mod helpers;
mod hydrate_components;
mod map_loader;
mod movement;
mod pause;
mod pits_and_planks;
mod pixel_perfect_camera;
mod scene;
mod tags;
mod teleporter;
mod treasure_train;
mod ui;
mod you_win;

use audio_server::*;
use bevy::{asset::load_internal_binary_asset, prelude::*};
use brmap::*;
use clickable_area::*;
use collision::*;
use collision_events::*;
use death::*;
use helpers::*;
use hydrate_components::*;
use movement::*;
use pause::*;
use pits_and_planks::*;
use pixel_perfect_camera::*;
use scene::*;
use tags::*;
use teleporter::*;
use treasure_train::*;
use ui::*;
use you_win::*;

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa::Off)
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    mode: AssetMode::Unprocessed,
                    file_path: "res".to_string(),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            HydrateComponentsPlugin,
            BRMapPlugin(vec![
                String::from("maps/tutorial/title.tmx"),
                String::from("maps/tutorial/0.tmx"),
                String::from("maps/tutorial/1.tmx"),
                String::from("maps/tutorial/2.tmx"),
                String::from("maps/tutorial/3.tmx"),
                String::from("maps/tutorial/4.tmx"),
                String::from("maps/tutorial/youwin.tmx"),
            ]),
            CollisionPlugin {
                debug_collisions: false,
            },
            CollisionEventsPlugin,
            AudioServerPlugin,
            TreasureTrainPlugin,
            ScenePlugin,
            MovementPlugin,
            ClickableAreaPlugin { debug_clicks: true },
            TeleporterPlugin,
            PitsAndPlanksPlugin,
            DeathPlugin,
            PausePlugin,
        )) // Yo, you can only have so many plugins per call to add_plugins
        .add_plugins((UIPlugin, PixelPerfectCameraPlugin, YouWinPlugin));

    // This needs to happen after `DefaultPlugins` is added.
    load_internal_binary_asset!(
        app,
        TextStyle::default().font,
        "../res/PressStart2P-Regular.ttf",
        |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
    );

    app.run();
}
