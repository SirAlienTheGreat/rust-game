pub(crate) mod setup_objects{
    use bevy_rapier3d::prelude::*;    
    use bevy::{prelude::*, core_pipeline::bloom::BloomSettings};
    use rand::prelude::*;

    #[cfg(feature="use-ray-tracing")]
    use bevy_hikari::HikariSettings;

    use smooth_bevy_cameras::{LookTransform, LookTransformBundle, Smoother};

    const WORLD_SIZE:f32 = 300.0;

    #[derive(Component)]
    pub(crate) struct Moving;

    // struct that identifies a component for user input.
    #[derive(Component)]
    pub(crate) struct Controlling;

    #[derive(Component)]
    pub(crate) struct PointingAtPlayer;

    #[derive(Component)]
    pub(crate) struct ControllingButWithInfo{
        pub(crate) theta:f32,
        pub(crate) v_theta: f32,
        pub(crate) has_contacts:bool,
        pub(crate) has_hit_object:bool,
        pub(crate) objects_hit:i32,
    }
    
    pub(crate) fn setup(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>) {
        // plane
        /*commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 2.0*WORLD_SIZE })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, -0.5, 0.0),
            ..default()
        }).insert(Collider::cuboid(WORLD_SIZE, 0.1, WORLD_SIZE));*/

        //walls 
        for i in [-1.0,1.0] {
            commands
                .spawn_empty()
                .insert(Collider::cuboid(0.1, 100.0, 100.0))
                .insert(TransformBundle::from(Transform::from_xyz(i* WORLD_SIZE, 0.0, 0.0)));
            
            commands
                .spawn_empty()
                .insert(Collider::cuboid(100.0, 100.0, 0.1))
                .insert(TransformBundle::from(Transform::from_xyz(0.0, 0.0, i* WORLD_SIZE)));
        }

        // player cube
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size:1.0 })),
            material: materials.add(StandardMaterial { 
                base_color: Color::GRAY, 
                emissive: Color::YELLOW,
                metallic: 1.0, 
                reflectance: 0.9, 
                ..default()}),
            transform: Transform::from_xyz(0.0, 5.5, 0.0),
            ..default()
        })  .insert(Controlling)
            .insert(ControllingButWithInfo {theta:0.0, v_theta: 0.0, has_contacts:true, has_hit_object:false, objects_hit:0})
            .insert(Collider::cuboid(0.5, 0.5, 0.5))
            .insert(RigidBody::Dynamic)
            .insert(Velocity {
                linvel: Vec3::new(1.0, -6.0, 3.0),
                angvel: Vec3::new(0.2, 0.4, 0.8),
            })
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(ExternalForce {
                force: Vec3{ x: 0.0, y: 0.0, z: 0.0 },
                torque: Vec3{ x: 0.0, y: 0.0, z: 0.0 },
            })
            .with_children(|parent|{
                parent.spawn(PointLightBundle{
                    point_light:PointLight{
                        color:Color::YELLOW,
                        radius:0.01,
                        intensity:100.0,
                        shadow_depth_bias:0.9,
                        shadow_normal_bias:0.01,
                        shadows_enabled: true,
                        ..default()
                    },
                    visibility:bevy::prelude::Visibility { is_visible: true },
                    ..default()
                });
            });

        // moving cube
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
            material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
            transform: Transform::from_xyz(0.0, 0.5, 1.5),
            ..default()
        })  .insert(Moving)
            .insert(Collider::cuboid(0.25, 0.25, 0.25));

        // balls
        let mut rng = rand::thread_rng();
        /*for _ in 0..10 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, sectors: 10, stacks:10  })),
                material: materials.add(StandardMaterial { base_color: Color::GRAY, emissive: Color::DARK_GRAY, unlit:false, ..default() }),
                transform: Transform::from_xyz(
                    rng.gen::<f32>() * WORLD_SIZE *2.0 - WORLD_SIZE,
                    10.0* rng.gen::<f32>(), 
                    rng.gen::<f32>() * WORLD_SIZE *2.0 - WORLD_SIZE),
                ..default()
            })
                .insert(RigidBody::Dynamic)
                .insert(Collider::ball(0.5))
                .insert(Restitution::coefficient(0.7));
        }*/

        // lights
        for (x,y,z, h) in [
                (5.0, -38.0, 13.0, 100.0), 
                (17.8, -45.1, 30.9, 311.0), 
                (17.35, -45.1, 39.5, 311.0), 
                (9.7, -45.1, 54.6, 311.0),
                (1.35, -45.1, 54.15, 311.0), 
                (-8.9, -45.1, 39.15, 311.0), 
                (-8.3, -45.1, 30.3, 311.0)]{
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.5, sectors: 10, stacks:10  })),
                material: materials.add(Color::hsla(h, 100.0, 0.05, 0.7).into()),
                transform: Transform::from_xyz( 
                    x, 
                    y, 
                    z),
                    ..default()
            }).with_children(|parent|{
                parent.spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 10.0,
                        radius: 1.0,
                        shadows_enabled: true,
                        color: Color::hsla(h, 100.0, 0.05, 0.0001),
                        ..default()
                    },
                    
                    ..default()
                });
            });
            
        }
        
        #[cfg(feature="use-ray-tracing")]
        commands.spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                color: Color::rgb(255.0,255.0,255.0),
                illuminance: 0.001,
                shadows_enabled: true,

                ..default()
            },
            
            ..default()
        });

        // ambient light
        /*commands.insert_resource(AmbientLight {
            color: Color::YELLOW,
            brightness: 0.1,
        });*/

        // Sun
        commands.spawn(DirectionalLightBundle {
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
                illuminance: 80.0,
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
            commands.spawn(PbrBundle {
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

        commands.spawn(PbrBundle {
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

        commands.spawn(PbrBundle {
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
            .spawn(LookTransformBundle {
                transform: LookTransform::new(eye, target),
                smoother: Smoother::new(0.9), // Value between 0.0 and 1.0, higher is smoother.
            })
            .insert((Camera3dBundle{
                
                camera:Camera { 
                    // HDR (needed for bloom) doesn't seem to work for WASM, so its disabled when on WASM
                    #[cfg(not(target_arch = "wasm32"))]
                    hdr: true, 
                    ..default() 
                },
                ..default()
            }, BloomSettings{
                intensity: 0.5,
                ..default()
            },
            #[cfg(feature="use-ray-tracing")]
            HikariSettings::default(),
            ));
    }

    pub(crate) fn point_things_at_player(
            mut items_to_point: Query<(&mut Transform, With<PointingAtPlayer>, Without<Controlling>)>, 
            player: Query<&mut Transform, With<Controlling>>){

        let direction = player.single().translation;
        for mut item in items_to_point.iter_mut() {
            //let mut item = item.as_mut();
            let x  = item.0.looking_at(direction.clone(), Vec3::new(0.0,1.0,0.0));
            item.0.rotation = x.rotation;
        }
    }
}