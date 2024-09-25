use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::window::PrimaryWindow;

use crate::*;

// Constants

const LABEL_PADDING: f32 = 10.0;

// Plugin
pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SceneState>()
            .add_systems(OnEnter(MapLoadState::Done), setup_scene)
            .add_systems(
                OnEnter(SceneState::Transitioning),
                (tear_down_scene, setup_scene)
                    .chain()
                    .run_if(in_state(MapLoadState::Done)),
            );
    }
}

// Components
#[derive(Debug, Component)]
pub struct NoTearDown;

#[derive(Debug, Component)]
pub struct LivesLabel;

// States

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum SceneState {
    Stable,
    #[default]
    Transitioning,
}

// Systems

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
    camera_query: Query<&Camera2d>,
    mut next_state: ResMut<NextState<SceneState>>,
    audio_server: Option<Res<AudioServer>>,
    active_sfx_query: Query<&AudioSink>,
    entity_hydrator: Res<ComponentHydrators>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let map = &map_server.maps[map_server.map_idx];
    let texture = &map.sprite_sheet.sprite;
    let window = window_query.get_single().expect("Couldn't find window");

    if let Err(err) = camera_query.get_single() {
        if let bevy::ecs::query::QuerySingleError::NoEntities(_) = err {
            commands.spawn((Camera2dBundle::default(), NoTearDown));

            // This is a little dirty

            let text_style = TextStyle {
                font_size: 60.0,
                ..Default::default()
            };

            commands.spawn((
                Text2dBundle {
                    text: Text::from_sections([
                        TextSection::new("Lives ", text_style.clone()),
                        TextSection::new(MAX_LIVES.to_string(), text_style.clone()),
                    ]),
                    text_anchor: Anchor::TopRight,
                    transform: Transform {
                        translation: Vec3::new(
                            window.width() / 2.0 - LABEL_PADDING,
                            window.height() / 2.0 - LABEL_PADDING,
                            0.0,
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                LivesLabel,
                NoTearDown,
            ));
        }
    }

    // TODO: temp, realistically this will go in the menu / initialization section
    if let Some(audio_server) = audio_server {
        if let Err(err) = active_sfx_query.get_single() {
            if let bevy::ecs::query::QuerySingleError::NoEntities(_) = err {
                commands.spawn(audio_server.dumbraider.create_loop());
            }
        }
    }

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

        let mut entity_commands = commands.spawn((sprite_bundle, texture_atlas));

        let components_property = obj.properties.iter().find(|prop| prop.name == "Components");

        if let Some(components_property) = components_property {
            for component_name in String::from(&components_property.value_s).split("|") {
                entity_hydrator.hydrate_entity(&mut entity_commands, &obj, component_name)
            }
        }
    }
    next_state.set(SceneState::Stable);
}
