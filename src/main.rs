//! Plays animations from a skinned glTF.

use std::f32::consts::PI;
use std::time::Duration;

use bevy::asset::ChangeWatcher;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

#[derive(Default, Debug)]
pub struct AnimationParams {
    pub path: String,
    pub name: String,
    pub playback_speed: f32,
}

impl AnimationParams {
    pub fn new(path: &str, name: &str) -> Self {
        Self {
            path: path.to_string(),
            name: name.to_string(),
            playback_speed: 1.0,
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct AnimationsMetadata(pub Vec<AnimationParams>);

impl AnimationsMetadata {
    pub fn new() -> Self {
        AnimationsMetadata(vec![
            AnimationParams::new("all_animations_6.glb#Animation0", "TPose"),
            AnimationParams::new("all_animations_6.glb#Animation1", "ClimbDown"),
            AnimationParams::new("all_animations_6.glb#Animation2", "CrouchWalk"),
            AnimationParams::new("all_animations_6.glb#Animation3", "FallOpen"),
            AnimationParams::new("all_animations_6.glb#Animation4", "FallDiagonal"),
            AnimationParams::new("all_animations_6.glb#Animation5", "FallHeadDown"),
            AnimationParams::new("all_animations_6.glb#Animation6", "RunSprint"),
            AnimationParams::new("all_animations_6.glb#Animation7", "WallHang"),
            AnimationParams::new("all_animations_6.glb#Animation8", "IdleStand"),
            AnimationParams::new("all_animations_6.glb#Animation9", "DashPose"),
            AnimationParams::new("all_animations_6.glb#Animation10", "RunFast"),
            AnimationParams::new("all_animations_6.glb#Animation11", "RunJog"),
            AnimationParams::new("all_animations_6.glb#Animation12", "Walk"),
            AnimationParams::new("all_animations_6.glb#Animation13", "WalkStride"),
            AnimationParams::new("all_animations_6.glb#Animation14", "JumpAscent"),
            AnimationParams::new("all_animations_6.glb#Animation15", "LadderHandsWide"),
            AnimationParams::new("all_animations_6.glb#Animation16", "LadderHandsMedium"),
            AnimationParams::new("all_animations_6.glb#Animation17", "WallSlide"),
        ])
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_millis(100)),
            ..default()
        }),))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(AnimationsMetadata::new())
        // .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                setup.run_if(
                    resource_exists::<AnimationsMetadata>()
                        .and_then(not(resource_exists::<AnimationsLoadedMarker>())),
                ),
                setup_scene_once_loaded.run_if(resource_exists::<Animations>()),
                keyboard_animation_control.run_if(resource_exists::<Animations>()),
            ),
        )
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

#[derive(Resource)]
struct AnimationsLoadedMarker;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    animation_meta: Res<AnimationsMetadata>,
) {
    println!("--------- setup");

    let anim_handles: Vec<Handle<AnimationClip>> = animation_meta
        .0
        .iter()
        .map(|params| asset_server.load(&params.path))
        .collect();
    commands.insert_resource(Animations(anim_handles));

    commands.insert_resource(AnimationsLoadedMarker);

    commands.spawn(Camera3dBundle {
        projection: OrthographicProjection {
            scale: 4.0,
            scaling_mode: ScalingMode::FixedVertical(2.0),
            ..default()
        }
        .into(),
        transform: Transform::from_xyz(0.0, 0.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // Fox
    let mut trans = Transform::default();
    trans.rotate_axis(Vec3::Y, 3.14159 * 0.5);
    commands.spawn(SceneBundle {
        // scene: asset_server.load("mixamo_character_1.glb#Scene0"),
        scene: asset_server.load("mixamo_character_2.glb#Scene0"),
        transform: trans,
        ..default()
    });

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - return: change animation");
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak()).repeat();
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    animation_meta: Res<AnimationsMetadata>,

    mut gizmos: Gizmos,
    time: Res<Time>,
    //locals
    mut current_animation: Local<usize>,
    mut vel: Local<f32>,
    mut gizmos_y: Local<bool>,
    mut use_params: Local<bool>,
) {
    gizmos.rect(
        Vec3::Y * (1.7 / 2.0) + Vec3::Z * time.elapsed_seconds().sin(),
        Quat::from_rotation_y(0.0),
        Vec2::new(0.6, 1.7),
        Color::GREEN,
    );
    let num_lines = 30;
    for i in 0..num_lines {
        let t = time.elapsed_seconds();
        let mut x = -t * *vel + i as f32;

        x = x % num_lines as f32 - (num_lines as f32 / 2.0) * x.signum();

        let (v, end) = if *gizmos_y {
            (Vec3::Y * x, Vec3::X)
        } else {
            (Vec3::X * x, Vec3::NEG_Y)
        };

        gizmos.ray(v, end, Color::BISQUE)
    }

    if keyboard_input.just_pressed(KeyCode::Back) {
        *gizmos_y = !*gizmos_y;
    }

    for mut player in &mut animation_players {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
            } else {
                player.pause();
            }
        }

        if keyboard_input.just_pressed(KeyCode::Up) {
            *vel += 0.1;
            println!("playback speed: {},   vel: {}", player.speed(), *vel);
        }

        if keyboard_input.just_pressed(KeyCode::Down) {
            *vel -= 0.1;
            println!("playback speed: {},   vel: {}", player.speed(), *vel);
        }

        if keyboard_input.just_pressed(KeyCode::ControlLeft) {
            *use_params = !*use_params;
        }

        if *use_params {
            let anim_params = &animation_meta.0[*current_animation];
            let speed = anim_params.playback_speed;
            player.set_speed(speed);
        } else {
            if keyboard_input.just_pressed(KeyCode::A) {
                let speed = player.speed();
                player.set_speed(speed + 0.1);
                println!("playback speed: {},   vel: {}", player.speed(), *vel);
            }

            if keyboard_input.just_pressed(KeyCode::Z) {
                let speed = player.speed();
                player.set_speed(speed - 0.1);
                println!("playback speed: {},   vel: {}", player.speed(), *vel);
            }
        }

        if keyboard_input.just_pressed(KeyCode::ControlLeft) {
            println!(
                "TOGGLED PARAMS {} playback speed: {},   vel: {}",
                *use_params,
                player.speed(),
                *vel
            );
        }

        if keyboard_input.just_pressed(KeyCode::Left) {
            let elapsed = player.elapsed();
            player.set_elapsed(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Right) {
            let elapsed = player.elapsed();
            player.set_elapsed(elapsed + 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Return) {
            *current_animation = (*current_animation + 1) % animations.0.len();
            player
                .play_with_transition(
                    animations.0[*current_animation].clone_weak(),
                    Duration::from_millis(250),
                )
                .repeat();

            println!(
                "Playing animation: {}",
                animation_meta.0[*current_animation].name
            );
            println!("{:?}", animation_meta.0[*current_animation]);
        }
    }
}
