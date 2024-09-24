use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};
use std::f32::consts::PI;

// Constants

pub const DEATH_OFFSET: f32 = 2.0; // Under treasure
pub const DEATH_SCALE: f32 = 0.8;
pub const DEATH_COLOR: Color = Color::GRAY;
pub const DEATH_ROTATION: f32 = 90.0;

pub const DEATH_DELAY: f32 = 1.0;

// Plugin

pub struct DeathPlugin;
impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movers_die).add_systems(
            Update,
            dead_adventurers_respawn().run_if(in_state(PauseState::Running)),
        );
    }
}

// Components

#[derive(Debug, Component)]
pub struct Dead {
    pub killed_by: Option<Entity>,
}

// Systems

fn movers_die(
    mut dead_mover_q: Query<
        (Entity, &mut Transform, &mut Sprite, &mut Collider, &Dead),
        With<Mover>,
    >,
    mut treasure_train_q: Query<(Entity, &mut TreasureTrain)>,
    mut treasure_collider_q: Query<&mut Collider, (With<Treasure>, Without<Mover>)>,
    mut commands: Commands,
) {
    for (mover_entity, mut transform, mut sprite, mut collider, dead) in dead_mover_q.iter_mut() {
        let mut did_treasure_transfer = false;

        if let Some(killed_by) = dead.killed_by {
            let mut killed_train_e: Option<Entity> = None;
            let mut killed_train: Option<Mut<'_, TreasureTrain>> = None;
            let mut killer_train: Option<Mut<'_, TreasureTrain>> = None;

            for (entity, treasure_train) in treasure_train_q.iter_mut() {
                if treasure_train.mover == mover_entity {
                    killed_train = Some(treasure_train);
                    killed_train_e = Some(entity);
                } else if treasure_train.mover == killed_by {
                    killer_train = Some(treasure_train);
                }
            }

            if let (Some(killed_train_e), Some(killed_train), Some(mut killer_train)) =
                (killed_train_e, killed_train, killer_train)
            {
                for treasure in &killed_train.treasures {
                    killer_train.treasures.push(*treasure);
                }

                // despawn the treasure train
                commands.entity(killed_train_e).despawn();

                did_treasure_transfer = true;
            }
        }

        if !did_treasure_transfer {
            for (entity, treasure_train) in treasure_train_q.iter() {
                if treasure_train.mover == mover_entity {
                    // re enable colliders for all treasures on the train
                    for &treasure_entity in &treasure_train.treasures {
                        if let Ok(mut treasure_collider) =
                            treasure_collider_q.get_mut(treasure_entity)
                        {
                            treasure_collider.active = true;
                        }
                    }
                    // despawn the treasure train
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }

        commands.entity(mover_entity).remove::<Mover>();
        collider.active = false;
        transform.rotate_z(deg_to_rad(DEATH_ROTATION));
        transform.scale = transform.scale * DEATH_SCALE;
        transform.translation.z -= DEATH_OFFSET;
        sprite.color = DEATH_COLOR;
    }
}

fn dead_adventurers_respawn(
) -> impl FnMut(Query<Entity, (With<Adventurer>, With<Dead>)>, Res<Time>, ResMut<NextState<SceneState>>)
{
    let mut death_delay = DEATH_DELAY;

    return move |dead_mover_q, time, mut next_state| {
        for _ in dead_mover_q.iter() {
            death_delay -= time.delta_seconds();
            if death_delay <= 0.0 {
                death_delay = DEATH_DELAY;
                next_state.set(SceneState::Transitioning);
            }
        }
    };
}

// Helpers

fn deg_to_rad(deg: f32) -> f32 {
    return deg * PI / 180.0;
}
