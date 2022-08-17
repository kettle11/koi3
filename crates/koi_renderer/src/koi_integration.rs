use crate::*;

use kgraphics::GraphicsContextTrait;
use koi_assets::*;
use koi_resources::Resources;
use koi_transform::GlobalTransform;

pub struct InitialSettings {
    pub name: String,
    pub window_width: usize,
    pub window_height: usize,
    pub color_space: kgraphics::ColorSpace,
}

impl Default for InitialSettings {
    fn default() -> Self {
        Self {
            name: "Koi".into(),
            window_width: 800,
            window_height: 800,
            color_space: kgraphics::ColorSpace::SRGB,
        }
    }
}

pub fn initialize_plugin(resources: &mut Resources) {
    let initial_settings = resources.remove::<InitialSettings>().unwrap_or_default();

    // Initialize the graphics context.
    let mut graphics_context =
        kgraphics::GraphicsContext::new_with_settings(kgraphics::GraphicsContextSettings {
            high_resolution_framebuffer: true,
            /// How many MSAA samples the window framebuffer should have
            samples: 4,
            color_space: Some(initial_settings.color_space),
        });

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

    graphics_context.resize(&window, window_width as _, window_height as _);

    // For now this needs to be called even if it's not used.
    let _render_target = graphics_context.get_render_target_for_window(
        &window,
        window_width as _,
        window_height as _,
    );

    resources.add(window);

    let mut renderer = Renderer::new(graphics_context, initial_settings.color_space);

    initialize_shaders(&mut renderer, resources);
    let materials = initialize_materials();
    let meshes = initialize_meshes(&mut renderer.raw_graphics_context);
    let textures = initialize_textures(&mut renderer);

    resources.add(renderer);
    resources.add(materials);
    resources.add(meshes);
    resources.add(textures);

    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_handler(koi_events::Event::Draw, draw);
}

pub fn draw(_: &koi_events::Event, world: &mut koi_ecs::World, resources: &mut Resources) {
    let window = resources.get::<kapp::Window>();
    let mut meshes = resources.get::<AssetStore<Mesh>>();
    let mut materials = resources.get::<AssetStore<Material>>();
    let mut shaders = resources.get::<AssetStore<Shader>>();
    let mut textures = resources.get::<AssetStore<Texture>>();

    meshes.finalize_asset_loads(resources);
    materials.finalize_asset_loads(resources);
    shaders.finalize_asset_loads(resources);
    textures.finalize_asset_loads(resources);

    let mut renderer = resources.get::<Renderer>();

    #[allow(clippy::significant_drop_in_scrutinee)]
    for mesh in meshes.get_dropped_assets() {
        if let Some(gpu_mesh) = mesh.gpu_mesh {
            gpu_mesh.delete(&mut renderer.raw_graphics_context);
        }
    }
    materials.cleanup_dropped_assets();
    // TODO: Properly deallocate dropped programs.
    shaders.cleanup_dropped_assets();

    let (window_width, window_height) = window.size();
    renderer
        .raw_graphics_context
        .resize(&*window, window_width, window_height);

    let mut cameras = world.query::<(&GlobalTransform, &Camera)>();
    for (_, (camera_transform, camera)) in cameras.iter() {
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

        let mut renderables = world
            .query::<koi_ecs::Without<(&Handle<Mesh>, &Handle<Material>, &GlobalTransform), &Camera>>();

        for (_, (gpu_mesh, material, transform)) in renderables.iter() {
            //todo
            render_pass.draw_mesh(gpu_mesh, material, transform);
        }
        renderer.submit_render_pass(render_pass, &meshes, &materials, &shaders, &textures);
    }

    if renderer.automatically_redraw {
        window.request_redraw();
    }
}
