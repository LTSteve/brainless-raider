// Constants
pub const FLOOR_Z: f32 = 0.0;
pub const ENTITY_Z_OFFSET: f32 = 10.0;
pub const SCALE: f32 = 4.0;

pub const TILE_WIDTH: f32 = 16.0;
pub const HALF_TILE_WIDTH: f32 = 8.0;
pub const MAP_WIDTH_COORD: f32 = 30.0;
pub const HALF_MAP_WIDTH_COORD: f32 = 15.0;
pub const MAP_WIDTH: f32 = MAP_WIDTH_COORD * TILE_WIDTH;
pub const HALF_MAP_WIDTH: f32 = HALF_MAP_WIDTH_COORD * TILE_WIDTH;

// Helpers

pub fn clamp(val: f32, min: f32, max: f32) -> f32 {
    if val > max {
        return max;
    }
    if val < min {
        return min;
    }
    return val;
}

pub fn coord_to_pos(val: f32) -> f32 {
    return (val * TILE_WIDTH - HALF_MAP_WIDTH + HALF_TILE_WIDTH) * SCALE;
}

pub fn pos_to_coord(val: f32) -> f32 {
    return ((val / SCALE) + HALF_MAP_WIDTH - HALF_TILE_WIDTH) / TILE_WIDTH;
}
