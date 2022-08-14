use crate::*;

use kgraphics::GraphicsContextTrait;
use koi_assets::*;
use koi_resources::Resources;
use koi_transform::GlobalTransform;

pub fn initialize_plugin(resources: &mut Resources) {
    // Initialize the graphics context.
    let mut graphics_context =
        kgraphics::GraphicsContext::new_with_settings(kgraphics::GraphicsContextSettings {
            high_resolution_framebuffer: true,
            /// How many MSAA samples the window framebuffer should have
            samples: 4,
        });

    let dimensions = 500;

    // Create the primary window
    let window = {
        let kapp_app = resources.get::<kapp::Application>();
        kapp_app
            .new_window()
            .title("Koi")
            .size(dimensions, dimensions)
            .build()
    };

    graphics_context.resize(&window, dimensions, dimensions);

    // For now this needs to be called even if it's not used.
    let _render_target =
        graphics_context.get_render_target_for_window(&window, dimensions, dimensions);

    resources.add(window);

    let mut renderer = Renderer::new(graphics_context);

    let mut shaders = AssetStore::<Shader>::new();
    initialize_constant_shader(&mut renderer, &mut shaders);

    let mut materials = AssetStore::<Material>::new();
    initialize_constant_materials(&mut materials);

    let mut meshes = AssetStore::<Mesh>::new();
    mesh_constants::initialize_constant_meshes(&mut renderer.raw_graphics_context, &mut meshes);

    resources.add(renderer);
    resources.add(shaders);
    resources.add(materials);
    resources.add(meshes);

    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_handler(koi_events::Event::Draw, draw);
}

pub fn draw(_: &koi_events::Event, world: &mut koi_ecs::World, resources: &mut Resources) {
    let mut renderer = resources.get::<Renderer>();
    let window = resources.get::<kapp::Window>();
    let gpu_meshes = resources.get::<AssetStore<Mesh>>();
    let materials = resources.get::<AssetStore<Material>>();
    let shaders = resources.get::<AssetStore<Shader>>();

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

        let mut renderables = world
            .query::<koi_ecs::Without<(&Handle<Mesh>, &Handle<Material>, &GlobalTransform), &Camera>>();

        for (_, (gpu_mesh, material, transform)) in renderables.iter() {
            //todo
            render_pass.draw_mesh(gpu_mesh, material, transform);
        }
        renderer.submit_render_pass(render_pass, &gpu_meshes, &materials, &shaders);
    }

    if renderer.automatically_redraw {
        window.request_redraw();
    }
}
