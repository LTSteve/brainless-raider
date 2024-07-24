use bevy::ecs::system::EntityCommands;
use bevy_utils::HashMap;

use crate::ObjectData;

#[derive(Clone)]
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
        mut self,
        component_name: &'static str,
        func: fn(&mut EntityCommands, &ObjectData),
    ) -> Self {
        self.hydrators.insert(component_name, func);
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
