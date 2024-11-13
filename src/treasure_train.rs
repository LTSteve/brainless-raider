use bevy::prelude::*;
use rand::Rng;

use crate::{
    coord_to_pos, Adventurer, Mover, SceneState, TreasuresLabel, SUCCESS_COLOR, TEXT_COLOR,
};

// Constants

const TREASURE_SPEED: f32 = 2.0;
const TREASURE_MIN_ROT: f32 = 0.5;
const TREASURE_MAX_ROT: f32 = 2.0;

// Plugin
pub struct TreasureTrainPlugin;
impl Plugin for TreasureTrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_treasure_trains, update_treasure_count))
            .add_systems(OnEnter(SceneState::Stable), count_total_treasures)
            .insert_resource(TreasureCount {
                player_treasures: 0,
                map_treasures: 0,
            });
    }
}

// Resources

#[derive(Debug, Resource)]
pub struct TreasureCount {
    pub player_treasures: u16,
    pub map_treasures: u16,
}

// Components

#[derive(Debug, Component)]
pub struct Treasure {
    pub rot_speed: f32,
}
impl Default for Treasure {
    fn default() -> Self {
        Self {
            rot_speed: rand::thread_rng().gen_range(TREASURE_MIN_ROT..TREASURE_MAX_ROT),
        }
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
    mut treasure_q: Query<(&mut Transform, &Treasure)>,
    mover_q: Query<&Mover>,
    time: Res<Time>,
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

        // move & spin treasures
        let mut i = 0;
        for &treasure_entity in &treasure_train.treasures {
            if let Ok((mut treasure_transform, treasure)) = treasure_q.get_mut(treasure_entity) {
                let coord = treasure_train.target_spots[i];
                let mvmt = Vec3::new(
                    coord_to_pos(coord.x as f32),
                    coord_to_pos(coord.y as f32),
                    treasure_transform.translation.z,
                ) - treasure_transform.translation;
                treasure_transform.translation =
                    treasure_transform.translation + mvmt * TREASURE_SPEED * time.delta_seconds();
                treasure_transform.rotate_z(treasure.rot_speed * time.delta_seconds());
            }
            i += 1;
        }
    }
}

fn update_treasure_count(
    mut treasure_count: ResMut<TreasureCount>,
    treasure_train_q: Query<&TreasureTrain>,
    adventurer_q: Query<Entity, With<Adventurer>>,
    mut treasure_label_q: Query<&mut Text, With<TreasuresLabel>>,
) {
    if let Ok(mut treasure_label) = treasure_label_q.get_single_mut() {
        let mut treasures_picked_up: u16 = 0;

        for treasure_train in treasure_train_q.iter() {
            if let Ok(_) = adventurer_q.get(treasure_train.mover) {
                treasures_picked_up = treasure_train.treasures.len() as u16;
            }
        }

        treasure_count.player_treasures = treasures_picked_up;
        treasure_label.sections[1].value = treasure_count.player_treasures.to_string();

        let text_color = if treasure_count.player_treasures == treasure_count.map_treasures {
            Color::hex(SUCCESS_COLOR).expect("invalid color hex")
        } else {
            Color::hex(TEXT_COLOR).expect("invalid color hex")
        };

        treasure_label.sections[0].style.color = text_color;
        treasure_label.sections[1].style.color = text_color;
    }
}

fn count_total_treasures(treasure_q: Query<&Treasure>, mut treasure_count: ResMut<TreasureCount>) {
    treasure_count.map_treasures = treasure_q.iter().count() as u16;
    treasure_count.player_treasures = 0;
}
