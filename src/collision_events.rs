use std::f32::consts::PI;

use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};

// Constants

// Plugin
pub struct CollisionEventsPlugin;
impl Plugin for CollisionEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (on_adventurer_goblinoid_collide));
    }
}

// Systems

pub fn on_adventurer_goblinoid_collide(
    mut commands: Commands,
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    collision_q: Query<(Entity, &Collider)>,
    mut transform_q: Query<(&mut Transform, &mut Sprite)>,
) {
    for e in ev_collision_enter.read() {
        if let [Ok(bundle_1), Ok(bundle_2)] = [collision_q.get(e.0), collision_q.get(e.1)] {
            if let [Some(goblinoid_entity), Some(_)] =
                collision_between("Goblinoid", "Adventurer", bundle_1, bundle_2)
            {
                commands.entity(goblinoid_entity).remove::<Mover>();
                commands.entity(goblinoid_entity).remove::<Collider>();
                if let Ok((mut transform, mut sprite)) = transform_q.get_mut(goblinoid_entity) {
                    transform.rotate_z(deg_to_rad(90.0));
                    transform.scale = transform.scale * 0.8;
                    transform.translation.z -= 3.0; // TODO: this is odd
                    sprite.color = Color::GRAY;
                }
            }
        }
    }
}

// Helpers
fn deg_to_rad(deg: f32) -> f32 {
    return deg * PI / 180.0;
}

fn collision_between(
    name_a: &str,
    name_b: &str,
    bundle_1: (Entity, &Collider),
    bundle_2: (Entity, &Collider),
) -> [Option<Entity>; 2] {
    let mut entity_a: Option<Entity> = None;
    let mut entity_b: Option<Entity> = None;

    if bundle_1.1.name == name_a {
        entity_a = Some(bundle_1.0);
    }
    if bundle_2.1.name == name_a {
        entity_a = Some(bundle_2.0);
    }
    if bundle_1.1.name == name_b {
        entity_b = Some(bundle_1.0);
    }
    if bundle_2.1.name == name_b {
        entity_b = Some(bundle_2.0);
    }

    return [entity_a, entity_b];
}
