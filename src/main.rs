use bevy::{prelude::*, window::CursorGrabMode};
use bevy_kira_audio::{AudioControl, AudioPlugin};
use smooth_bevy_cameras::{LookTransform, LookTransformPlugin};
use bevy_rapier3d::prelude::*;
use std::env;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod decomp_caching;
mod setup_world;
mod movement;
mod skyboxv2;

fn main() {
    let args: Vec<String> = env::args().collect();
    let conf = CliArgs {
        hitboxes: args.contains(&"hitboxes".to_string()),
        show_fps: args.contains(&"show_fps".to_string()) || args.contains(&"fps".to_string()),
        debug: args.contains(&"debug".to_string()),
    };


    let mut app = App::new();

    app 
        .add_plugins(DefaultPlugins)
        .add_plugins(AudioPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(LookTransformPlugin)
        
        .insert_resource(ClearColor(Color::rgb(0.,0.,0.)))
        .insert_resource(TimesLoaded{times:0})
        
        
        .add_systems(Startup, setup_world::setup_objects::setup)
        .add_systems(Startup, spawn_gltf_objects)
        .add_systems(Startup, start_background_audio)

        
        .add_systems(Update, doing_the_wave)
        .add_systems(Update, movement::movement::gamepad_connections)
        .add_systems(Update, movement::movement::controls)
        .add_systems(Update, move_camera_system)

        
        .add_systems(Update, setup_world::setup_objects::point_things_at_player)

        .add_systems(Update, move_scene_entities)

        .add_plugins(bevy::diagnostic::LogDiagnosticsPlugin::default())
        
        
        .add_systems(
            Update,
            (
                //skyboxv2::skyboxv2::cycle_cubemap_asset,
                skyboxv2::skyboxv2::asset_loaded.after(skyboxv2::skyboxv2::cycle_cubemap_asset),
                //skyboxv2::skyboxv2::camera_controller,
                //skyboxv2::skyboxv2::animate_light_direction,
            ),
        );
        

        //.add_plugins(MaterialPlugin::<skybox::skybox::CubemapMaterial>::default())
        //.add_systems(Startup, skybox::skybox::setup_cubebox)
        //.add_systems(Update, skybox::skybox::cycle_cubemap_asset)
        //.add_systems(Update, skybox::skybox::asset_loaded.after(skybox::skybox::cycle_cubemap_asset));

    if conf.hitboxes{
        app.add_plugins(RapierDebugRenderPlugin::default());
    }
    if conf.show_fps{
        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    }
    if conf.debug{
        app.add_plugins(WorldInspectorPlugin::new());
    } else {
        app.add_systems(Update, cursor_grab_system);
    }

    app.run();
}
// struct that indicates that item will move as sin wave

fn start_background_audio(asset_server: Res<AssetServer>, audio: Res<bevy_kira_audio::Audio>) {
    //bevy_kira_audio::AudioControl::play(&audio, asset_server.load("Glimpsing-Infinity-Asher-Fulero.mp3")).looped().with_volume(0.25);
    audio.play(asset_server.load("Glimpsing-Infinity-Asher-Fulero.mp3")).looped().with_volume(0.25);

    
    println!("playing audio")
}

struct CliArgs{
    hitboxes:bool,
    show_fps:bool,
    debug:bool,
}

fn doing_the_wave(time: Res<Time>, mut query: Query<&mut Transform, With<setup_world::setup_objects::Moving>>){

    for mut transform in &mut query{
        let x_pos:f32 = (time.elapsed_seconds() / 5.0).sin() as f32;
        transform.rotate_x(0.3*time.delta_seconds());
        //let forward = transform.forward();
        transform.translation = Vec3{
            x: x_pos,
            y: 0.5,
            z: 1.5
        };
    }
}

fn move_camera_system(mut cameras: Query<&mut LookTransform>, players: Query<(&mut Transform, &mut setup_world::setup_objects::ControllingButWithInfo), With<setup_world::setup_objects::Controlling>>) {
    // Later, another system will update the `Transform` and apply smoothing automatically.
    for mut c in cameras.iter_mut() {
        for (transform, player_info) in &players {
            c.target = transform.translation;
            c.eye = transform.translation + Vec3{ x: 7.0 * player_info.theta.sin() * player_info.v_theta.cos(), y: 7.0 * player_info.v_theta.sin(), z: 7.0 * player_info.theta.cos() * player_info.v_theta.cos() };
        }
    }
    
}



// Cursor crab function shamelessly stolen from bevy-cheatbook.github.io
fn cursor_grab_system(
    mut windows: Query<&mut Window>,
    mouse: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let mut window = windows.get_single_mut();
    match window {
        Ok(mut window) =>{
            if mouse.just_pressed(MouseButton::Left) {
                window.cursor.visible = false;
                window.cursor.grab_mode = CursorGrabMode::Locked;
            }
        
            if key.just_pressed(KeyCode::Escape) {
                window.cursor.visible = true;
                window.cursor.grab_mode = CursorGrabMode::None;
            }
        },
        Err(err) => println!("Grab didn't work bc of {err}")
    }
    
}







#[derive(Component)]
struct MakeHitboxes;

/// The system needs to load meshes multiple times to fix a bug in rapier
#[derive(Resource)]
struct TimesLoaded{
    times:i32
}

fn spawn_gltf_objects(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    let gltf_h = ass.load("11-18-22_full_asembly_metallic_test.glb#Scene0");

    let scene = SceneBundle {
        scene: gltf_h,
        ..Default::default()
    };
    commands.spawn(scene).insert(MakeHitboxes)
       .insert(Transform::from_scale(Vec3{x:0.2,y:0.2,z:0.2}).with_translation(Vec3{x:6.0,y:-50.0,z:0.0}))
       .insert(Name::new("World"));

    /*for i in 0..1 {
        let gltf_h2 = ass.load("claw.gltf#Scene0");
        let scene2 = SceneBundle {
            scene: gltf_h2,
            ..Default::default()
        };
        commands.spawn(scene2).insert(MakeHitboxes)
            .insert(Transform::from_xyz(3.0 * i as f32 - 15.0,0.0,3.0 * i as f32 - 20.0));
    }*/
}



fn move_scene_entities( 
    mut moved_scene: Query<(Entity, &mut Transform),With<MakeHitboxes>>,
    children: Query<&Children>,
    mesh_handles: Query<&Handle<Mesh>>,
    mut commands: Commands,
    assets: Res<Assets<Mesh>>,
) {
    if moved_scene.iter().len() >=1{
        let mut cache = decomp_caching::decomp_caching::load_cache();
        let children = children.into();
        let mut decompositions: Vec<(Entity, decomp_caching::decomp_caching::RenderedDecomp)> = vec![];

        

        for (moved_scene_entity, _)  in &moved_scene {
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
                    info!("meshing");

                    let mesh_collider = Collider::from_bevy_mesh(mesh,
                        &ComputedColliderShape::TriMesh).unwrap();//ConvexDecomposition(VHACDParameters{..Default::default()})).unwrap();

                    //commands.entity(entity).insert(mesh_collider);
                    // Too memory-intensive?
                    let trimesh = mesh_collider.as_trimesh().unwrap();

                    let vertices = trimesh.vertices().collect::<Vec<bevy::prelude::Vec3>>();
                    let indices = trimesh.indices();

                    


                    let decomposition = decomp_caching::decomp_caching::decompose(vertices, indices.into(), &mut cache);
                    match decomposition {
                        Some(decomp) => decompositions.push((entity,decomp)),
                        None => println!("couldn't decompose shape"),
                    }
                    

                    //commands.entity(entity).insert(decomposition);

                    //println!("removing entity {:?}",moved_scene_entity.id());
                    let x = commands.entity(moved_scene_entity.clone());
                    commands.entity(moved_scene_entity.clone()).remove::<MakeHitboxes>();
                }
                
            }});
            
        }

        for (entity, rendered_decomp) in decompositions {
            let collider:Collider = rendered_decomp.decomp.into();
            commands.entity(entity).insert(collider);
            
        }

        //This is a raelly horrible workaround for a glitch in Rapier
        //When the mesh is loaded, it doesn't scale correctly until its updated
        //So this forces an update for the object by moving it a very small amount
        for (_, mut transform) in &mut moved_scene{
            transform.translation.z += 0.1;
        }
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


