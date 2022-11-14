use bevy::{prelude::*, window::PresentMode};
use bevy::input::mouse::MouseMotion;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};
use bevy_rapier3d::prelude::*;
fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Rust is the future of programming!".to_string(),
            width: 1920.,
            height: 1080.,
            present_mode: PresentMode::AutoVsync,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_startup_system(setup)
        .add_startup_system(setup_physics)
        .add_system(move_camera_system)
        .add_system(doing_the_wave)
        .add_system(controls)
        .add_system(cursor_grab_system)
        .run();
}
// struct that indicates that item will move as sin wave
#[derive(Component)]
struct Moving;

fn setup_physics(mut commands: Commands) {
    /* Create the ground. */
    commands
        .spawn()
        .insert(Collider::cuboid(100.0, 0.1, 100.0))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

    /* Create the bouncing ball. */
    commands
        .spawn()
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7))
        .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 4.0, 0.0)));
}

fn doing_the_wave(time: Res<Time>, mut query: Query<&mut Transform, With<Moving>>){

    for mut transform in &mut query{
        let x_pos:f32 = (time.seconds_since_startup() / 5.0).sin() as f32;
        transform.rotate_x(0.3*time.delta_seconds());
        //let forward = transform.forward();
        transform.translation = Vec3{
            x: x_pos,
            y: 0.5,
            z: 1.5
        };
    }
}

// struct that identifies a component for user input.
#[derive(Component)]
struct Controlling;

#[derive(Component)]
struct ControllingButWithInfo{
    y_vel:f32,
    theta:f32,
}

fn controls (
        time: Res<Time>,
        mut query: Query<(&mut Transform, &mut ControllingButWithInfo), With<Controlling>>,
        keyboard_input: Res<Input<KeyCode>>,
        mut motion_evr: EventReader<MouseMotion>
        ){
    //Time since previous frame
    //useful bc then physics arent bound to framerate
    let time_modifier = time.delta_seconds();

    

    for  (mut transform, mut player_info) in &mut query{

        // Multipliers that make things work based on the direction of the camera, in addition to the FPS
        // The labels on them are based on how they work for the forward/backwards (W/S) directions
        // This is because when going sideways (A/D), the intended direction is perpendicular to W/S
        // Perpendicular lines mean slope of the negative reciprocal, which means flipping the X and Y
        // Hopefully this naming scheme still makes sense in a few months :)
        let x_multiplier = player_info.theta.sin() * time_modifier;
        let z_multiplier = player_info.theta.cos() * time_modifier;

        if keyboard_input.pressed(KeyCode::A){
            transform.translation = transform.translation + Vec3{x:-2.0*z_multiplier,y:0.0,z:2.0*x_multiplier};
        }

        if keyboard_input.pressed(KeyCode::D){
            transform.translation = transform.translation + Vec3{x:2.0*z_multiplier,y:0.0,z:-2.0*x_multiplier};
        }

        if keyboard_input.pressed(KeyCode::W){
            transform.translation = transform.translation + Vec3{x:-2.0*x_multiplier,y:0.0,z:-2.0*z_multiplier};
        }

        if keyboard_input.pressed(KeyCode::S){
            transform.translation = transform.translation + Vec3{x:2.0*x_multiplier,y:0.0,z:2.0*z_multiplier};
        }

        if keyboard_input.pressed(KeyCode::Space){
            if transform.translation.y <= 0.1 {
                player_info.y_vel = 2.0;
            }
        }

        for ev in motion_evr.iter() {
            player_info.theta -= 0.001 * ev.delta.x;
        }

        
    }

    //Gravity and velocity
    
    for (mut transform, mut player_info) in &mut query {
        //println!("Position is {}, Velocity is {}", transform.translation.y, player_info.y_vel);
        //gravity
        if transform.translation.y > 0.0 {
            player_info.y_vel = player_info.y_vel - 2.0*time_modifier;
        } else if player_info.y_vel < 0.0 {
            player_info.y_vel = 0.0
        }
        
        // velocity
        transform.translation = transform.translation + Vec3{x:0.0,y:player_info.y_vel*time_modifier,z:0.0};
    }

    

}

fn move_camera_system(mut cameras: Query<&mut LookTransform>, players: Query<(&mut Transform, &mut ControllingButWithInfo), With<Controlling>>) {
    // Later, another system will update the `Transform` and apply smoothing automatically.
    for mut c in cameras.iter_mut() {
        for (transform, player_info) in &players {
            c.target = transform.translation;
            c.eye = transform.translation + Vec3{ x: 7.0 * player_info.theta.sin(), y: 7.0, z: 7.0 * player_info.theta.cos() }
        }
    }
}

// Cursor crab function shamelessly stolen from bevy-cheatbook.github.io
fn cursor_grab_system(
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Escape) {
        window.set_cursor_lock_mode(false);
        window.set_cursor_visibility(true);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ass: Res<AssetServer>,) {
    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 15.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    }).insert(Collider::cuboid(7.5, 0.5, 7.5));


    // player cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    }).insert(Controlling)
      .insert(ControllingButWithInfo {y_vel:0.0, theta:45.0})
      .insert(Collider::cuboid(0.5, 0.5, 0.5));


    // moving cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        transform: Transform::from_xyz(0.0, 0.5, 1.5),
        ..default()
    }) .insert(Moving)
        .insert(Collider::cuboid(0.5, 0.5, 0.5));


    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    let eye = Vec3::default();
    let target = Vec3::default();

    //claw 

    //let my_gltf = ass.load("claw.gltf#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
   /*  commands.spawn_bundle(SceneBundle {
        scene: my_gltf.clone(),
        ..Default::default()
    });*/

    commands
        .spawn_bundle(LookTransformBundle {
            transform: LookTransform::new(eye, target),
            smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
        })
        .insert_bundle(Camera3dBundle::default());
}

