pub(crate) mod skybox {
    //! Load a cubemap texture onto a cube like a skybox and cycle through different compressed texture formats
    use bevy::{
        asset::LoadState,
        pbr::{MaterialPipeline, MaterialPipelineKey},
        prelude::*,
        reflect::TypeUuid,
        render::{
            mesh::MeshVertexBufferLayout,
            render_asset::RenderAssets,
            render_resource::{
                AsBindGroup, AsBindGroupError, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
                BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
                OwnedBindingResource, PreparedBindGroup, RenderPipelineDescriptor, SamplerBindingType,
                ShaderRef, ShaderStages, SpecializedMeshPipelineError, TextureSampleType,
                TextureViewDescriptor, TextureViewDimension,
            },
            renderer::RenderDevice,
            texture::{CompressedImageFormats, FallbackImage},
        },
    };

    const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[
        (
            "textures/space-skybox-k.png",
            CompressedImageFormats::NONE,
        )
    ];

/*fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MaterialPlugin::<CubemapMaterial>::default())
        .add_startup_system(setup_cubebox)
        .add_system(cycle_cubemap_asset)
        .add_system(asset_loaded.after(cycle_cubemap_asset))
        .add_system(camera_controller)
        .add_system(animate_light_direction)
        .run();
}*/
    #[derive(Resource)]
    pub(crate) struct Cubemap {
        is_loaded: bool,
        index: usize,
        image_handle: Handle<Image>,
    }

    pub(crate) fn setup_cubebox(mut commands: Commands, asset_server: Res<AssetServer>) {

        let skybox_handle = asset_server.load(CUBEMAPS[0].0);

        commands.insert_resource(Cubemap {
            is_loaded: false,
            index: 0,
            image_handle: skybox_handle,
        });
    }

    const CUBEMAP_SWAP_DELAY: f32 = 3.0;

    pub(crate) fn cycle_cubemap_asset(
        time: Res<Time>,
        mut next_swap: Local<f32>,
        mut cubemap: ResMut<Cubemap>,
        asset_server: Res<AssetServer>,
        render_device: Res<RenderDevice>,
    ) {
        let now = time.delta_seconds();
        if *next_swap == 0.0 {
            *next_swap = now + CUBEMAP_SWAP_DELAY;
            return;
        } else if now < *next_swap {
            return;
        }
        *next_swap += CUBEMAP_SWAP_DELAY;

        let supported_compressed_formats =
            CompressedImageFormats::from_features(render_device.features());

        let mut new_index = cubemap.index;
        for _ in 0..CUBEMAPS.len() {
            new_index = (new_index + 1) % CUBEMAPS.len();
            if supported_compressed_formats.contains(CUBEMAPS[new_index].1) {
                break;
            }
            info!("Skipping unsupported format: {:?}", CUBEMAPS[new_index]);
        }

        // Skip swapping to the same texture. Useful for when ktx2, zstd, or compressed texture support
        // is missing
        if new_index == cubemap.index {
            return;
        }

        cubemap.index = new_index;
        cubemap.image_handle = asset_server.load(CUBEMAPS[cubemap.index].0);
        cubemap.is_loaded = false;
    }

    pub(crate) fn asset_loaded(
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        mut images: ResMut<Assets<Image>>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut cubemap_materials: ResMut<Assets<CubemapMaterial>>,
        mut cubemap: ResMut<Cubemap>,
        cubes: Query<&Handle<CubemapMaterial>>,
    ) {
        if !cubemap.is_loaded
            && asset_server.get_load_state(cubemap.image_handle.clone_weak()) == LoadState::Loaded
        {
            info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
            let mut image = images.get_mut(&cubemap.image_handle).unwrap();
            // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
            // so they appear as one texture. The following code reconfigures the texture as necessary.
            if image.texture_descriptor.array_layer_count() == 1 {
                image.reinterpret_stacked_2d_as_array(
                    image.texture_descriptor.size.height / image.texture_descriptor.size.width,
                );
                image.texture_view_descriptor = Some(TextureViewDescriptor {
                    dimension: Some(TextureViewDimension::Cube),
                    ..default()
                });
            }

            // spawn cube
            let mut updated = false;
            for handle in cubes.iter() {
                if let Some(material) = cubemap_materials.get_mut(handle) {
                    updated = true;
                    material.base_color_texture = Some(cubemap.image_handle.clone_weak());
                }
            }
            if !updated {
                commands.spawn(MaterialMeshBundle::<CubemapMaterial> {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 10000.0 })),
                    material: cubemap_materials.add(CubemapMaterial {
                        base_color_texture: Some(cubemap.image_handle.clone_weak()),
                    }),
                    ..default()
                });
            }

            cubemap.is_loaded = true;
        }
    }

    #[derive(Debug, Clone, TypeUuid)]
    #[uuid = "9509a0f8-3c05-48ee-a13e-a93226c7f488"]
    pub(crate) struct CubemapMaterial {
        base_color_texture: Option<Handle<Image>>,
    }

    impl bevy::reflect::TypePath for CubemapMaterial {
        fn type_path() -> &'static str {
            todo!()
        }

        fn type_ident() -> Option<&'static str> {
            None
        }

        fn crate_name() -> Option<&'static str> {
            None
        }

        fn module_path() -> Option<&'static str> {
            None
        }

        fn short_type_path() -> &'static str {
            todo!()
        }
    }

    impl Material for CubemapMaterial {
        fn fragment_shader() -> ShaderRef {
            "shaders/cubemap_unlit.wgsl".into()
        }

        fn specialize(
            _pipeline: &MaterialPipeline<Self>,
            descriptor: &mut RenderPipelineDescriptor,
            _layout: &MeshVertexBufferLayout,
            _key: MaterialPipelineKey<Self>,
        ) -> Result<(), SpecializedMeshPipelineError> {
            descriptor.primitive.cull_mode = None;
            Ok(())
        }
    }

    impl AsBindGroup for CubemapMaterial {
        type Data = ();

        fn as_bind_group(
            &self,
            layout: &BindGroupLayout,
            render_device: &RenderDevice,
            images: &RenderAssets<Image>,
            _fallback_image: &FallbackImage,
        ) -> Result<PreparedBindGroup<()>, AsBindGroupError> {
            let base_color_texture = self
                .base_color_texture
                .as_ref()
                .ok_or(AsBindGroupError::RetryNextUpdate)?;
            let image = images
                .get(base_color_texture)
                .ok_or(AsBindGroupError::RetryNextUpdate)?;
            let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&image.texture_view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&image.sampler),
                    },
                ],
                label: Some("cubemap_texture_material_bind_group"),
                layout,
            });

            Ok(PreparedBindGroup {
                bind_group,
                bindings: vec![
                    OwnedBindingResource::TextureView(image.texture_view.clone()),
                    OwnedBindingResource::Sampler(image.sampler.clone()),
                ],
                data: (),
            })
        }

        fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    // Cubemap Base Color Texture
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::Cube,
                        },
                        count: None,
                    },
                    // Cubemap Base Color Texture Sampler
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            })
        }    
    }

}