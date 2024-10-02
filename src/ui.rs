use bevy::{ecs::system::EntityCommands, prelude::*, sprite::Anchor, window::PrimaryWindow};

use crate::{
    get_property_value_from_object_or_default_f, get_property_value_from_object_or_default_s,
    AudioServer, BackgroundLoop, ComponentHydrators, MapLoadState, MapServer, ObjectData,
    SceneState, Uninintialized,
};

// Constants

pub const TEXT_COLOR: &str = "C5CCB8";
pub const SUCCESS_COLOR: &str = "6EAA78";

const NORMAL_BUTTON: Color = Color::rgba(0.0, 0.0, 0.0, 0.3);
const HOVERED_BUTTON: Color = Color::rgba(0.0, 0.0, 0.0, 0.8);
const PRESSED_BUTTON: Color = Color::rgba(0.0, 0.0, 0.0, 1.0);

// Plugin

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators)
            .add_systems(Update, (initialize_background_loop, initialize_labels))
            .add_systems(
                Update,
                start_button_system.run_if(in_state(MapLoadState::Done)),
            );
    }
}

// Components

#[derive(Debug, Component, Default)]
pub struct LivesLabel;

#[derive(Debug, Component, Default)]
pub struct TreasuresLabel;

#[derive(Debug, Component, Default)]
pub struct TitleLabel;

#[derive(Debug, Component)]
pub struct LabelProperties {
    pub offset: Vec2,
}

#[derive(Debug, Component, Default)]
pub struct StartButton;

// Hydrators

pub fn hydrate_label(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let text_style = TextStyle {
        font_size: 60.0,
        color: Color::hex(TEXT_COLOR).expect("invalid hex color"),
        ..Default::default()
    };

    let anchor_s =
        get_property_value_from_object_or_default_s(object_data, "anchor", "TopLeft".into());

    let anchor = if anchor_s == "TopLeft" {
        Anchor::TopLeft
    } else {
        if anchor_s == "TopRight" {
            Anchor::TopRight
        } else {
            Anchor::Center
        }
    };

    let mut sections = Vec::<TextSection>::new();
    let mut idx = 0;
    loop {
        let section = get_property_value_from_object_or_default_s(
            object_data,
            &format!("section_{}", idx),
            String::new(),
        );

        if section.is_empty() {
            break;
        }

        sections.push(TextSection::new(section, text_style.clone()));
        idx += 1;
    }

    let x_offset = get_property_value_from_object_or_default_f(object_data, "x_offset", 0.0);
    let y_offset = get_property_value_from_object_or_default_f(object_data, "y_offset", 0.0);

    entity_commands.insert((
        Text2dBundle {
            text: Text::from_sections(sections),
            text_anchor: anchor,
            visibility: Visibility::Hidden,
            transform: Transform::default(), // will be set in initialize fns
            ..Default::default()
        },
        LabelProperties {
            offset: Vec2::new(x_offset as f32, y_offset as f32),
        },
    ));
}

fn hydrate_start_button(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let text_style = TextStyle {
        font_size: 60.0,
        color: Color::hex(TEXT_COLOR).expect("invalid hex color"),
        ..Default::default()
    };

    let text =
        get_property_value_from_object_or_default_s(object_data, "text", "hello world".into());

    let x_offset = get_property_value_from_object_or_default_f(object_data, "x_offset", 0.0);
    let y_offset = get_property_value_from_object_or_default_f(object_data, "y_offset", 0.0);

    entity_commands
        .insert(NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(text.len() as f32 * 33.0),
                            height: Val::Px(60.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            left: Val::Px(x_offset as f32),
                            top: Val::Px(y_offset as f32),
                            position_type: PositionType::Relative,
                            ..Default::default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    StartButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(text, text_style));
                });
        });
}

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators
        .register_tag::<(LivesLabel, Uninintialized)>("LivesLabel")
        .register_tag::<(TreasuresLabel, Uninintialized)>("TreasuresLabel")
        .register_tag::<(TitleLabel, Uninintialized)>("TitleLabel")
        .register_hydrator("Text2dBundle", hydrate_label)
        .register_hydrator("StartButton", hydrate_start_button);
}

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

fn initialize_labels(
    mut commands: Commands,
    mut label_q: Query<(Entity, &mut Transform, &LabelProperties, &Anchor), With<Uninintialized>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for (entity, mut transform, properties, anchor) in label_q.iter_mut() {
        let window = window_query.get_single().expect("Couldn't find window");

        transform.translation = ui_location_from_anchor_offsets(
            *anchor,
            Vec2::new(window.width(), window.height()),
            properties.offset,
        );

        commands
            .entity(entity)
            .remove::<Uninintialized>()
            .remove::<Visibility>()
            .insert(Visibility::Visible);
    }
}

fn start_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<StartButton>),
    >,
    audio_server: Option<Res<AudioServer>>,
    mut next_state: ResMut<NextState<SceneState>>,
    mut map_server: ResMut<MapServer>,
    mut commands: Commands,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                if let Some(audio_server) = &audio_server {
                    commands.spawn(audio_server.click.create_one_shot());
                }
                map_server.map_idx = 1;
                next_state.set(SceneState::Transitioning);
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

// Helpers

fn ui_location_from_anchor_offsets(anchor: Anchor, window_vec: Vec2, offset_vec: Vec2) -> Vec3 {
    if anchor == Anchor::Center {
        return Vec3::new(offset_vec.x, offset_vec.y, 0.0);
    } else {
        let anchor_mod = if anchor == Anchor::TopRight {
            1.0
        } else {
            -1.0
        };

        return Vec3::new(
            (window_vec.x / 2.0 - offset_vec.x) * anchor_mod,
            window_vec.y / 2.0 - offset_vec.y,
            0.0,
        );
    }
}
