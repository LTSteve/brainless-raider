use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;

use crate::*;

// Constants

const TOOL_PROPERTY: &str = "_tool";

// Plugin
pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_systems(Startup, (add_hydrators, setup_camera))
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

#[derive(Debug, Component)]
pub struct NoTearDown {
    pub id: String,
    pub ignore_duplicates: bool,
}

#[derive(Debug, Component)]
struct Tool;

#[derive(Debug, Component, Default)]
pub struct BackgroundLoop;

#[derive(Debug, Component, Default)]
pub struct Uninintialized;

// States

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SceneState {
    Stable,
    #[default]
    Transitioning,
}

// Hydrators

pub fn hydrate_no_tear_down(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    entity_commands.insert(NoTearDown {
        id: object_data.name.clone(),
        ignore_duplicates: false,
    });
}

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators
        .register_tag::<(BackgroundLoop, Uninintialized)>("BackgroundLoop")
        .register_hydrator("NoTearDown", hydrate_no_tear_down);
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
    let map = map_server.get_current_map();
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
            entity_commands.insert((sprite_bundle, texture_atlas, PIXEL_PERFECT_LAYERS));
        }

        let components_property = obj.properties.iter().find(|prop| prop.name == "Components");

        if let Some(components_property) = components_property {
            for component_name in String::from(&components_property.value_s).split("|") {
                entity_hydrator.hydrate_entity(&mut entity_commands, &obj, component_name);
            }
        }
    }
}

fn post_setup_scene(
    mut next_state: ResMut<NextState<SceneState>>,
    mut commands: Commands,
    no_tear_down_q: Query<(Entity, &NoTearDown)>,
) {
    let mut combinations = no_tear_down_q.iter_combinations::<2>();
    while let Some([(entity1, no_tear_down1), (_, no_tear_down2)]) = combinations.fetch_next() {
        if no_tear_down1.ignore_duplicates || no_tear_down2.ignore_duplicates {
            continue;
        }
        if no_tear_down1.id.eq(&no_tear_down2.id) {
            commands.entity(entity1).despawn();
        }
    }

    next_state.set(SceneState::Stable);
}
