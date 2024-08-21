use bevy::prelude::*;

use crate::NoTearDown;

// Plugin

pub struct AudioServerPlugin;
impl Plugin for AudioServerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AudioFiles {
            click: "sfx/click.ogg",
            die: "sfx/die.ogg",
            dumbraider: "sfx/dumbraider.ogg",
            exit: "sfx/exit.ogg",
            kill: "sfx/kill.ogg",
            pick_up: "sfx/pick_up.ogg",
            portal: "sfx/portal.ogg",
        })
        .add_systems(Startup, load_audio);
    }
}

// Resources

#[derive(Resource)]
struct AudioFiles {
    click: &'static str,
    die: &'static str,
    dumbraider: &'static str,
    exit: &'static str,
    kill: &'static str,
    pick_up: &'static str,
    portal: &'static str,
}

#[derive(Resource)]
pub struct AudioServer {
    pub click: PlayableAudioSource,
    pub die: PlayableAudioSource,
    pub dumbraider: PlayableAudioSource,
    pub exit: PlayableAudioSource,
    pub kill: PlayableAudioSource,
    pub pick_up: PlayableAudioSource,
    pub portal: PlayableAudioSource,
}

pub struct PlayableAudioSource(Handle<AudioSource>);

impl PlayableAudioSource {
    pub fn create_one_shot(&self) -> (AudioBundle, NoTearDown) {
        (
            AudioBundle {
                source: self.0.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn, // hmm... TODO: can we save these audio bundles and re-use them rather than spawn/despawn
                    ..default()
                },
            },
            NoTearDown,
        )
    }
    pub fn create_loop(&self) -> (AudioBundle, NoTearDown) {
        (
            AudioBundle {
                source: self.0.clone(),
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Loop, // hmm... TODO: can we save these audio bundles and re-use them rather than spawn/despawn
                    ..default()
                },
            },
            NoTearDown,
        )
    }
}

// Systems

fn load_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_files: Res<AudioFiles>,
) {
    commands.insert_resource(AudioServer {
        click: PlayableAudioSource(asset_server.load(audio_files.click)),
        die: PlayableAudioSource(asset_server.load(audio_files.die)),
        dumbraider: PlayableAudioSource(asset_server.load(audio_files.dumbraider)),
        exit: PlayableAudioSource(asset_server.load(audio_files.exit)),
        kill: PlayableAudioSource(asset_server.load(audio_files.kill)),
        pick_up: PlayableAudioSource(asset_server.load(audio_files.pick_up)),
        portal: PlayableAudioSource(asset_server.load(audio_files.portal)),
    });
    commands.remove_resource::<AudioFiles>();
}
