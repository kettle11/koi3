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

    let shaders = initialize_shaders(&mut renderer);
    let materials = initialize_materials();
    let meshes = initialize_meshes(&mut renderer.raw_graphics_context);
    let textures = initialize_textures(&mut renderer);

    resources.add(renderer);
    resources.add(shaders);
    resources.add(materials);
    resources.add(meshes);
    resources.add(textures);

    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_handler(koi_events::Event::Draw, draw);
}

pub fn draw(_: &koi_events::Event, world: &mut koi_ecs::World, resources: &mut Resources) {
    let mut renderer = resources.get::<Renderer>();
    let window = resources.get::<kapp::Window>();
    let mut meshes = resources.get::<AssetStore<Mesh>>();
    let mut materials = resources.get::<AssetStore<Material>>();
    let mut shaders = resources.get::<AssetStore<Shader>>();
    let textures = resources.get::<AssetStore<Texture>>();

    // Todo: his needs to be changed to work with async.
    while let Some((path, handle)) = shaders.need_loading.pop() {
        match std::fs::read_to_string(&path) {
            Ok(shader_string) => {
                match renderer.new_shader(&shader_string, ShaderSettings::default()) {
                    Ok(shader) => {
                        shaders.replace(&handle, shader);
                    }
                    Err(e) => println!("Error compiling shader: {:?}", e),
                }
            }
            Err(_) => println!("Could not load shader from path: {:?}", path),
        }
    }

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
