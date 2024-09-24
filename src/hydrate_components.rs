use bevy::{ecs::system::EntityCommands, prelude::*};
use bevy_utils::HashMap;

use crate::ObjectData;

pub struct HydrateComponentsPlugin;
impl Plugin for HydrateComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ComponentHydrators::new());
    }
}

#[derive(Clone, Debug, Resource)]
pub struct ComponentHydrators {
    hydrators: HashMap<&'static str, fn(&mut EntityCommands, &ObjectData)>,
}

impl ComponentHydrators {
    pub fn new() -> Self {
        return Self {
            hydrators: HashMap::new(),
        };
    }

    pub fn register_hydrator(
        &mut self,
        component_name: &'static str,
        func: fn(&mut EntityCommands, &ObjectData),
    ) -> &mut Self {
        self.hydrators.insert(component_name, func);
        return self;
    }

    pub fn register_tag<T>(&mut self, component_name: &'static str) -> &mut Self
    where
        T: Default + Component,
    {
        self.hydrators.insert(component_name, hydrate_tag::<T>);
        return self;
    }

    pub fn hydrate_entity(
        &self,
        entity_commands: &mut EntityCommands,
        object_data: &ObjectData,
        component_name: &str,
    ) {
        match self.hydrators.iter().find(|kvp| kvp.0 == &component_name) {
            Some(kvp) => {
                kvp.1(entity_commands, object_data);
            }
            None => {
                println!(
                    "tried to hydrate component:{} with no hydrator",
                    component_name
                );
            }
        }
    }
}

fn hydrate_tag<T>(entity_commands: &mut EntityCommands, _: &ObjectData)
where
    T: Default + Component,
{
    entity_commands.insert(T::default());
}

pub fn get_property_value_from_object_or_default_s(
    object_data: &ObjectData,
    property_name: &str,
    default_value: String,
) -> String {
    let v = object_data
        .properties
        .iter()
        .find(|p| p.name == property_name);
    let v = if let Some(prop) = v {
        prop.value_s.clone()
    } else {
        default_value
    };

    return v;
}

pub fn get_property_value_from_object_or_default_i(
    object_data: &ObjectData,
    property_name: &str,
    default_value: i64,
) -> i64 {
    let v = object_data
        .properties
        .iter()
        .find(|p| p.name == property_name);
    let v = if let Some(prop) = v {
        prop.value_i
    } else {
        default_value
    };

    return v;
}

pub fn get_property_value_from_object_or_default_f(
    object_data: &ObjectData,
    property_name: &str,
    default_value: f64,
) -> f64 {
    let v = object_data
        .properties
        .iter()
        .find(|p| p.name == property_name);
    let v = if let Some(prop) = v {
        prop.value_f
    } else {
        default_value
    };

    return v;
}

pub fn get_property_value_from_object_or_default_c(
    object_data: &ObjectData,
    property_name: &str,
    default_value: Color,
) -> Color {
    let v = object_data
        .properties
        .iter()
        .find(|p| p.name == property_name);
    let v = if let Some(prop) = v {
        prop.value_c
    } else {
        default_value
    };

    return v;
}

pub fn get_property_value_from_object_or_default_b(
    object_data: &ObjectData,
    property_name: &str,
    default_value: bool,
) -> bool {
    let v = object_data
        .properties
        .iter()
        .find(|p| p.name == property_name);
    let v = if let Some(prop) = v {
        prop.value_b
    } else {
        default_value
    };

    return v;
}
