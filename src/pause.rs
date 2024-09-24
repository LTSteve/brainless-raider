use bevy::prelude::*;

// Plugin

pub struct PausePlugin;
impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.insert_state(PauseState::Running)
            .add_systems(Update, on_pause.run_if(in_state(PauseState::Running)))
            .add_systems(Update, on_unpause.run_if(in_state(PauseState::Paused)));
    }
}

// States

#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PauseState {
    #[default]
    Running,
    Paused,
}

// Systems

pub fn on_pause(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<PauseState>>) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(PauseState::Paused);
    }
}

pub fn on_unpause(keys: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<PauseState>>) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(PauseState::Running);
    }
}
