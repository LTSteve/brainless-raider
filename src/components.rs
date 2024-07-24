use bevy::{ecs::system::EntityCommands, prelude::*};
use roxmltree::*;

use crate::find_property_by_name_or_template;

#[derive(Debug, Component)]
pub struct Mover {
    pub dir: IVec2,
}
pub fn mover_hydrator(commands: &mut EntityCommands, data: String, template_data: String) {
    let document = Document::parse(&data).expect("can't parse hydrator data");
    let template_document =
        Document::parse(&template_data).expect("can't parse hydrator template data");
    commands.insert(Mover {
        dir: IVec2 {
            x: find_property_by_name_or_template(&document, &template_document, "x", 0),
            y: find_property_by_name_or_template(&document, &template_document, "y", 0),
        },
    });
}
