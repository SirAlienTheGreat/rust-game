use bevy::{prelude::*, window::PresentMode};
use bevy::input::mouse::MouseMotion;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};
use bevy_rapier3d::prelude::*;
use rand::prelude::*;
mod decomp_caching;

// Global constants that I'm changing a lot
const WORLD_SIZE:f32 = 300.0;
const PLAYER_SPEED:f32 = 2.5;
const DEADZONE:f32 = 0.10;

// Convex decomposition



fn main() {
    static CREATING_OBJECTS:&str = "creating-objects";

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
        .add_system(move_camera_system)
        .add_system(doing_the_wave)
        //.add_system(gamepad)
        .add_system(gamepad_connections)
        .add_system(controls)
        .add_system(cursor_grab_system)

        .add_startup_stage(CREATING_OBJECTS, SystemStage::single_threaded())
        .add_startup_system_to_stage(CREATING_OBJECTS, setup)
        .add_startup_system_to_stage(CREATING_OBJECTS, spawn_gltf_objects)

        .add_system(move_scene_entities)

        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .run();
}
// struct that indicates that item will move as sin wave
#[derive(Component)]
struct Moving;

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
    has_contacts:bool,
    has_hit_object:bool,
    objects_hit:i32,
}

fn controls (
        time: Res<Time>,
        mut query: Query<(&mut ControllingButWithInfo, &mut Velocity, Entity), With<Controlling>>,
        keyboard_input: Res<Input<KeyCode>>,
        mut motion_evr: EventReader<MouseMotion>,
        axes: Res<Axis<GamepadAxis>>,
        buttons: Res<Input<GamepadButton>>,
        my_gamepad: Option<Res<MyGamepad>>,
        mut collision_events: EventReader<CollisionEvent>,
        rapier_context: Res<RapierContext>,
        ){
    //Time since previous frame
    //useful bc then physics arent bound to framerate
    //println!("FPS: {:?}",bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS);

    for  (mut player_info, mut velocity, entity) in &mut query{

        // slowing down
        velocity.linvel.x = velocity.linvel.x/2.0;
        velocity.linvel.z = velocity.linvel.z/2.0;

        let mut input_theta:f32 = 0.0;
        let mut speed = 1.0;
        let mut moving = false;

        let mut dashing = false;


        if collision_events.len() >= 1 {
            player_info.has_hit_object = true;
        }
        for i in collision_events.iter() {
            println!("Collision event: {:?}",i);
            match i {
                CollisionEvent::Started(_, _, _) => {
                    player_info.objects_hit+=1;
                },
                CollisionEvent::Stopped(_,_, _) => {
                    player_info.objects_hit-=1;
                },
            }
        }


        if rapier_context.contacts_with(entity).count() >= 1 {
            player_info.has_contacts = true;
            let mut colliders = vec![];
            for i in rapier_context.contacts_with(entity) {
                colliders.push(i.collider2())
            }
        } else {
            player_info.has_contacts = false;
        }

        // Controller controls
        let gamepad = if let Some(ref gp) = my_gamepad {
            // a gamepad is connected, we have the id
            Some(gp.0)
        } else {
            None
        };
        match gamepad {
            Some(gamepad) => {
                // If a gamepad exists, this code is run.
                
                let axis_lx = GamepadAxis {
                    gamepad, axis_type: GamepadAxisType::LeftStickX
                };
                let axis_ly = GamepadAxis {
                    gamepad, axis_type: GamepadAxisType::LeftStickY
                };

                
                //Reading Left stick
                let x = axes.get(axis_lx).unwrap_or(0.0_f32);
                let y = axes.get(axis_ly).unwrap_or(0.0_f32);

                input_theta = 2.0*((y/(x + (x*x + y*y).sqrt())).atan());

                if y == 0.0 && x < 0.0 {
                    input_theta = std::f32::consts::PI;
                }

                if !input_theta.is_nan(){
                    moving = true;
                    speed = (x*x + y*y).sqrt();
                    if speed > 1.0 {
                        speed = 1.0;
                    }
                } else {
                    input_theta = 0.0;
                }

                if speed <= DEADZONE {
                    moving = false;
                }

                // camera
                let axis_rx = GamepadAxis {
                    gamepad, axis_type: GamepadAxisType::RightStickX
                };

                let c_x = axes.get(axis_rx).unwrap_or(0.0_f32);


                player_info.theta -= 2.0 * c_x * time.delta_seconds();
                
                // Jumping
                let jump_button = GamepadButton {
                    gamepad, button_type: GamepadButtonType::South
                };
        
                if buttons.pressed(jump_button) && player_info.objects_hit >=1{
                    velocity.linvel.y = 10.0;
                    player_info.has_hit_object = false;
                }

                // Fast fall
                let fast_fall_button = GamepadButton {
                    gamepad, button_type: GamepadButtonType::RightTrigger2
                };

                if buttons.pressed(fast_fall_button)&& player_info.objects_hit <=1{
                    velocity.linvel.y = -30.0;
                }

                // Dash
                let dash_button = GamepadButton {
                    gamepad, button_type: GamepadButtonType::West
                };

                dashing = buttons.just_pressed(dash_button)
                
            }
            None => {},
        }

        // Keyboard controls

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
            velocity.linvel = velocity.linvel + Vec3{ x: PLAYER_SPEED*input_theta.cos()*speed, y: 0.0, z: -PLAYER_SPEED*input_theta.sin()*speed };                        
        }

        if keyboard_input.pressed(KeyCode::Space) && player_info.objects_hit >=1{
            velocity.linvel.y = 10.0;
            player_info.has_hit_object = false;
            
        }

        // Fast fall

        if keyboard_input.pressed(KeyCode::LShift) && !player_info.objects_hit >=1{
            velocity.linvel.y = -30.0;
        }

        // Dashing
        if moving && (keyboard_input.just_pressed(KeyCode::Q) || dashing){
            velocity.linvel = velocity.linvel + Vec3{ x: PLAYER_SPEED*input_theta.cos()*speed * 30.0, y: 0.0, z: -PLAYER_SPEED*input_theta.sin()*speed  * 30.0};
        }

        for ev in motion_evr.iter() {
            player_info.theta -= 0.001 * ev.delta.x;
        }
   
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
    mut materials: ResMut<Assets<StandardMaterial>>) {
    // plane
    /*commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 2.0*WORLD_SIZE })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    }).insert(Collider::cuboid(WORLD_SIZE, 0.1, WORLD_SIZE));*/

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
        transform: Transform::from_xyz(0.0, 5.5, 0.0),
        ..default()
    })  .insert(Controlling)
        .insert(ControllingButWithInfo {y_vel:0.0, theta:0.0, has_contacts:true, has_hit_object:false, objects_hit:0})
        .insert(Collider::cuboid(0.5, 0.5, 0.5))
        .insert(RigidBody::Dynamic)
        .insert(Velocity {
            linvel: Vec3::new(1.0, 2.0, 3.0),
            angvel: Vec3::new(0.2, 0.4, 0.8),
        })
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ExternalForce {
            force: Vec3{ x: 0.0, y: 0.0, z: 0.0 },
            torque: Vec3{ x: 0.0, y: 0.0, z: 0.0 },
        });


    // moving cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        transform: Transform::from_xyz(0.0, 0.5, 1.5),
        ..default()
    })  .insert(Moving)
        .insert(Collider::cuboid(0.25, 0.25, 0.25));

    // balls
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, sectors: 10, stacks:10  })),
            material: materials.add(Color::rgb(rng.gen(), rng.gen(), rng.gen()).into()),
            transform: Transform::from_xyz(
                rng.gen::<f32>() * WORLD_SIZE *2.0 - WORLD_SIZE,
                10.0* rng.gen::<f32>(), 
                rng.gen::<f32>() * WORLD_SIZE *2.0 - WORLD_SIZE),
            ..default()
        })
            .insert(RigidBody::Dynamic)
            .insert(Collider::ball(0.5))
            .insert(Restitution::coefficient(0.7));
    }
    // lights

    for _ in 0..8{
        commands.spawn_bundle(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz( 
                rng.gen::<f32>() * WORLD_SIZE *2.0 - WORLD_SIZE, 
                8.0, 
                rng.gen::<f32>() * WORLD_SIZE *2.0 - WORLD_SIZE),
            ..default()
        });
    }
    

    /*commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::rgb(255.0,255.0,255.0),
            illuminance: 13000.0,
            shadows_enabled: true,

            ..default()
        },
        
        ..default()
    });*/

    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::YELLOW,
        brightness: 0.1,
    });

    // Sun
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -WORLD_SIZE,
                right: WORLD_SIZE,
                bottom: -WORLD_SIZE,
                top: WORLD_SIZE,
                near: -10.0 * WORLD_SIZE,
                far: 10.0 * WORLD_SIZE,
                ..default()
            },
            illuminance: 8000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
            ..default()
        },
        ..default()
    });


    //platforms 
    let init_x = rng.gen::<f32>()* 2.0* WORLD_SIZE- WORLD_SIZE;

    for i in 0..3 {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box { 
                min_x: -2.5, 
                max_x: 2.5, 
                min_y: -0.5, 
                max_y: 0.5, 
                min_z: -2.5, 
                max_z: 2.5 })),
            material: materials.add(Color::rgb(1.0, i as f32 / 3.0, 0.0).into()),
            transform: Transform::from_xyz(init_x, 3.0 * i as f32, i as f32 * 8.0),
            ..default()
        }) //.insert(Moving)
            .insert(Collider::cuboid(2.5, 0.5, 2.5));
    }

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box { 
            min_x: -9.0, 
            max_x: 9.0, 
            min_y: -2.5, 
            max_y: 2.5, 
            min_z: -0.5, 
            max_z: 0.5 })),
        material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        transform: Transform::from_xyz(4.5, 7.0, 8.0),
        ..default()
    }) //.insert(Moving)
        .insert(Collider::cuboid(9.0, 2.5, 0.5));

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box { 
            min_x: -9.0, 
            max_x: 9.0, 
            min_y: -2.5, 
            max_y: 2.5, 
            min_z: -0.5, 
            max_z: 0.5 })),
        material: materials.add(Color::rgb(1.0, 0.0, 1.0).into()),
        transform: Transform::from_xyz(4.5, 7.0, 15.0),
        ..default()
    }) //.insert(Moving)
        .insert(Collider::cuboid(9.0, 2.5, 0.5));

    let eye = Vec3::default();
    let target = Vec3::default();

    commands
        .spawn_bundle(LookTransformBundle {
            transform: LookTransform::new(eye, target),
            smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
        })
        .insert_bundle(Camera3dBundle::default());
}

