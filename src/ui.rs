use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_utils::HashMap;

use crate::{BackgroundLoop, ComponentHydrators, NoTearDown, ObjectData};

// Plugin

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators);
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
