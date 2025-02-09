//! Plays animations from a skinned glTF.

use std::f32::consts::PI;
use std::time::Duration;

use bevy::color::palettes;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::scene::SceneInstanceReady;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

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
            AnimationParams::new("all_animations_7.glb#Animation0", "TPose"),
            AnimationParams::new("all_animations_7.glb#Animation1", "ClimbDown"),
            AnimationParams::new("all_animations_7.glb#Animation2", "CrouchWalk"),
            AnimationParams::new("all_animations_7.glb#Animation3", "FallOpen"),
            AnimationParams::new("all_animations_7.glb#Animation4", "FallDiagonal"),
            AnimationParams::new("all_animations_7.glb#Animation5", "FallHeadDown"),
            AnimationParams::new("all_animations_7.glb#Animation6", "RunSprint"),
            AnimationParams::new("all_animations_7.glb#Animation7", "WallHang"),
            AnimationParams::new("all_animations_7.glb#Animation8", "IdleStand"),
            AnimationParams::new("all_animations_7.glb#Animation9", "DashPose"),
            AnimationParams::new("all_animations_7.glb#Animation10", "RunFast"),
            AnimationParams::new("all_animations_7.glb#Animation11", "RunJog"),
            AnimationParams::new("all_animations_7.glb#Animation12", "Walk"),
            AnimationParams::new("all_animations_7.glb#Animation13", "WalkStride"),
            AnimationParams::new("all_animations_7.glb#Animation14", "JumpAscent"),
            AnimationParams::new("all_animations_7.glb#Animation15", "LadderHandsWide"),
            AnimationParams::new("all_animations_7.glb#Animation16", "LadderHandsMedium"),
            AnimationParams::new("all_animations_7.glb#Animation17", "WallSlide"),
        ])
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(AssetPlugin { ..default() }),))
        .add_plugins(WorldInspectorPlugin::default())
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .insert_resource(AnimationsMetadata::new())
        .init_resource::<GizmosConfig>()
        .add_systems(Startup, setup)
        .add_systems(Startup, load_model_and_animations)
        .add_systems(Update, draw_gizmos.after(keyboard_animation_control))
        .add_systems(
            Update,
            keyboard_animation_control.run_if(resource_exists::<Animations>),
        )
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

fn setup(mut commands: Commands) {
    println!("--------- setup");

    commands.spawn((
        Camera3d::default(),
        OrthographicProjection {
            scale: 1.0,
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 1.0,
            },
            ..OrthographicProjection::default_3d()
        },
        Transform::from_xyz(0.0, 0.0, 12.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .build(),
    ));

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - return: change animation");
}

fn load_model_and_animations(
    mut commands: Commands,
    animation_meta: Res<AnimationsMetadata>,
    asset_server: Res<AssetServer>,
) {
    let anim_handles: Vec<Handle<AnimationClip>> = animation_meta
        .0
        .iter()
        .map(|params| asset_server.load(&params.path))
        .collect();
    commands.insert_resource(Animations(anim_handles));

    // Fox
    commands
        .spawn((
            SceneRoot(asset_server.load("mixamo_character_2.glb#Scene0")),
            Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
        ))
        .observe(setup_scene_once_loaded);
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
    animations: Res<Animations>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    let Ok(children) = children
        .get(trigger.entity())
        .and_then(|child| children.get(child[0]))
    else {
        unreachable!()
    };

    let animation_graph = AnimationGraph::from_clips(animations.0.iter().cloned());
    let handle = animation_graphs.add(animation_graph.0);

    for child in children {
        if let Ok((player, mut ani_player)) = animation_players.get_mut(*child) {
            bevy::log::warn!("Adding AnimationTransitions and AnimationGraph");
            let mut transitions = AnimationTransitions::new();
            transitions.play(&mut ani_player, animation_graph.1[0], Duration::default());

            commands
                .entity(player)
                .insert((AnimationGraphHandle(handle.clone()), transitions));
        }
    }
}

