use bevy::ecs::system::EntityCommands;

use crate::*;

// Plugin

pub struct ClickableAreaPlugin {
    pub debug_clicks: bool,
}
impl Plugin for ClickableAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MouseClickEvent>().add_systems(
            Update,
            track_clickable_areas.run_if(in_state(MapLoadState::Done)),
        );
        if self.debug_clicks {
            app.add_systems(Update, log_clicks.run_if(in_state(MapLoadState::Done)));
        }
    }
}

// Events

#[derive(Event)]
pub struct MouseClickEvent(pub Entity);

// Components

#[derive(Debug, Component)]
pub struct ClickableArea {
    pub location: Vec2,
    pub radius_squared: f32,
    pub name: String,
}

pub fn hydrate_clickable_area(entity_commands: &mut EntityCommands, object_data: &ObjectData) {
    let radius = get_property_value_from_object_or_default_f(object_data, "radius", 0.5) as f32;
    entity_commands.insert(ClickableArea {
        location: Vec2::new(
            coord_to_pos(object_data.x as f32),
            coord_to_pos(object_data.y as f32),
        ),
        radius_squared: radius * radius * SCALE,
        name: object_data.obj_type.clone(),
    });
}

// Systems

fn log_clicks(
    mut ev_mouse_click: EventReader<MouseClickEvent>,
    clickable_area_q: Query<&ClickableArea>,
) {
    for e in ev_mouse_click.read() {
        if let Ok(clickable_area) = clickable_area_q.get(e.0) {
            println!("{:?} clicked!", clickable_area.name);
        }
    }
}

fn track_clickable_areas(
    mut ev_mouse_click: EventWriter<MouseClickEvent>,
    area_q: Query<(Entity, &ClickableArea)>,
    mut window_q: Query<&mut Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<ButtonInput<MouseButton>>,
) {
    if let Ok(mut window) = window_q.get_single_mut() {
        let mut hovering: Option<Entity> = None;
        if let Some(mouse_postion) = window.cursor_position() {
            if let Ok((camera, camera_transform)) = camera_q.get_single() {
                let mouse_ray = camera.viewport_to_world(camera_transform, mouse_postion);
                if let Some(mouse_ray) = mouse_ray {
                    for (area_entity, area) in area_q.iter() {
                        if mouse_ray.origin.truncate().distance_squared(area.location)
                            <= area.radius_squared
                        {
                            hovering = Some(area_entity);
                        }
                    }
                }
            }
        }
        match hovering {
            Some(entity) => {
                window.cursor.icon = CursorIcon::Pointer;
                if buttons.just_pressed(MouseButton::Left) {
                    ev_mouse_click.send(MouseClickEvent(entity));
                }
            }
            None => {
                window.cursor.icon = CursorIcon::Default;
            }
        }
    }
}
