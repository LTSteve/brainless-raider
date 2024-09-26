use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};

// Plugin

pub struct PitsAndPlanksPlugin;
impl Plugin for PitsAndPlanksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators)
            .add_systems(
                PostUpdate,
                (hide_inactive_planks, initialize_planks_triggers),
            )
            .add_systems(PreUpdate, toggle_planks_triggers)
            .add_systems(Update, movers_fall_into_pits);
    }
}

// Components

#[derive(Debug, Component)]
pub struct Planks {
    pub id: u16,
    pub active: bool,
}

#[derive(Debug, Component)]
pub struct PlanksTrigger {
    pub planks: Vec<Entity>,
    pub planks_ids: Vec<i64>,
    pub active_planks_idx: usize,
}

#[derive(Debug, Component, Default)]
pub struct Pit;

#[derive(Debug, Component)]
pub struct OverPlanksCounter(pub u16);

#[derive(Debug, Component)]
pub struct OverPitCounter(pub u16);

// Hydrators

fn hydrate_planks(entity_commands: &mut EntityCommands, object_data: &ObjectData, _: &World) {
    let active = get_property_value_from_object_or_default_b(object_data, "active", true);
    entity_commands.insert((
        Planks {
            id: object_data.id,
            active,
        },
        Uninintialized,
    ));
}

fn hydrate_planks_trigger(
    entity_commands: &mut EntityCommands,
    object_data: &ObjectData,
    _: &World,
) {
    let mut planks_ids = Vec::<i64>::new();
    let mut idx = 1;

    loop {
        let planks_name = format!("{}{}", "planks_", idx);
        let planks_id = get_property_value_from_object_or_default_i(object_data, &planks_name, -1);
        if planks_id == -1 {
            break;
        }
        planks_ids.push(planks_id);
        idx += 1;
    }

    entity_commands.insert((
        PlanksTrigger {
            planks: Vec::new(),
            planks_ids,
            active_planks_idx: 0,
        },
        Uninintialized,
    ));
}

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators
        .register_hydrator("Planks", hydrate_planks)
        .register_hydrator("PlanksTrigger", hydrate_planks_trigger)
        .register_tag::<Pit>("Pit");
}

fn hide_inactive_planks(mut planks_q: Query<(&mut Sprite, &Planks)>) {
    for (mut planks_sprite, planks) in planks_q.iter_mut() {
        planks_sprite.color = if planks.active {
            Color::WHITE
        } else {
            Color::NONE
        };
    }
}

fn initialize_planks_triggers(
    planks_q: Query<(Entity, &Planks)>,
    mut planks_trigger_q: Query<(Entity, &mut PlanksTrigger), With<Uninintialized>>,
    mut commands: Commands,
) {
    for (planks_trigger_entity, mut planks_trigger) in planks_trigger_q.iter_mut() {
        for planks_id in planks_trigger.planks_ids.clone() {
            for (planks_entity, planks) in planks_q.iter() {
                if planks.id == planks_id as u16 {
                    planks_trigger.planks.push(planks_entity);
                    break;
                }
            }
        }

        commands
            .entity(planks_trigger_entity)
            .remove::<Uninintialized>();
    }
}

fn toggle_planks_triggers(
    mut ev_mouse_click: EventReader<MouseClickEvent>,
    mut planks_trigger_q: Query<&mut PlanksTrigger, With<ClickableArea>>,
    mut planks_q: Query<(&mut Planks, &mut Collider)>,
    audio_server: Option<Res<AudioServer>>,
    mut commands: Commands,
) {
    for e in ev_mouse_click.read() {
        let entity = e.0;
        if let Ok(mut planks_trigger) = planks_trigger_q.get_mut(entity) {
            if planks_trigger.planks.len() == 1 {
                if let Ok((mut planks, mut collider)) =
                    planks_q.get_mut(planks_trigger.planks[planks_trigger.active_planks_idx])
                {
                    planks.active = !planks.active;
                    collider.active = planks.active;
                }
            } else {
                if let Ok((mut planks, mut collider)) =
                    planks_q.get_mut(planks_trigger.planks[planks_trigger.active_planks_idx])
                {
                    planks.active = false;
                    collider.active = false;
                }

                planks_trigger.active_planks_idx =
                    (planks_trigger.active_planks_idx + 1) % planks_trigger.planks.len();

                if let Ok((mut planks, mut collider)) =
                    planks_q.get_mut(planks_trigger.planks[planks_trigger.active_planks_idx])
                {
                    planks.active = true;
                    collider.active = true;
                }

                if let Some(audio_server) = &audio_server {
                    commands.spawn(audio_server.click.create_one_shot());
                }
            }
        }
    }
}

fn movers_fall_into_pits(
    mover_q: Query<
        (
            Entity,
            &OverPlanksCounter,
            &OverPitCounter,
            Option<&Adventurer>,
        ),
        With<Mover>,
    >,
    audio_server: Option<Res<AudioServer>>,
    mut commands: Commands,
) {
    for (entity, over_planks_counter, over_pit_counter, adventurer) in mover_q.iter() {
        if over_planks_counter.0 > 0 {
            continue;
        }
        if over_pit_counter.0 > 0 {
            if let Some(audio_server) = &audio_server {
                if let Some(_) = adventurer {
                    commands.spawn(audio_server.die.create_one_shot());
                } else {
                    commands.spawn(audio_server.kill.create_one_shot());
                }
            }
            commands.entity(entity).insert(Dead { killed_by: None });
        }
    }
}