#[derive(Debug, Default, Resource)]
struct GizmosConfig {
    y: bool,
    vel: f32,
}

fn draw_gizmos(mut gizmos: Gizmos, gizmos_config: Res<GizmosConfig>, time: Res<Time>) {
    gizmos.rect(
        Isometry3d::new(
            Vec3::Y * (1.7 / 2.0) + Vec3::Z * time.elapsed_secs().sin(),
            Quat::from_rotation_y(0.0),
        ),
        Vec2::new(0.6, 1.7),
        palettes::basic::GREEN,
    );

    let num_lines = 30;
    for i in 0..num_lines {
        let t = time.elapsed_secs();
        let mut x = -t * gizmos_config.vel + i as f32;

        x = x % num_lines as f32 - (num_lines as f32 / 2.0) * x.signum();

        let (v, end) = if gizmos_config.y {
            (Vec3::Y * x, Vec3::X)
        } else {
            (Vec3::X * x, Vec3::NEG_Y)
        };

        gizmos.ray(v, end, palettes::css::BISQUE)
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    animations: Res<Animations>,
    animation_meta: Res<AnimationsMetadata>,
    mut gizmos_config: ResMut<GizmosConfig>,
    //locals
    mut current_animation: Local<usize>,
    mut use_params: Local<bool>,
) {
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        gizmos_config.y = !gizmos_config.y;
    }

    for (mut player, mut transitions) in &mut animation_players {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.all_paused() {
                player.resume_all();
            } else {
                player.pause_all();
            }
        }

        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            gizmos_config.vel += 0.1;
            println!(
                "playback speed: {},   vel: {}",
                player.playing_animations().next().unwrap().1.speed(),
                gizmos_config.vel
            );
        }

        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            gizmos_config.vel -= 0.1;
            println!(
                "playback speed: {},   vel: {}",
                player.playing_animations().next().unwrap().1.speed(),
                gizmos_config.vel
            );
        }

        if keyboard_input.just_pressed(KeyCode::ControlLeft) {
            *use_params = !*use_params;
        }

        if *use_params {
            let anim_params = &animation_meta.0[*current_animation];
            let speed = anim_params.playback_speed;
            player.adjust_speeds(speed);
        } else {
            if keyboard_input.just_pressed(KeyCode::KeyA) {
                let speed = player.playing_animations().next().unwrap().1.speed();
                player.adjust_speeds(speed + 0.1);
                println!(
                    "playback speed: {},   vel: {}",
                    player.playing_animations().next().unwrap().1.speed(),
                    gizmos_config.vel
                );
            }

            if keyboard_input.just_pressed(KeyCode::KeyZ) {
                let speed = player.playing_animations().next().unwrap().1.speed();
                player.adjust_speeds(speed - 0.1);
                println!(
                    "playback speed: {},   vel: {}",
                    player.playing_animations().next().unwrap().1.speed(),
                    gizmos_config.vel
                );
            }
        }

        if keyboard_input.just_pressed(KeyCode::ControlLeft) {
            println!(
                "TOGGLED PARAMS {} playback speed: {},   vel: {}",
                *use_params,
                player.playing_animations().next().unwrap().1.speed(),
                gizmos_config.vel
            );
        }

        if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
            for (_, playing) in player.playing_animations_mut() {
                let elapsed = playing.elapsed();
                playing.seek_to(elapsed - 0.1);
            }
        }

        if keyboard_input.just_pressed(KeyCode::AltRight) {
            for (_, playing) in player.playing_animations_mut() {
                let elapsed = playing.elapsed();
                playing.seek_to(elapsed + 0.1);
            }
        }

        if keyboard_input.just_pressed(KeyCode::Enter) {
            *current_animation = (*current_animation + 1) % animations.0.len();
            transitions
                .play(
                    &mut player,
                    (*current_animation as u32).into(),
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
