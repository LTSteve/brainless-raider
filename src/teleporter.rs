use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};

// Plugin

pub struct TeleporterPlugin;
impl Plugin for TeleporterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                initialize_enter_portals,
                initialize_teleporters,
                hide_inactive_exit_portals,
                toggle_exit_portals,
            ),
        );
    }
}

// Components

#[derive(Debug, Component)]
pub struct EnterPortal {
    id: u16,
    exit_portal_id: i64,
    pub exit_portal: Option<Entity>,
}

#[derive(Debug, Component)]
pub struct ExitPortal {
    id: u16,
    pub exit_dir: IVec2,
    active: bool,
}

#[derive(Debug, Component)]
pub struct Teleporter {
    enter_portal_id: i64,
    enter_portal: Option<Entity>,
    exit_portal_ids: Vec<i64>,
    exit_portals: Vec<Entity>,
    active_exit_portal: usize,
}

// Hydrators

pub fn hydrate_enter_portal(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let exit_portal_id = get_property_value_from_object_or_default_i(object_data, "exit_portal", 0);

    if exit_portal_id == 0 {
        println!("Hydrated EnterPortal with no exit_portal assigned!");
    }

    entity_commands.insert((
        EnterPortal {
            id: object_data.id,
            exit_portal_id,
            exit_portal: None,
        },
        Uninintialized,
    ));
}

pub fn hydrate_exit_portal(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let x = get_property_value_from_object_or_default_i(object_data, "exit_dir_x", 0);
    let y = get_property_value_from_object_or_default_i(object_data, "exit_dir_y", 0);

    entity_commands.insert(ExitPortal {
        id: object_data.id,
        exit_dir: IVec2::new(x as i32, y as i32),
        active: false,
    });
}

pub fn hydrate_teleporter(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let enter_portal_id =
        get_property_value_from_object_or_default_i(object_data, "enter_portal", 0);

    if enter_portal_id == 0 {
        println!("Hydrated ExitPortal with no exit_portal assigned!");
    }

    let mut exit_portal_ids = Vec::<i64>::new();
    let mut idx = 1;

    loop {
        let exit_portal_name = format!("{}{}", "exit_portal_", idx);
        let exit_portal_id =
            get_property_value_from_object_or_default_i(object_data, &exit_portal_name, -1);
        if exit_portal_id == -1 {
            break;
        }
        exit_portal_ids.push(exit_portal_id);
        idx += 1;
    }

    entity_commands.insert((
        Teleporter {
            enter_portal_id,
            enter_portal: None,
            exit_portal_ids,
            exit_portals: Vec::new(),
            active_exit_portal: 0,
        },
        Uninintialized,
    ));
}

// Systems

fn initialize_enter_portals(
    mut enter_q: Query<(Entity, &mut EnterPortal), With<Uninintialized>>,
    exit_q: Query<(Entity, &ExitPortal)>,
    mut commands: Commands,
) {
    for (enter_entity, mut enter_portal) in enter_q.iter_mut() {
        for (exit_entity, exit_portal) in exit_q.iter() {
            if exit_portal.id == (enter_portal.exit_portal_id as u16) {
                enter_portal.exit_portal = Some(exit_entity);
                break;
            }
        }
        commands.entity(enter_entity).remove::<Uninintialized>();
    }
}

fn initialize_teleporters(
    mut teleporter_q: Query<(Entity, &mut Teleporter), With<Uninintialized>>,
    enter_q: Query<(Entity, &mut EnterPortal)>,
    mut exit_q: Query<(Entity, &mut ExitPortal)>,
    mut commands: Commands,
) {
    for (teleporter_entity, mut teleporter) in teleporter_q.iter_mut() {
        for (enter_entity, enter_portal) in enter_q.iter() {
            if enter_portal.id == (teleporter.enter_portal_id as u16) {
                teleporter.enter_portal = Some(enter_entity);
                break;
            }
        }

        let teleporter_ids = teleporter.exit_portal_ids.clone();
        let mut first = true;

        for id in teleporter_ids.iter() {
            for (exit_entity, mut exit_portal) in exit_q.iter_mut() {
                if exit_portal.id == (*id as u16) {
                    teleporter.exit_portals.push(exit_entity);
                    exit_portal.active = first;
                    first = false;
                }
            }
        }

        commands
            .entity(teleporter_entity)
            .remove::<Uninintialized>();
    }
}

fn hide_inactive_exit_portals(mut exit_q: Query<(&mut Sprite, &ExitPortal)>) {
    for (mut sprite, exit_portal) in exit_q.iter_mut() {
        sprite.color = if exit_portal.active {
            Color::WHITE
        } else {
            Color::NONE
        };
    }
}

fn toggle_exit_portals(
    mut ev_mouse_click: EventReader<MouseClickEvent>,
    mut teleporter_q: Query<&mut Teleporter, With<ClickableArea>>,
    mut exit_q: Query<(Entity, &mut ExitPortal)>,
    mut enter_q: Query<&mut EnterPortal>,
    audio_server: Option<Res<AudioServer>>,
    mut commands: Commands,
) {
    for e in ev_mouse_click.read() {
        let entity = e.0;
        if let Ok(mut teleporter) = teleporter_q.get_mut(entity) {
            if let Ok((_, mut exit)) =
                exit_q.get_mut(teleporter.exit_portals[teleporter.active_exit_portal])
            {
                exit.active = false;
            }

            teleporter.active_exit_portal =
                (teleporter.active_exit_portal + 1) % teleporter.exit_portals.len();

            if let Ok((exit_entity, mut exit_portal)) =
                exit_q.get_mut(teleporter.exit_portals[teleporter.active_exit_portal])
            {
                exit_portal.active = true;

                if let Some(enter_portal) = teleporter.enter_portal {
                    if let Ok(mut enter) = enter_q.get_mut(enter_portal) {
                        enter.exit_portal = Some(exit_entity);
                    }
                }
            }

            if let Some(audio_server) = &audio_server {
                commands.spawn(audio_server.click.create_one_shot());
            }
        }
    }
}
