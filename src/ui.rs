use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_utils::HashMap;

use crate::{
    audio_server, AudioServer, BackgroundLoop, ComponentHydrators, NoTearDown, ObjectData,
    Uninintialized,
};

// Plugin

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators)
            .add_systems(Update, initialize_background_loop);
    }
}

// Components

#[derive(Debug, Component)]
pub struct LivesLabel;

#[derive(Debug, Component)]
pub struct TreasuresLabel;

// Hydrators

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {}

fn initialize_background_loop(
    mut commands: Commands,
    audio_server: Option<Res<AudioServer>>,
    background_loop_q: Query<Entity, (With<BackgroundLoop>, With<Uninintialized>)>,
) {
    if let Ok(entity) = background_loop_q.get_single() {
        if audio_server.is_none() {
            return;
        }
        let audio_server = audio_server.unwrap();
        commands.spawn(audio_server.dumbraider.create_loop());
        commands.entity(entity).remove::<Uninintialized>();
    }
}
