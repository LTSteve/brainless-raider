use crate::*;
use bevy::{ecs::system::EntityCommands, prelude::*};
use std::f32::consts::PI;

// Constants

const RIGHT: IVec2 = IVec2::new(1, 0);
const LEFT: IVec2 = IVec2::new(-1, 0);
const UP: IVec2 = IVec2::new(0, 1);
const DOWN: IVec2 = IVec2::new(0, -1);

const MOVER_SPEED: f32 = 1.95;

// Plugin

pub struct MovementPlugin;
impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_hydrators).add_systems(
            Update,
            move_movers
                .run_if(in_state(MapLoadState::Done))
                .run_if(in_state(PauseState::Running)),
        );
    }
}

fn add_hydrators(mut hydrators: ResMut<ComponentHydrators>) {
    hydrators.register_hydrator("Mover", hydrate_mover);
}

// Components

#[derive(Debug, Component)]
pub struct Mover {
    pub dir: IVec2,
    pub target: IVec2,
    pub coord: IVec2,
    pub move_percent: f32,
    pub clockwise: bool,
}

pub fn hydrate_mover(entity_commands: &mut EntityCommands, object_data: &ObjectData, _: &World) {
    let x = get_property_value_from_object_or_default_i(object_data, "dir_x", 0);
    let y = get_property_value_from_object_or_default_i(object_data, "dir_y", 0);
    let clockwise = get_property_value_from_object_or_default_b(object_data, "clockwise", false);

    entity_commands.insert((
        Mover {
            dir: IVec2::new(x as i32, y as i32),
            target: IVec2::new(
                object_data.x as i32 + x as i32,
                object_data.y as i32 + y as i32,
            ),
            coord: IVec2::new(object_data.x as i32, object_data.y as i32),
            move_percent: 0.0,
            clockwise,
        },
        OverPlanksCounter(0),
        OverPitCounter(0),
    ));
}

// Systems

pub fn move_movers(
    mut movers: Query<(&mut Transform, &mut Mover)>,
    time: Res<Time>,
    map_server: Res<MapServer>,
) {
    let active_map = &map_server.maps[map_server.map_idx];

    for (mut transform, mut mover) in movers.iter_mut() {
        mover.move_percent = clamp(
            mover.move_percent + MOVER_SPEED * time.delta_seconds(),
            0.0,
            1.0,
        );
        let destination = Vec3::new(
            coord_to_pos(mover.target.x as f32),
            coord_to_pos(mover.target.y as f32),
            transform.translation.z,
        );
        let previous_position = Vec3::new(
            coord_to_pos(mover.coord.x as f32),
            coord_to_pos(mover.coord.y as f32),
            transform.translation.z,
        );
        transform.translation = cerp_v3(previous_position, destination, mover.move_percent);

        if mover.move_percent == 1.0 {
            mover.move_percent = 0.0;
            mover.coord = mover.target;

            let forward = mover.dir;
            let side = rotate_dir(mover.dir, mover.clockwise);
            let back = -mover.dir;
            let target: IVec2;

            if tile_data_from_coord(mover.coord + forward, active_map) == 1 {
                target = mover.coord + forward;
            } else if tile_data_from_coord(mover.coord + side, active_map) == 1 {
                target = mover.coord + side;
                mover.dir = side;
            } else if tile_data_from_coord(mover.coord + back, active_map) == 1 {
                target = mover.coord + back;
                mover.dir = back;
            } else {
                target = mover.coord;
                mover.dir = IVec2::ZERO;
            }

            mover.target = target;
        }
    }
}

// Helpers

fn rotate_dir(dir: IVec2, cw: bool) -> IVec2 {
    if dir == RIGHT {
        return if cw { DOWN } else { UP };
    } else if dir == DOWN {
        return if cw { LEFT } else { RIGHT };
    } else if dir == LEFT {
        return if cw { UP } else { DOWN };
    } else if dir == UP {
        return if cw { RIGHT } else { LEFT };
    }
    return IVec2::ZERO;
}

fn cerp_v3(start: Vec3, end: Vec3, percent: f32) -> Vec3 {
    return lerp_v3(start, end, (percent * PI * 0.5).sin());
}

fn lerp_v3(start: Vec3, end: Vec3, percent: f32) -> Vec3 {
    return start * (1.0 - percent) + end * percent;
}
