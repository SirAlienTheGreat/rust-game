pub(crate) mod skyboxv2{
    //! Load a cubemap texture onto a cube like a skybox and cycle through different compressed texture formats

    

    use bevy::{
        asset::LoadState,
        core_pipeline::Skybox,
        prelude::*,
        render::{
            render_resource::{TextureViewDescriptor, TextureViewDimension},
            texture::CompressedImageFormats,
        },
    };

    pub(crate) const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[
        (
            "textures/space-skybox-k.png",
            CompressedImageFormats::NONE,
        ),
    ];

    #[derive(Resource)]
    pub(crate) struct Cubemap {
        pub(crate) is_loaded: bool,
        pub(crate) index: usize,
        pub(crate) image_handle: Handle<Image>,
    }

    pub(crate) fn asset_loaded(
        asset_server: Res<AssetServer>,
        mut images: ResMut<Assets<Image>>,
        mut cubemap: ResMut<Cubemap>,
        mut skyboxes: Query<&mut Skybox>,
    ) {
        if !cubemap.is_loaded
            && asset_server.get_load_state(cubemap.image_handle.clone_weak()) == LoadState::Loaded
        {
            info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
            let image = images.get_mut(&cubemap.image_handle).unwrap();
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

            for mut skybox in &mut skyboxes {
                skybox.0 = cubemap.image_handle.clone();
            }

            cubemap.is_loaded = true;
        }
    }

    
}