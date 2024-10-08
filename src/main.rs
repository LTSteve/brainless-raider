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
mod scene;
mod tags;
mod teleporter;
mod treasure_train;
mod ui;

use audio_server::*;
use bevy::prelude::*;
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
use scene::*;
use tags::*;
use teleporter::*;
use treasure_train::*;
use ui::*;

fn main() {
    App::new()
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
                String::from("maps/tutorial/2.tmx"),
                String::from("maps/tutorial/3.tmx"),
                String::from("maps/tutorial/0.tmx"),
                String::from("maps/tutorial/1.tmx"),
                String::from("maps/tutorial/4.tmx"),
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
            UIPlugin,
        ))
        .run();
}
