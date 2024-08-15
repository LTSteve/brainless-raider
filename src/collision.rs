use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};

// Constants

// Plugin
pub struct CollisionPlugin {
    pub debug_collisions: bool,
}
impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEnterEvent>()
            .add_event::<CollisionExitEvent>()
            .add_systems(Update, update_colliders);
        if self.debug_collisions {
            app.add_systems(Update, (debug_collision_exit, debug_collision_enter));
        }
    }
}

// Events

#[derive(Event)]
pub struct CollisionEnterEvent(pub Entity, pub Entity);

#[derive(Event)]
pub struct CollisionExitEvent(pub Entity, pub Entity);

// Components

#[derive(Debug, Component)]
pub struct Collider {
    pub radius: f32,
    pub name: String,
    pub colliding_with: Vec<Entity>,
}

pub fn hydrate_collider(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let radius = get_property_value_from_object_or_default_f(object_data, "collider_radius", 4.0);

    entity_commands.insert(Collider {
        radius: radius as f32,
        name: object_data.obj_type.clone(),
        colliding_with: Vec::new(),
    });
}

// Systems

pub fn update_colliders(
    mut ev_collision_enter: EventWriter<CollisionEnterEvent>,
    mut ev_collision_exit: EventWriter<CollisionExitEvent>,
    mut colliders: Query<(Entity, &Transform, &mut Collider)>,
) {
    let mut combinations = colliders.iter_combinations_mut::<2>();
    while let Some([(entity1, transform1, mut collider1), (entity2, transform2, mut collider2)]) =
        combinations.fetch_next()
    {
        let dist: f32 = collider1.radius + collider2.radius;
        let dist2: f32 = dist * dist;
        if transform1
            .translation
            .distance_squared(transform2.translation)
            <= dist2
        {
            let mut new_collision = false;
            if !collider1.colliding_with.contains(&entity2) {
                collider1.colliding_with.push(entity2);
                new_collision = true;
            }

            if !collider2.colliding_with.contains(&entity1) {
                collider2.colliding_with.push(entity1);
                new_collision = true;
            }

            if new_collision {
                ev_collision_enter.send(CollisionEnterEvent(entity1, entity2));
            }
        } else {
            let pos1 = collider1.colliding_with.iter().position(|e| e.eq(&entity2));
            let pos2 = collider2.colliding_with.iter().position(|e| e.eq(&entity1));

            let mut new_uncollision = false;
            if let Some(pos1) = pos1 {
                collider1.colliding_with.remove(pos1);
                new_uncollision = true;
            }

            if let Some(pos2) = pos2 {
                collider2.colliding_with.remove(pos2);
                new_uncollision = true;
            }

            if new_uncollision {
                ev_collision_exit.send(CollisionExitEvent(entity1, entity2));
            }
        }
    }
}

pub fn debug_collision_enter(
    mut ev_collision_enter: EventReader<CollisionEnterEvent>,
    colliders: Query<&Collider>,
) {
    for e in ev_collision_enter.read() {
        if let [Ok(collider1), Ok(collider2)] = [colliders.get(e.0), colliders.get(e.1)] {
            println!("{:?} entering {:?}", collider1.name, collider2.name);
        }
    }
}

pub fn debug_collision_exit(
    mut ev_collision_exit: EventReader<CollisionExitEvent>,
    colliders: Query<&Collider>,
) {
    for e in ev_collision_exit.read() {
        if let [Ok(collider1), Ok(collider2)] = [colliders.get(e.0), colliders.get(e.1)] {
            println!("{:?} exiting {:?}", collider1.name, collider2.name);
        }
    }
}

// Helpers
