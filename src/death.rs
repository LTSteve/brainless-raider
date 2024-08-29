use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};
use std::f32::consts::PI;

// Constants

pub const DEATH_OFFSET: f32 = 2.0; // Under treasure
pub const DEATH_SCALE: f32 = 0.8;
pub const DEATH_COLOR: Color = Color::GRAY;
pub const DEATH_ROTATION: f32 = 90.0;

// Plugin

pub struct DeathPlugin;
impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, movers_die);
    }
}

// Components

#[derive(Debug, Component)]
pub struct Dead;

// Systems

fn movers_die(
    mut dead_mover_q: Query<
        (Entity, &mut Transform, &mut Sprite, &mut Collider),
        (With<Dead>, With<Mover>),
    >,
    treasure_train_q: Query<(Entity, &TreasureTrain)>,
    mut treasure_collider_q: Query<&mut Collider, (With<Treasure>, Without<Mover>)>,
    mut commands: Commands,
) {
    for (mover_entity, mut transform, mut sprite, mut collider) in dead_mover_q.iter_mut() {
        for (entity, treasure_train) in treasure_train_q.iter() {
            if treasure_train.mover == mover_entity {
                // re enable colliders for all treasures on the train
                for &treasure_entity in &treasure_train.treasures {
                    if let Ok(mut treasure_collider) = treasure_collider_q.get_mut(treasure_entity)
                    {
                        treasure_collider.active = true;
                    }
                }
                // despawn the treasure train
                commands.entity(entity).despawn();
                break;
            }
        }
        commands
            .entity(mover_entity)
            .remove::<Mover>()
            .remove::<Dead>();
        collider.active = false;
        transform.rotate_z(deg_to_rad(DEATH_ROTATION));
        transform.scale = transform.scale * DEATH_SCALE;
        transform.translation.z -= DEATH_OFFSET;
        sprite.color = DEATH_COLOR;
    }
}

// Helpers

fn deg_to_rad(deg: f32) -> f32 {
    return deg * PI / 180.0;
}
