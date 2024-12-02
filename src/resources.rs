use bevy::prelude::*;
use std::collections::HashMap;
use bevy::time::Timer;

#[derive(Default, Resource)]
pub struct ActiveHighlights {
    pub highlights: HashMap<usize, Entity>, // Lane index to entity mapping
}

#[derive(Resource, Default)]
pub struct Score {
    pub total: u32,
}

#[derive(Resource)]
pub struct BeatMapPlayback {
    pub elapsed: Timer,
    pub duration: f32,
    pub started: bool,
    pub song_started: bool,
}

