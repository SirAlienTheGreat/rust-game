use bevy::{prelude::*, window::{PresentMode, CursorGrabMode}};
#[cfg(feature="use-ray-tracing")]
use bevy_hikari::HikariPlugin;
use smooth_bevy_cameras::{LookTransform, LookTransformPlugin};
use bevy_rapier3d::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;

mod decomp_caching;
mod setup_world;
mod movement;
mod skybox;


fn main() {
    static CREATING_OBJECTS:&str = "creating-objects";

    let mut app = App::new();

    app .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            window: WindowDescriptor {
                title: "Rust is the future of programming!".to_string(),
                width: 1920.,
                height: 1080.,
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            ..default()
        })  .build()
            .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),)
        //.add_plugin(PbrPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(LookTransformPlugin)
        .add_system(move_camera_system)
        .add_system(doing_the_wave)
        //.add_system(gamepad)
        .add_system(movement::movement::gamepad_connections)
        .add_system(movement::movement::controls)
        .add_system(cursor_grab_system)

        .add_startup_stage(CREATING_OBJECTS, SystemStage::single_threaded())
        .add_startup_system_to_stage(CREATING_OBJECTS, setup_world::setup_objects::setup)
        .add_startup_system_to_stage(CREATING_OBJECTS, spawn_gltf_objects)
        .add_system(setup_world::setup_objects::point_things_at_player)

        .add_system(move_scene_entities)

        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())

        .add_plugin(MaterialPlugin::<skybox::skybox::CubemapMaterial>::default())
        .add_startup_system(skybox::skybox::setup_cubebox)
        .add_system(skybox::skybox::cycle_cubemap_asset)
        .add_system(skybox::skybox::asset_loaded.after(skybox::skybox::cycle_cubemap_asset));

    #[cfg(feature="use-ray-tracing")]
    {app.add_plugin(HikariPlugin);}

    app.run();
}
// struct that indicates that item will move as sin wave


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
    mut windows: ResMut<Windows>,
    btn: Res<Input<MouseButton>>,
    key: Res<Input<KeyCode>>,
) {
    let window = windows.get_primary_mut().unwrap();

    if btn.just_pressed(MouseButton::Left) {
        println!("grabbing mouse");
        window.set_cursor_grab_mode(CursorGrabMode::Locked);
        window.set_cursor_visibility(false);
    }

    if key.just_pressed(KeyCode::Escape) {
        println!("ungrabbing mouse");
        window.set_cursor_grab_mode(CursorGrabMode::None);
        window.set_cursor_visibility(true);
    }
}

/// set up a simple 3D scene


#[derive(Component)]
struct MakeHitboxes;

fn spawn_gltf_objects(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    println!("making objects");
    let gltf_h = ass.load("11-8-22_voxel_cave_setting_v2.glb#Scene0");
    let scene = SceneBundle {
        scene: gltf_h,
        ..Default::default()
    };
    commands.spawn(scene).insert(MakeHitboxes)
       .insert(Transform::from_scale(Vec3{x:0.2,y:0.2,z:0.2}).with_translation(Vec3{x:6.0,y:-50.0,z:0.0}));
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
    if moved_scene.iter().len() >=1{
        let mut cache = decomp_caching::decomp_caching::load_cache();
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

                    

                    let decomposition = decomp_caching::decomp_caching::decompose(vertices, indices.into(), &mut cache);
                    match decomposition {
                        Some(decomp) => decompositions.push((entity,decomp)),
                        None => println!("couldn't decompose shape"),
                    }
                    

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
    

    
    
}

fn iter_hierarchy(entity: Entity, children_query: &Query<&Children>, f: &mut impl FnMut(Entity)) {
    (f)(entity);
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter().copied() {
            iter_hierarchy(child, children_query, f);
        }
    }
}


