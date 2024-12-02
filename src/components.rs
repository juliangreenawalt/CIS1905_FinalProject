use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
}

#[derive(Component)]
pub struct BeatMap {
    pub name: String,
    pub difficulty: String,
    pub bpm: u32,
    pub njs: u32,
    pub hp: u32,
    pub song_file: String,
    pub hit_objects: Vec<HitObject>,
}

#[derive(Component)]
pub struct HitObject {
    pub beat: f32,
    pub lane: usize,
    pub duration: Option<f32>,
}

#[derive(Component)]
pub struct ScoreDisplay;

#[derive(Component)]
pub struct ActiveNote {
    pub beat: f32, // When this note should be hit
    pub lane: usize,
    pub njs: u32,
}
