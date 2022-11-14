use bevy::core_pipeline::core_2d::graph::input;
use bevy::{prelude::*, window::PresentMode};
use bevy::input::mouse::MouseMotion;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};
use bevy_rapier3d::prelude::*;

// Global constants that I'm changing a lot
const WORLD_SIZE:f32 = 10.0;
const PLAYER_SPEED:f32 = 2.5;


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
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_startup_system(setup)
        //.add_startup_system(setup_physics)
        .add_system(move_camera_system)
        .add_system(doing_the_wave)
        .add_system(gamepad)
        .add_system(gamepad_connections)
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
        mut query: Query<(&mut Transform, &mut ControllingButWithInfo, &mut Velocity), With<Controlling>>,
        keyboard_input: Res<Input<KeyCode>>,
        mut motion_evr: EventReader<MouseMotion>
        ){
    //Time since previous frame
    //useful bc then physics arent bound to framerate
    let time_modifier = time.delta_seconds();

    

    for  (mut transform, mut player_info, mut velocity) in &mut query{

        // Multipliers that make things work based on the direction of the camera, in addition to the FPS
        // The labels on them are based on how they work for the forward/backwards (W/S) directions
        // This is because when going sideways (A/D), the intended direction is perpendicular to W/S
        // Perpendicular lines mean slope of the negative reciprocal, which means flipping the X and Y
        // Hopefully this naming scheme still makes sense in a few months :)
        let x_multiplier = player_info.theta.sin();// * time_modifier;
        let z_multiplier = player_info.theta.cos();// * time_modifier;

        // slowing down
        velocity.linvel.x = velocity.linvel.x/2.0;
        velocity.linvel.z = velocity.linvel.z/2.0;

        let mut input_theta:f32 = 0.0;
        let mut moving = false;

        if keyboard_input.pressed(KeyCode::A){
            input_theta = input_theta + std::f32::consts::PI;
            moving = true;
        }

        if keyboard_input.pressed(KeyCode::D){
            input_theta = input_theta;
            moving = true;
        }

        if keyboard_input.pressed(KeyCode::W){
            input_theta = input_theta + 0.5*std::f32::consts::PI;
            moving = true;
        }

        if keyboard_input.pressed(KeyCode::S){
            input_theta = input_theta + 1.5*std::f32::consts::PI;
            moving = true;
        }

        if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::W) {
            input_theta = 0.75*std::f32::consts::PI;
            moving = true;
        }
        
        if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::W) {
            input_theta = 0.25*std::f32::consts::PI;
            
            moving = true;
        }

        if keyboard_input.pressed(KeyCode::A) && keyboard_input.pressed(KeyCode::S) {
            input_theta = 1.25*std::f32::consts::PI;
            moving = true;
        }
        
        if keyboard_input.pressed(KeyCode::D) && keyboard_input.pressed(KeyCode::S) {
            input_theta = 1.75*std::f32::consts::PI;
            moving = true;
        }

        input_theta += player_info.theta;

        if moving{
            velocity.linvel = velocity.linvel + Vec3{ x: PLAYER_SPEED*input_theta.cos(), y: 0.0, z: -PLAYER_SPEED*input_theta.sin() };
            //transform.translation = transform.translation +  Vec3{ x: PLAYER_SPEED*input_theta.cos()*time.delta_seconds(), y: 0.0, z: -PLAYER_SPEED*input_theta.sin()*time.delta_seconds() };
            //maybe try this?
            //if player_info.theta % (std::f32::consts::PI*2) ;
            
            println!("Transforming {input_theta} radians: x:{}, y:{}", input_theta.cos(), input_theta.sin());
        }


        /*if keyboard_input.pressed(KeyCode::A){
            velocity.linvel =  velocity.linvel + Vec3{x:-PLAYER_SPEED*z_multiplier,y:0.0,z:PLAYER_SPEED*x_multiplier};
        }

        if keyboard_input.pressed(KeyCode::D){
            velocity.linvel =  velocity.linvel + Vec3{x:PLAYER_SPEED*z_multiplier,y:0.0,z:-PLAYER_SPEED*x_multiplier};
        }

        if keyboard_input.pressed(KeyCode::W){
            velocity.linvel =  velocity.linvel + Vec3{x:-PLAYER_SPEED*x_multiplier,y:0.0,z:-PLAYER_SPEED*z_multiplier};
        }

        if keyboard_input.pressed(KeyCode::S){
            velocity.linvel =  velocity.linvel + Vec3{x:PLAYER_SPEED*x_multiplier,y:0.0,z:PLAYER_SPEED*z_multiplier};
        }*/

        if keyboard_input.pressed(KeyCode::Space){
            //if transform.translation.y <= 0.1 {
                //player_info.y_vel = 2.0;
                velocity.linvel.y = 2.0;
            //}
            
        }

        for ev in motion_evr.iter() {
            player_info.theta -= 0.001 * ev.delta.x;
        }

        
    }

    //Gravity and velocity
    
    /*for (mut transform, mut player_info) in &mut query {
        //println!("Position is {}, Velocity is {}", transform.translation.y, player_info.y_vel);
        //gravity
        if transform.translation.y > 0.0 {
            player_info.y_vel = player_info.y_vel - 2.0*time_modifier;
        } else if player_info.y_vel < 0.0 {
            player_info.y_vel = 0.0
        }
        
        // velocity
        transform.translation = transform.translation + Vec3{x:0.0,y:player_info.y_vel*time_modifier,z:0.0};
    }*/

    

}

