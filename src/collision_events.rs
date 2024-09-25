use std::f32::consts::PI;

use crate::*;
use bevy::{
    ecs::query::{QueryData, QueryFilter},
    prelude::*,
};

// Plugin

pub struct CollisionEventsPlugin;
impl Plugin for CollisionEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators).add_systems(
            Update,
            (
                on_adventurer_goblinoid_collide,
                on_mover_treasure_collide,
                on_adventurer_exit_collide.run_if(in_state(MapLoadState::Done)),
                on_mover_portal_collide,
                on_mover_pit_collide,
                on_mover_pit_uncollide,
                on_mover_planks_collide,
                on_mover_planks_uncollide,
            ),
        );
    }
}

// Systems

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators
        .register_tag::<Goblinoid>("Goblinoid")
        .register_tag::<Adventurer>("Adventurer")
        .register_tag::<Treasure>("Treasure")
        .register_tag::<Exit>("Exit");
}

pub fn on_adventurer_goblinoid_collide(
    mut commands: Commands,
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    adventurer_q: Query<Entity, With<Adventurer>>,
    mut goblinoid_q: Query<Entity, With<Goblinoid>>,
    audio_server: Option<Res<AudioServer>>,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &adventurer_q);

        if let (Ok(adventurer_entity), Ok(goblinoid_entity)) =
            (adventurer_q.get(entity1), goblinoid_q.get_mut(entity2))
        {
            commands.entity(goblinoid_entity).insert(Dead {
                killed_by: Some(adventurer_entity),
            });
            if let Some(audio_server) = &audio_server {
                commands.spawn(audio_server.kill.create_one_shot());
            }
        }
    }
}

pub fn on_mover_treasure_collide(
    mut commands: Commands,
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    mover_q: Query<(Entity, &Mover)>,
    mut treasure_q: Query<(Entity, &mut Collider), With<Treasure>>,
    audio_server: Option<Res<AudioServer>>,
    mut treasure_train_q: Query<&mut TreasureTrain>,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &mover_q);
        if let (Ok((mover_entity, mover)), Ok((treasure_entity, mut treasure_collider))) =
            (mover_q.get(entity1), treasure_q.get_mut(entity2))
        {
            treasure_collider.active = false;
            if let Some(audio_server) = &audio_server {
                commands.spawn(audio_server.pick_up.create_one_shot());
            }

            let mut found_treasure_train: Option<Mut<TreasureTrain>> = None;

            for tr in treasure_train_q.iter_mut() {
                if tr.mover == mover_entity {
                    found_treasure_train = Some(tr);
                    break;
                }
            }

            if let Some(mut treasure_train_entity) = found_treasure_train {
                treasure_train_entity.treasures.push(treasure_entity);
            } else {
                commands.spawn(TreasureTrain {
                    mover: mover_entity,
                    treasures: vec![treasure_entity],
                    head_spot: mover.target,
                    target_spots: vec![mover.coord],
                });
            }
        }
    }
}

pub fn on_adventurer_exit_collide(
    mut commands: Commands,
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    adventurer_q: Query<Entity, With<Adventurer>>,
    exit_q: Query<&Exit>,
    audio_server: Option<Res<AudioServer>>,
    mut next_state: ResMut<NextState<SceneState>>,
    mut map_server: ResMut<MapServer>,
    treasure_count: ResMut<TreasureCount>,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &adventurer_q);
        if let (Ok(entity), Ok(_)) = (adventurer_q.get(entity1), exit_q.get(entity2)) {
            if treasure_count.map_treasures == treasure_count.player_treasures {
                if let Some(audio_server) = &audio_server {
                    commands.spawn(audio_server.exit.create_one_shot());
                }
                map_server.map_idx = (map_server.map_idx + 1) % 5; // TODO: temp
                next_state.set(SceneState::Transitioning);
            } else {
                if let Some(audio_server) = &audio_server {
                    commands.spawn(audio_server.die.create_one_shot());
                }
                commands.entity(entity).insert(Dead { killed_by: None });
            }
        }
    }
}

pub fn on_mover_portal_collide(
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    mut mover_q: Query<(&mut Mover)>,
    portal_q: Query<&EnterPortal>,
    exit_portal_q: Query<(&Transform, &ExitPortal), Without<Mover>>,
    audio_server: Option<Res<AudioServer>>,
    mut commands: Commands,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &mover_q);

        if let (Ok(mut mover), Ok(portal)) = (mover_q.get_mut(entity1), portal_q.get(entity2)) {
            if let Some(exit_entity) = portal.exit_portal {
                if let Ok((exit_transform, exit_portal)) = exit_portal_q.get(exit_entity) {
                    let teleport_start_coord = mover.target;
                    let teleport_end_coord = IVec2::new(
                        pos_to_coord(exit_transform.translation.x) as i32,
                        pos_to_coord(exit_transform.translation.y) as i32,
                    );

                    mover.coord = teleport_start_coord;
                    mover.target = teleport_end_coord;
                    mover.dir = exit_portal.exit_dir;
                    mover.move_percent = -(1.0 - mover.move_percent); // :D

                    if let Some(audio_server) = &audio_server {
                        commands.spawn(audio_server.portal.create_one_shot());
                    }
                }
            }
        }
    }
}

pub fn on_mover_pit_collide(
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    mut mover_q: Query<&mut OverPitCounter, With<Mover>>,
    pit_q: Query<&Pit, Without<Mover>>,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &mover_q);

        if let (Ok(mut over_pit_counter), Ok(_)) = (mover_q.get_mut(entity1), pit_q.get(entity2)) {
            over_pit_counter.0 += 1;
        }
    }
}

pub fn on_mover_pit_uncollide(
    mut ev_collision_exit: EventReader<CollisionExitEvent>,
    mut mover_q: Query<&mut OverPitCounter, With<Mover>>,
    pit_q: Query<&Pit, Without<Mover>>,
) {
    for e in ev_collision_exit.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &mover_q);

        if let (Ok(mut over_pit_counter), Ok(_)) = (mover_q.get_mut(entity1), pit_q.get(entity2)) {
            over_pit_counter.0 -= 1;
        }
    }
}

pub fn on_mover_planks_collide(
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    mut mover_q: Query<&mut OverPlanksCounter, With<Mover>>,
    planks_q: Query<&Planks, Without<Mover>>,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &mover_q);

        if let (Ok(mut over_planks_counter), Ok(planks)) =
            (mover_q.get_mut(entity1), planks_q.get(entity2))
        {
            if planks.active {
                over_planks_counter.0 += 1;
            }
        }
    }
}

pub fn on_mover_planks_uncollide(
    mut ev_collision_exit: EventReader<CollisionExitEvent>,
    mut mover_q: Query<&mut OverPlanksCounter, With<Mover>>,
    planks_q: Query<&Planks, Without<Mover>>,
) {
    for e in ev_collision_exit.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &mover_q);

        if let (Ok(mut over_planks_counter), Ok(_)) =
            (mover_q.get_mut(entity1), planks_q.get(entity2))
        {
            if over_planks_counter.0 > 0 {
                over_planks_counter.0 -= 1;
            }
        }
    }
}

// Helpers

fn align_entities<T, F>(e1: Entity, e2: Entity, query_first: &Query<T, F>) -> (Entity, Entity)
where
    T: QueryData,
    F: QueryFilter,
{
    if let Ok(_) = query_first.get(e1) {
        return (e1, e2);
    }
    return (e2, e1);
}
