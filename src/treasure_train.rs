use bevy::prelude::*;

use crate::{coord_to_pos, Mover};

// Plugin
pub struct TreasureTrainPlugin;
impl Plugin for TreasureTrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_treasure_trains));
    }
}

// Components
#[derive(Debug, Component)]
pub struct Treasure {
    pub following: Option<Entity>,
}
impl Default for Treasure {
    fn default() -> Self {
        Self { following: None }
    }
}

#[derive(Debug, Component)]
pub struct TreasureTrain {
    pub mover: Entity,
    pub treasures: Vec<Entity>,
    pub head_spot: IVec2,
    pub target_spots: Vec<IVec2>,
}

// Systems

fn update_treasure_trains(
    mut treasure_train_q: Query<&mut TreasureTrain>,
    mut treasure_q: Query<&mut Transform, With<Treasure>>,
    mover_q: Query<&Mover>,
) {
    for mut treasure_train in treasure_train_q.iter_mut() {
        // fill target spots
        while treasure_train.treasures.len() > treasure_train.target_spots.len() {
            let last_coord = treasure_train.target_spots.last().unwrap().clone();
            treasure_train.target_spots.push(last_coord);
        }

        // update target spots
        if let Ok(mover) = mover_q.get(treasure_train.mover) {
            if mover.target != treasure_train.head_spot {
                for i in (0..treasure_train.target_spots.len()).rev() {
                    if i == 0 {
                        treasure_train.target_spots[i] = treasure_train.head_spot;
                    } else {
                        treasure_train.target_spots[i] = treasure_train.target_spots[i - 1];
                    }
                }
                treasure_train.head_spot = mover.target;
            }
        }

        // move treasures
        let mut i = 0;
        for &treasure_entity in &treasure_train.treasures {
            if let Ok(mut treasure_transform) = treasure_q.get_mut(treasure_entity) {
                let coord = treasure_train.target_spots[i];
                let mvmt = Vec3::new(
                    coord_to_pos(coord.x as f32),
                    coord_to_pos(coord.y as f32),
                    treasure_transform.translation.z,
                ) - treasure_transform.translation;
                treasure_transform.translation = treasure_transform.translation + mvmt * 0.1;
            }
            i += 1;
        }
    }
}
