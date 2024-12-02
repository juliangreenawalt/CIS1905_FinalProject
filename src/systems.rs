use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;

use crate::components::*;
//use crate::events::*;
use crate::resources::*;

pub const PLAYER_SPEED: f32 = 500.0;
pub const PLAYER_SIZE: f32 = 16.0;

pub fn spawn_player(
    mut commands: Commands, 
    window_query: Query<&Window, With<PrimaryWindow>>, 
    asset_server: Res<AssetServer>,
) {
    let window = window_query.get_single().unwrap();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0 , 0.0),
            texture: asset_server.load("sprites/Skeleton.png"),
            ..default()
        },
        Player {}
    ));
}

//Initial function to set up the window and the background frame for each map
pub fn spawn_camera(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
) {
    commands.spawn(Camera2dBundle::default());

    //Set up background image
    let background_img = asset_server.load("../assets/sprites/frame.png");

    let lanes = [
        (1, Vec3::new(-242.0, -290.0, 1.0)), //Lane 1
        (2, Vec3::new(-80.0, -290.0, 1.0)),  //Lane 2
        (3, Vec3::new(80.0, -290.0, 1.0)),   //Lane 3
        (4, Vec3::new(242.0, -290.0, 1.0)),  //Lane 4
    ];

    //Spawn a sprite for the background
    commands.spawn(SpriteBundle {
        texture: background_img,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });

    for (_lanes, position) in lanes {
        let container = asset_server.load("sprites/notecontainer_unclicked.png");
        commands.spawn(SpriteBundle {
            texture: container,
            transform: Transform {
                translation: position,
                ..Default::default()
            },
            ..Default::default()
        });
    }
    
}

//setup function to read in all beatmaps files and create beatmap components
pub fn load_beatmaps(
    mut commands: Commands, 
    asset_server: Res<AssetServer>, 
) {
    let beatmap_dir = "assets/beatmaps";

    //Read beatmap files from the directory
    if let Ok(entries) = fs::read_dir(beatmap_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                //check for .map extension
                if path.extension().and_then(|s| s.to_str()) == Some("map") {
                    if let Ok(beatmap) = parse_beatmap(&path) {
                        println!("Loaded beatmap: {:?}", beatmap.name);

                        //Spawn an entity with the BeatMap component
                        commands.spawn(beatmap);
                    }
                }
            }
        }
    }
}

//Helper function to parse a single beatmap file
fn parse_beatmap(path: &Path) -> io::Result<BeatMap> {
    let mut name = String::new();
    let mut difficulty = String::new();
    let mut bpm = 0;
    let mut njs = 0;
    let mut hp = 0;
    let mut song_file = String::new();
    let mut hit_objects = Vec::new();

    if let Ok(lines) = read_lines(path) {
        for line in lines {
            if let Ok(entry) = line {
                if entry.starts_with('#') || entry.trim().is_empty() {
                    continue; //Skip comments and empty lines
                }

                if entry.starts_with("name:") {
                    name = entry["name:".len()..].trim().to_string();
                } else if entry.starts_with("difficulty:") {
                    difficulty = entry["difficulty:".len()..].trim().to_string();
                } else if entry.starts_with("bpm:") {
                    bpm = entry["bpm:".len()..].trim().parse().unwrap_or(0);
                } else if entry.starts_with("njs:") {
                    njs = entry["njs:".len()..].trim().parse().unwrap_or(0);
                } else if entry.starts_with("hp:") {
                    hp = entry["hp:".len()..].trim().parse().unwrap_or(0);
                } else if entry.starts_with("song_file:") {
                    song_file = format!(r"audio\songs\{}", entry["song_file:".len()..].trim());
                } else {
                    //Parse hit objects
                    let parts: Vec<&str> = entry.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let beat: f32 = parts[0].parse().unwrap();
                        let lane: usize = parts[1].parse().unwrap();
                        let duration = if parts.len() == 3 {
                            Some(parts[2].parse().unwrap())
                        } else {
                            None
                        };

                        hit_objects.push(HitObject {
                            beat,
                            lane,
                            duration,
                        });
                    }
                }
            }
        }
    }

    Ok(BeatMap {
        name,
        difficulty,
        bpm,
        njs,
        hp,
        song_file,
        hit_objects,
    })
}