fn gamepad (
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut ControllingButWithInfo, &mut Velocity), With<Controlling>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    my_gamepad: Option<Res<MyGamepad>>,
    ){
    //Time since previous frame
    //useful bc then physics arent bound to framerate
    let time_modifier = time.delta_seconds();

        println!("here");

    for  (mut transform, mut player_info, mut velocity) in &mut query{

        // Multipliers that make things work based on the direction of the camera, in addition to the FPS
        // The labels on them are based on how they work for the forward/backwards (W/S) directions
        // This is because when going sideways (A/D), the intended direction is perpendicular to W/S
        // Perpendicular lines mean slope of the negative reciprocal, which means flipping the X and Y
        // Hopefully this naming scheme still makes sense in a few months :)
        let x_multiplier = player_info.theta.sin();// * time_modifier;
        let z_multiplier = player_info.theta.cos();// * time_modifier;

        let mut input_theta:f32 = 0.0;
        let mut moving = false;

        let gamepad = if let Some(ref gp) = my_gamepad {
            // a gamepad is connected, we have the id
            gp.0
        } else {
            // no gamepad is connected
            return;
        };

        let axis_lx = GamepadAxis {
            gamepad, axis_type: GamepadAxisType::LeftStickX
        };
        let axis_ly = GamepadAxis {
            gamepad, axis_type: GamepadAxisType::LeftStickY
        };

        

        let x = axes.get(axis_lx).unwrap();
        let y = axes.get(axis_ly).unwrap();

        input_theta = 2.0*((y/(x + (x*x + y*y).sqrt())).atan());

        //input_theta = ((axes.get(axis_ly).unwrap())/(axes.get(axis_lx).unwrap())).atan();
        if !input_theta.is_nan(){
            moving = true;
        }
        

        //println!("{:?}",input_theta);

        input_theta += player_info.theta;

        if moving{
            velocity.linvel = velocity.linvel + Vec3{ x: PLAYER_SPEED*input_theta.cos(), y: 0.0, z: -PLAYER_SPEED*input_theta.sin() };
        
            println!("Transforming {input_theta} radians: x:{}, y:{}", input_theta.cos(), input_theta.sin());
        }

        let jump_button = GamepadButton {
            gamepad, button_type: GamepadButtonType::South
        };

        if buttons.pressed(jump_button) {
            velocity.linvel.y = 2.0;
        }


        // camera
        let axis_rx = GamepadAxis {
            gamepad, axis_type: GamepadAxisType::RightStickX
        };

        let c_x = axes.get(axis_rx).unwrap();

        println!("camera stick is {}",c_x);

        player_info.theta -= 2.0 * c_x * time.delta_seconds();

        /*if keyboard_input.pressed(KeyCode::Space){
            velocity.linvel.y = 2.0;
            
        }

        for ev in motion_evr.iter() {
            player_info.theta -= 0.001 * ev.delta.x;
        }*/
        
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
        mesh: meshes.add(Mesh::from(shape::Plane { size: 2.0*WORLD_SIZE })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    }).insert(Collider::cuboid(WORLD_SIZE, 0.1, WORLD_SIZE));

    //walls
    for i in [-1.0,1.0] {
        commands
            .spawn()
            .insert(Collider::cuboid(0.1, 100.0, 100.0))
            .insert_bundle(TransformBundle::from(Transform::from_xyz(i* WORLD_SIZE, 0.0, 0.0)));
        
        commands
            .spawn()
            .insert(Collider::cuboid(100.0, 100.0, 0.1))
            .insert_bundle(TransformBundle::from(Transform::from_xyz(0.0, 0.0, i* WORLD_SIZE)));
    }

    // player cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    })  .insert(Controlling)
        .insert(ControllingButWithInfo {y_vel:0.0, theta:0.0})
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(RigidBody::Dynamic)
        .insert(Velocity {
            linvel: Vec3::new(1.0, 2.0, 3.0),
            angvel: Vec3::new(0.2, 0.4, 0.8),
        });


    // moving cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        transform: Transform::from_xyz(0.0, 0.5, 1.5),
        ..default()
    }) //.insert(Moving)
        .insert(Collider::cuboid(0.25, 0.25, 0.25));

    // ball
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, sectors: 10, stacks:10  })),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        transform: Transform::from_xyz(5.0, 10.5, 1.5),
        ..default()
    })
        .insert(RigidBody::Dynamic)
        .insert(Collider::ball(0.5))
        .insert(Restitution::coefficient(0.7));

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

    let my_gltf = ass.load("octopusanimation.glb#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn_bundle(SceneBundle {
        scene: my_gltf.clone(),
        ..Default::default()
    });

    commands
        .spawn_bundle(LookTransformBundle {
            transform: LookTransform::new(eye, target),
            smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
        })
        .insert_bundle(Camera3dBundle::default());
}

struct MyGamepad(Gamepad);

fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<MyGamepad>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.iter() {
        // the ID of the gamepad
        let id = ev.gamepad;
        match ev.event_type {
            GamepadEventType::Connected => {
                println!("New gamepad connected with ID: {:?}", id);

                // if we don't have any gamepad yet, use this one
                if my_gamepad.is_none() {
                    commands.insert_resource(MyGamepad(id));
                }
            }
            GamepadEventType::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                // if it's the one we previously associated with the player,
                // disassociate it:
                if let Some(MyGamepad(old_id)) = my_gamepad.as_deref() {
                    if *old_id == id {
                        commands.remove_resource::<MyGamepad>();
                    }
                }
            }
            // other events are irrelevant
            _ => {}
        }
    }
}