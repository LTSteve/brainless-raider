use bevy::ecs::system::EntityCommands;

use crate::*;

// Plugin

pub struct ClickableAreaPlugin;
impl Plugin for ClickableAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            track_clickable_areas.run_if(in_state(MapLoadState::Done)),
        );
    }
}

// Components

#[derive(Debug, Component)]
pub struct ClickableArea {
    pub location: Vec2,
    pub radius_squared: f32,
}

pub fn hydrate_clickable_area(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let radius = get_property_value_from_object_or_default_f(object_data, "radius", 0.5) as f32;
    entity_commands.insert(ClickableArea {
        location: Vec2::new(
            coord_to_pos(object_data.x as f32),
            coord_to_pos(object_data.y as f32),
        ),
        radius_squared: radius * radius * SCALE,
    });
}

// Systems

fn track_clickable_areas(
    area_q: Query<&ClickableArea>,
    mut window_q: Query<&mut Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    if let Ok(mut window) = window_q.get_single_mut() {
        let mut hovering = false;
        if let Some(mouse_postion) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_q.get_single() {
                let mouse_ray = camera.viewport_to_world(camera_transform, mouse_postion);
                if let Some(mouse_ray) = mouse_ray {
                    for area in area_q.iter() {
                        if mouse_ray.origin.truncate().distance_squared(area.location)
                            <= area.radius_squared
                        {
                            hovering = true;
                        }
                    }
                }
            }
        }
        if hovering {
            window.cursor.icon = CursorIcon::Pointer;
        } else {
            window.cursor.icon = CursorIcon::Default;
        }
    }
}
