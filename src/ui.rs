use bevy::{ecs::system::EntityCommands, prelude::*, sprite::Anchor, window::PrimaryWindow};
use bevy_utils::HashMap;

use crate::{
    audio_server, get_property_value_from_object_or_default_f,
    get_property_value_from_object_or_default_s, AudioServer, BackgroundLoop, ComponentHydrators,
    NoTearDown, ObjectData, Uninintialized, MAX_LIVES,
};

// Constants

pub const TEXT_COLOR: &str = "C5CCB8";
pub const SUCCESS_COLOR: &str = "6EAA78";

// Plugin

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators)
            .add_systems(Update, (initialize_background_loop, initialize_labels));
    }
}

// Components

#[derive(Debug, Component, Default)]
pub struct LivesLabel;

#[derive(Debug, Component, Default)]
pub struct TreasuresLabel;

#[derive(Debug, Component)]
pub struct LabelProperties {
    pub offset: Vec2,
}

// Hydrators

pub fn hydrate_label(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let text_style = TextStyle {
        font_size: 60.0,
        color: Color::hex(TEXT_COLOR).expect("invalid hex color"),
        ..Default::default()
    };

    let anchor =
        if get_property_value_from_object_or_default_s(object_data, "anchor", "TopLeft".into())
            == "TopLeft"
        {
            Anchor::TopLeft
        } else {
            Anchor::TopRight
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

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators
        .register_tag::<(LivesLabel, Uninintialized)>("LivesLabel")
        .register_tag::<(TreasuresLabel, Uninintialized)>("TreasuresLabel")
        .register_hydrator("Text2dBundle", hydrate_label);
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

        transform.translation = label_location_from_anchor_offsets(
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

// Helpers

fn label_location_from_anchor_offsets(anchor: Anchor, window_vec: Vec2, offset_vec: Vec2) -> Vec3 {
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