#[derive(Component)]
struct MakeHitboxes;

fn spawn_gltf_objects(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    println!("making objects");
    let gltf_h = ass.load("cave.gltf#Scene0");
    let scene = SceneBundle {
        scene: gltf_h,
        ..Default::default()
    };
    commands.spawn_bundle(scene).insert(MakeHitboxes)
       .insert(Transform::from_scale(Vec3{x:0.9,y:0.9,z:0.9}).with_translation(Vec3{x:6.0,y:-50.0,z:0.0}));
    println!("made objects");
    /*for i in 0..1 {
        let gltf_h2 = ass.load("claw.glb#Scene0");
        let scene2 = SceneBundle {
            scene: gltf_h2,
            ..Default::default()
        };
        commands.spawn_bundle(scene2).insert(MakeHitboxes)
            .insert(Transform::from_xyz(3.0 * i as f32 - 15.0,3.0,3.0 * i as f32 - 20.0));
    }*/
}


fn move_scene_entities(
    moved_scene: Query<Entity, With<MakeHitboxes>>,
    children: Query<&Children>,
    mesh_handles: Query<&Handle<Mesh>>,
    mut commands: Commands,
    assets: Res<Assets<Mesh>>,
) {
    let children = children.into();
    let mut decompositions = vec![];

    for moved_scene_entity in &moved_scene {
        iter_hierarchy(moved_scene_entity, &children, &mut {
            
            |entity| {
            /*if let Ok(mut transform) = transforms.get_mut(entity) {
                transform.translation = Vec3::new(
                    offset * time.seconds_since_startup().sin() as f32 / 20.,
                    0.,
                    time.seconds_since_startup().cos() as f32 / 20.,
                );
                offset += 1.0;
                
            }*/

            if let Ok(mesh_handle) = mesh_handles.get(entity) {
                let mesh = assets.get(mesh_handle).expect("Couldn't get mesh from handle");
                

                let mesh_collider = Collider::from_bevy_mesh(mesh,
                    &ComputedColliderShape::TriMesh).unwrap();//ConvexDecomposition(VHACDParameters{..Default::default()})).unwrap();

                //commands.entity(entity).insert(mesh_collider);
                // Too memory-intensive?
                let trimesh = mesh_collider.as_trimesh().unwrap();

                let vertices = trimesh.vertices().collect::<Vec<bevy::prelude::Vec3>>();
                let indices = trimesh.indices();

                

                let decomposition = decomp_caching::decomp_caching::decompose(vertices, indices.into());
                decompositions.push((entity,decomposition));

                //commands.entity(entity).insert(decomposition);

                //println!("removing entity {:?}",moved_scene_entity.id());

                commands.entity(moved_scene_entity.clone()).remove::<MakeHitboxes>();
            }
            
        }});
        
    }

    for (entity, rendered_decomp) in decompositions {
        let collider:Collider = rendered_decomp.decomp.into();
        commands.entity(entity).insert(collider);
    }

    
    
}

fn iter_hierarchy(entity: Entity, children_query: &Query<&Children>, f: &mut impl FnMut(Entity)) {
    (f)(entity);
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter().copied() {
            iter_hierarchy(child, children_query, f);
        }
    }
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
