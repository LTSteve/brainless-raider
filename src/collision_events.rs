use std::f32::consts::PI;

use crate::*;
use bevy::{
    ecs::query::{QueryData, QueryEntityError, QueryFilter},
    prelude::*,
};

// Constants

pub const GOBLINOID_DEATH_OFFSET: f32 = 2.0; // Under treasure
pub const GOBLINOID_DEATH_SCALE: f32 = 0.8;
pub const GOBLINOID_DEATH_COLOR: Color = Color::GRAY;
pub const GOBLINOID_DEATH_ROTATION: f32 = 90.0;

// Plugin

pub struct CollisionEventsPlugin;
impl Plugin for CollisionEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (on_adventurer_goblinoid_collide, on_mover_treasure_collide),
        );
    }
}

// Systems

pub fn on_adventurer_goblinoid_collide(
    mut commands: Commands,
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    adventurer_q: Query<Entity, With<Adventurer>>,
    mut goblinoid_q: Query<(Entity, &mut Transform, &mut Sprite), With<Goblinoid>>,
    audio_server: Option<Res<AudioServer>>,
    treasure_train_q: Query<(Entity, &TreasureTrain)>,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &adventurer_q);

        if let (Ok(_), Ok((goblinoid_entity, mut goblinoid_transform, mut goblinoid_sprite))) =
            (adventurer_q.get(entity1), goblinoid_q.get_mut(entity2))
        {
            for (entity, treasure_train) in treasure_train_q.iter() {
                if treasure_train.mover == goblinoid_entity {
                    // re enable colliders for all treasures on the train
                    for &treasure_entity in &treasure_train.treasures {
                        commands
                            .entity(treasure_entity)
                            .remove::<ColliderDisabled>();
                    }
                    // despawn the treasure train
                    commands.entity(entity).despawn();
                    break;
                }
            }
            commands.entity(goblinoid_entity).remove::<Mover>();
            commands.entity(goblinoid_entity).insert(ColliderDisabled);
            goblinoid_transform.rotate_z(deg_to_rad(GOBLINOID_DEATH_ROTATION));
            goblinoid_transform.scale = goblinoid_transform.scale * GOBLINOID_DEATH_SCALE;
            goblinoid_transform.translation.z -= GOBLINOID_DEATH_OFFSET;
            goblinoid_sprite.color = GOBLINOID_DEATH_COLOR;
            if let Some(audio_server) = &audio_server {
                commands.spawn(audio_server.kill.create_one_shot());
            }
        }
    }
}

pub fn on_mover_treasure_collide(
    mut commands: Commands,
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    adventurer_q: Query<(Entity, &Mover)>,
    treasure_q: Query<Entity, With<Treasure>>,
    audio_server: Option<Res<AudioServer>>,
    mut treasure_train_q: Query<&mut TreasureTrain>,
) {
    for e in ev_collision_enter.read() {
        let (entity1, entity2) = align_entities(e.0, e.1, &adventurer_q);
        if let (Ok((adventurer_entity, mover)), Ok(treasure_entity)) =
            (adventurer_q.get(entity1), treasure_q.get(entity2))
        {
            commands.entity(treasure_entity).insert(ColliderDisabled);
            if let Some(audio_server) = &audio_server {
                commands.spawn(audio_server.pick_up.create_one_shot());
            }

            let mut found_treasure_train: Option<Mut<TreasureTrain>> = None;

            for tr in treasure_train_q.iter_mut() {
                if tr.mover == adventurer_entity {
                    found_treasure_train = Some(tr);
                    break;
                }
            }

            if let Some(mut treasure_train_entity) = found_treasure_train {
                treasure_train_entity.treasures.push(treasure_entity);
            } else {
                commands.spawn(TreasureTrain {
                    mover: adventurer_entity,
                    treasures: vec![treasure_entity],
                    head_spot: mover.target,
                    target_spots: vec![mover.coord],
                });
            }
        }
    }
}

// Helpers
fn deg_to_rad(deg: f32) -> f32 {
    return deg * PI / 180.0;
}

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
