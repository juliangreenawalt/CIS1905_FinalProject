pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

//use events::*;
use resources::*;
use systems::*;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1280.0, 720.0).into(),
                title: "Rusto".to_string(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .init_resource::<ActiveHighlights>()
        .init_resource::<Score>()
        .insert_resource(BeatMapPlayback {
            elapsed: Timer::from_seconds(0.0, TimerMode::Once), //Timer duration and duration need to match!!!
            duration: 0.0,
            started: false,
            song_started: false,
        })
        .add_systems(Startup, load_beatmaps)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, setup_score_ui)
        .add_systems(Update, player_input)
        .add_systems(Update, play_beatmap)
        .add_systems(Update, update_score_ui)
        .add_systems(FixedUpdate, update_notes)
        .run();
}