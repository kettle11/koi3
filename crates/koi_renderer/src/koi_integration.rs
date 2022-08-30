use crate::{
    cube_map::{initialize_cube_maps, CubeMap},
    *,
};

use koi_assets::*;
use koi_resources::Resources;
use koi_transform::GlobalTransform;

pub struct InitialSettings {
    pub name: String,
    pub window_width: usize,
    pub window_height: usize,
    pub color_space: koi_graphics_context::ColorSpace,
}

impl Default for InitialSettings {
    fn default() -> Self {
        Self {
            name: "Koi".into(),
            window_width: 800,
            window_height: 800,
            color_space: koi_graphics_context::ColorSpace::SRGB,
        }
    }
}

pub fn initialize_plugin(resources: &mut Resources) {
    let world_cloner = resources.get_mut::<koi_ecs::WorldCloner>();
    world_cloner.register_clone_type::<Handle<Material>>();
    world_cloner.register_clone_type::<Handle<Mesh>>();

    let initial_settings = resources.remove::<InitialSettings>().unwrap_or_default();

    let window_width = initial_settings.window_width;
    let window_height = initial_settings.window_height;
    // Create the primary window
    let window = {
        let kapp_app = resources.get::<kapp::Application>();
        kapp_app
            .new_window()
            .title(&initial_settings.name)
            .size(window_width as _, window_height as _)
            .build()
    };

    // Initialize the graphics context.
    let mut graphics_context = koi_graphics_context::GraphicsContext::new(
        koi_graphics_context::GraphicsContextSettings {
            high_resolution_framebuffer: true,
            /// How many MSAA samples the window framebuffer should have
            samples: 4,
            color_space: Some(initial_settings.color_space),
        },
        &window,
    );

    // graphics_context.resize(&window, window_width as _, window_height as _);

    // For now this needs to be called even if it's not used.
    /*
    let _render_target = graphics_context.get_render_target_for_window(
        &window,
        window_width as _,
        window_height as _,
    );
    */

    resources.add(window);

    let mut renderer = Renderer::new(graphics_context, initial_settings.color_space);

    initialize_shaders(&mut renderer, resources);
    let materials = initialize_materials();
    let meshes = initialize_meshes(&mut renderer.raw_graphics_context);
    let textures = initialize_textures(&mut renderer);
    let morphable_meshes = AssetStore::<MorphableMeshData>::new(MorphableMeshData {
        mesh: Handle::PLACEHOLDER,
        morph_targets_texture: Handle::PLACEHOLDER,
    });

    resources.add(renderer);
    resources.add(materials);
    resources.add(meshes);
    resources.add(textures);
    resources.add(morphable_meshes);

    initialize_cube_maps(resources);

    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_handler(koi_events::Event::PostDraw, draw);
}

pub fn draw(_: &koi_events::Event, world: &mut koi_ecs::World, resources: &mut Resources) {
    let mut cube_maps = resources.get::<AssetStore<CubeMap>>();
    // cube_maps.finalize_asset_loads(resources);

    let window = resources.get::<kapp::Window>();
    let mut meshes = resources.get::<AssetStore<Mesh>>();
    let mut materials = resources.get::<AssetStore<Material>>();
    let mut shaders = resources.get::<AssetStore<Shader>>();
    let mut textures = resources.get::<AssetStore<Texture>>();
    let morphable_mesh_data = resources.get::<AssetStore<MorphableMeshData>>();

    meshes.finalize_asset_loads(resources);
    materials.finalize_asset_loads(resources);
    shaders.finalize_asset_loads(resources);
    textures.finalize_asset_loads(resources);

    let mut renderer = resources.get::<Renderer>();

    meshes.cleanup_dropped_assets();
    materials.cleanup_dropped_assets();
    shaders.cleanup_dropped_assets();
    textures.cleanup_dropped_assets();
    cube_maps.cleanup_dropped_assets();

    let (window_width, window_height) = window.size();
    // TODO: Does this need to be resized?
    // renderer
    //     .raw_graphics_context
    //     .resize(&*window, window_width, window_height);

    let mut camera_query = world.query::<(&GlobalTransform, &Camera)>();

    // TODO: Avoid this allocation
    let mut cameras = Vec::new();

    for (t, c) in camera_query.iter() {
        cameras.push((t.clone(), c.clone()));
    }

    for (_, (camera_transform, camera)) in cameras {
        let mut render_pass = renderer.begin_render_pass(
            camera,
            camera_transform,
            window_width as f32,
            window_height as f32,
        );

        let mut directional_lights =
            world.query::<koi_ecs::Without<(&DirectionalLight, &GlobalTransform), &Camera>>();

        for (_, (light, light_transform)) in directional_lights.iter() {
            render_pass.add_directional_light(light_transform, light)
        }

        let mut point_lights =
            world.query::<koi_ecs::Without<(&PointLight, &GlobalTransform), &Camera>>();

        for (_, (light, light_transform)) in point_lights.iter() {
            render_pass.add_point_light(light_transform, light)
        }

        let mut renderables = world.query::<(&Handle<Mesh>, &Handle<Material>, &GlobalTransform)>();

        for (_, (gpu_mesh, material, transform)) in renderables.iter() {
            render_pass.draw_mesh(gpu_mesh, material, transform);
        }
        renderer.submit_render_pass(
            render_pass,
            &meshes,
            &materials,
            &shaders,
            &textures,
            &cube_maps,
            &morphable_mesh_data,
        );
    }

    if renderer.automatically_redraw {
        window.request_redraw();
    }
}
