use std::f32::consts::PI;

use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};

// Constants

pub const GOBLINOID_DEATH_OFFSET: f32 = 1.0;
pub const GOBLINOID_DEATH_SCALE: f32 = 0.8;
pub const GOBLINOID_DEATH_COLOR: Color = Color::GRAY;
pub const GOBLINOID_DEATH_ROTATION: f32 = 90.0;

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
    adventurer_q: Query<Entity, With<Adventurer>>,
    mut goblinoid_q: Query<(Entity, &mut Transform, &mut Sprite), With<Goblinoid>>,
) {
    for e in ev_collision_enter.read() {
        if let (Ok(_), Ok((goblinoid_entity, mut transform, mut sprite))) =
            (adventurer_q.get(e.0), goblinoid_q.get_mut(e.1))
        {
            commands.entity(goblinoid_entity).remove::<Mover>();
            commands.entity(goblinoid_entity).remove::<Collider>();
            transform.rotate_z(deg_to_rad(GOBLINOID_DEATH_ROTATION));
            transform.scale = transform.scale * GOBLINOID_DEATH_SCALE;
            transform.translation.z -= GOBLINOID_DEATH_OFFSET;
            sprite.color = GOBLINOID_DEATH_COLOR;
        }
    }
}

// Helpers
fn deg_to_rad(deg: f32) -> f32 {
    return deg * PI / 180.0;
}
