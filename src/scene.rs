use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::*;

// Constants

const TOOL_PROPERTY: &str = "_tool";

// Plugin
pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_systems(Startup, add_hydrators)
            .add_systems(
                OnEnter(MapLoadState::Done),
                (setup_scene, post_setup_scene).chain(),
            )
            .add_systems(
                OnEnter(SceneState::Transitioning),
                (tear_down_scene, setup_scene, post_setup_scene)
                    .chain()
                    .run_if(in_state(MapLoadState::Done)),
            );
    }
}

// Components

#[derive(Debug, Component, Default)]
pub struct NoTearDown;

#[derive(Debug, Component)]
struct Tool;

#[derive(Debug, Component, Default)]
pub struct BackgroundLoop;

// States

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SceneState {
    Stable,
    #[default]
    Transitioning,
}

// Hydrators

pub fn hydrate_camera(entity_commands: &mut EntityCommands, _: &ObjectData) {
    entity_commands.insert(Camera2dBundle::default());
}

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators
        .register_tag::<NoTearDown>("NoTearDown")
        .register_tag::<(BackgroundLoop, Uninintialized)>("BackgroundLoop")
        .register_hydrator("Camera2dBundle", hydrate_camera);
}

fn tear_down_scene(
    mut commands: Commands,
    to_tear_down_query: Query<Entity, (Without<NoTearDown>, Without<Window>)>,
) {
    for entity in to_tear_down_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_scene(
    mut commands: Commands,
    map_server: Res<MapServer>,
    entity_hydrator: Res<ComponentHydrators>,
) {
    let map = &map_server.maps[map_server.map_idx];
    let texture = &map.sprite_sheet.sprite;

    for idx in 0..map.data.len() {
        if map.data[idx] == 0 {
            continue;
        }

        let x = idx % map.width;
        let y = map.width - idx / map.width - 1;

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3 {
                        x: coord_to_pos(x as f32),
                        y: coord_to_pos(y as f32),
                        z: FLOOR_Z,
                    },
                    scale: Vec3::splat(SCALE),
                    ..default()
                },
                texture: texture.clone(),
                ..default()
            },
            TextureAtlas {
                layout: map.sprite_sheet.texture_atlas_layout.clone(),
                index: (map.data[idx] as usize) - 1,
            },
        ));
    }

    for obj in map.objects.iter() {
        let sprite_bundle = SpriteBundle {
            transform: Transform {
                translation: Vec3 {
                    x: coord_to_pos(obj.x as f32),
                    y: coord_to_pos(obj.y as f32),
                    z: obj.z + ENTITY_Z_OFFSET,
                },
                scale: Vec3::splat(SCALE),
                ..default()
            },
            texture: texture.clone(),
            ..default()
        };
        let texture_atlas = TextureAtlas {
            layout: obj.sprite_sheet.texture_atlas_layout.clone(),
            index: obj.sprite_idx as usize - 1,
        };

        let tool_property = obj
            .properties
            .iter()
            .find(|prop| prop.name == TOOL_PROPERTY);
        let is_tool = tool_property.is_some() && tool_property.unwrap().value_b;

        let mut entity_commands = commands.spawn(());

        if is_tool {
            entity_commands.insert(Tool);
        } else {
            entity_commands.insert((sprite_bundle, texture_atlas));
        }

        let components_property = obj.properties.iter().find(|prop| prop.name == "Components");

        if let Some(components_property) = components_property {
            for component_name in String::from(&components_property.value_s).split("|") {
                entity_hydrator.hydrate_entity(&mut entity_commands, &obj, component_name);
            }
        }
    }
}

fn post_setup_scene(mut next_state: ResMut<NextState<SceneState>>) {
    next_state.set(SceneState::Stable);
}