//Helper function to read lines from a file
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn play_beatmap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut playback: ResMut<BeatMapPlayback>,
    query: Query<&BeatMap>,
) {
    if let Some(beatmap) = query.iter().next() {

        playback.elapsed.tick(time.delta());

        //Spawn notes if playback has not started
        if !playback.started {
            playback.started = true; //Ensure notes spawn only once

            for hit_object in &beatmap.hit_objects {
                let lane_x = match hit_object.lane {
                    1 => -242.0,
                    2 => -80.0,
                    3 => 80.0,
                    4 => 242.0,
                    _ => continue,
                };

                //Calculate the y spawn position based on bpm, njs, and beat
                let seconds_per_beat = 60.0 / beatmap.bpm as f32;
                let scroll_time = seconds_per_beat * (beatmap.njs as f32 * 50.0);
                let start_y = -292.0 + (scroll_time * playback.duration - 1.0) + scroll_time * (hit_object.beat + playback.duration);

                //Red for outside, green for inside
                let texture_handle: Handle<Image> = if hit_object.lane == 1 || hit_object.lane == 4 {
                    asset_server.load(r"sprites\outside_note.png")
                } else {
                    asset_server.load(r"sprites\inside_note.png")
                };

                //Spawn note
                commands.spawn((
                    SpriteBundle {
                        texture: texture_handle,
                        transform: Transform {
                            translation: Vec3::new(lane_x, start_y, 0.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ActiveNote {
                        beat: hit_object.beat,
                        lane: hit_object.lane,
                        njs: beatmap.njs,
                    },
                ));
            }
        }

        //If grace period is defined
        if playback.elapsed.finished() && !playback.song_started {
            playback.song_started = true;

            println!("Loading audio file: {}", &beatmap.song_file);

            let song_handle: Handle<AudioSource> = asset_server.load(&beatmap.song_file);
            commands.spawn(AudioSourceBundle {
                source: song_handle,
                ..Default::default()
            });
        }
    }
}


pub fn update_notes(
    mut query: Query<(&mut Transform, &ActiveNote)>,
    time: Res<Time>,
    playback: Res<BeatMapPlayback>,
) {
    if playback.started {
        for (mut transform, note) in query.iter_mut() {
            transform.translation.y -= 50.0 * (note.njs as f32) * time.delta_seconds();
        }
    }
}


//Read player input and determine how accurate keypress was
pub fn player_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut active_highlights: ResMut<ActiveHighlights>,
    mut score: ResMut<Score>,
    notes_query: Query<(Entity, &Transform, &ActiveNote)>,
) {
    let key_to_lane = [
        (KeyCode::KeyD, 1, Vec3::new(-242.0, -290.0, 1.0)), //Lane 1
        (KeyCode::KeyF, 2, Vec3::new(-80.0, -290.0, 1.0)),  //Lane 2
        (KeyCode::KeyJ, 3, Vec3::new(80.0, -290.0, 1.0)),   //Lane 3
        (KeyCode::KeyK, 4, Vec3::new(242.0, -290.0, 1.0)),  //Lane 4
    ];

    for (key, lane_index, position) in key_to_lane {
        //Handle key press
        if keyboard_input.just_pressed(key) {
            //Highlight lane
            if !active_highlights.highlights.contains_key(&lane_index) {
                let highlight_texture = asset_server.load("sprites/notecontainer_clicked.png");
                let entity = commands.spawn(SpriteBundle {
                    texture: highlight_texture,
                    transform: Transform {
                        translation: position,
                        ..Default::default()
                    },
                    ..Default::default()
                }).id();
                active_highlights.highlights.insert(lane_index, entity);
            }

            //Check for notes in the lane
            for (entity, transform, note) in notes_query.iter() {
                if note.lane == lane_index &&
                   transform.translation.y >= -452.0 && //Within hit container
                   transform.translation.y <= -162.0
                {
                    //Process the note
                    let acc = (transform.translation.y + 290.0).abs();
                    let (feedback, points) = if acc <= 20.0 {
                        ("Excellent", 100)
                    } else if acc <= 40.0 {
                        ("Great", 70)
                    } else if acc <= 60.0 {
                        ("Good", 50)
                    } else {
                        ("Bad", 0)
                    };

                    score.total += points;

                    println!("{} ({:.2})", feedback, acc);
                    println!("Score: {}", score.total);
                    commands.entity(entity).despawn();
                    break;
                }
            }
        }

        // Handle key release
        if keyboard_input.just_released(key) {
            if let Some(entity) = active_highlights.highlights.remove(&lane_index) {
                commands.entity(entity).despawn();
            }
        }
    }
}

//Spawn and Update UI
pub fn setup_score_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "Score: 0",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::WHITE,
                },
            )
            .with_justify(JustifyText::Right),
            ..Default::default()
        },
        ScoreDisplay,
    ));
}

pub fn update_score_ui(score: Res<Score>, mut query: Query<&mut Text, With<ScoreDisplay>>) {
    for mut text in query.iter_mut() {
        text.sections[0].value = format!("Score: {}", score.total);
    }
}


