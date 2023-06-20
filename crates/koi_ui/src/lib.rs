use kmath::*;
use koi_assets::*;
use koi_ecs::*;
use koi_renderer::*;
use koi_resources::*;
use koi_transform::*;
pub use kui;
use kui::*;

pub fn initialize_plugin(world: &mut World) {
    let projection_matrix = projection_matrices::orthographic_gl(-1.0, 1.0, -1.0, 1.0, 0.0, 1.0);
    world.spawn((
        Transform::new(),
        Camera {
            clear_color: None,
            projection_mode: ProjectionMode::Custom(projection_matrix),
            ..Default::default()
        },
        RenderFlags::USER_INTERFACE,
    ));
}

pub struct ScreenSpaceUI<UIState> {
    drawer: kui::Drawer,
    context: StandardContext<UIState>,
    root_widget: Box<dyn kui::Widget<UIState, StandardContext<UIState>>>,
    ui_material: Handle<Material>,
    ui_mesh: Handle<Mesh>,
}

// This is safe because
unsafe impl<UIState> Send for ScreenSpaceUI<UIState> {}
unsafe impl<UIState> Sync for ScreenSpaceUI<UIState> {}

impl<UIState: 'static> ScreenSpaceUI<UIState> {
    pub fn new(
        world: &mut World,
        resources: &Resources,
        style: StandardStyle,
        ui: impl Widget<UIState, StandardContext<UIState>> + 'static,
    ) -> Entity {
        let mut meshes = resources.get::<AssetStore<Mesh>>();
        let mut materials = resources.get::<AssetStore<Material>>();
        let mut graphics_context = &mut resources.get::<Renderer>().raw_graphics_context;

        let ui_mesh = meshes.add(Mesh::new(&mut graphics_context, MeshData::default()));
        let ui_material = materials.add(Material {
            shader: Shader::UNLIT_UI,
            ..Default::default()
        });

        let fonts = kui::Fonts::default();

        use kui::*;

        let screen_space_ui = Self {
            drawer: kui::Drawer::new(),
            context: StandardContext::new(style, Default::default(), fonts),
            root_widget: Box::new(ui),
            ui_material: ui_material.clone(),
            ui_mesh: ui_mesh.clone(),
        };

        world.spawn((
            Transform::new(),
            ui_mesh.clone(),
            ui_material.clone(),
            screen_space_ui,
            RenderFlags::USER_INTERFACE,
        ))
    }
}

pub fn draw_screen_space_uis<UIState: 'static>(world: &mut World, resources: &Resources) {
    let mut meshes = resources.get::<AssetStore<Mesh>>();
    let mut textures = resources.get::<AssetStore<Texture>>();
    let mut materials = resources.get::<AssetStore<Material>>();
    let mut graphics_context = &mut resources.get::<Renderer>().raw_graphics_context;
    let mut ui_state = resources.get::<UIState>();

    for (_, (ui, transform)) in world
        .query::<(&mut ScreenSpaceUI<UIState>, &GlobalTransform)>()
        .iter()
    {
        let window = resources.get::<kapp::Window>();
        let (window_width, window_height) = window.size();
        let ui_scale = window.scale();

        let width = window_width as f32 / ui_scale as f32;
        let height = window_height as f32 / ui_scale as f32;

        ui.context.standard_style_mut().ui_scale = ui_scale as _;
        ui.context.standard_input_mut().view_size = Vec2::new(width, height);

        let constraints = kui::MinAndMaxSize {
            min: Vec3::ZERO,
            max: Vec3::new(width, height, 10_000.0),
        };

        ui.root_widget
            .layout(&mut ui_state, &mut (), &mut ui.context, constraints);
        ui.drawer.reset();
        ui.drawer.set_view_width_height(width, height);

        ui.root_widget.draw(
            &mut ui_state,
            &mut (),
            &mut ui.context,
            &mut ui.drawer,
            Box3::new_with_min_corner_and_size(constraints.min, constraints.max),
        );

        let first_mesh_data = &ui.drawer.first_mesh;
        let mesh_data = MeshData {
            positions: first_mesh_data.positions.clone(),
            indices: first_mesh_data.indices.clone(),
            colors: first_mesh_data.colors.clone(),
            texture_coordinates: first_mesh_data.texture_coordinates.clone(),
            ..Default::default()
        };

        *meshes.get_mut(&ui.ui_mesh) = Mesh::new(&mut graphics_context, mesh_data);

        if ui.drawer.texture_atlas.changed {
            ui.drawer.texture_atlas.changed = false;

            unsafe {
                let new_texture = graphics_context.new_texture_with_bytes(
                    ui.drawer.texture_atlas.width as u32,
                    ui.drawer.texture_atlas.height as u32,
                    1,
                    &ui.drawer.texture_atlas.data,
                    koi_graphics_context::PixelFormat::R8Unorm,
                    koi_graphics_context::TextureSettings {
                        srgb: false,
                        ..Default::default()
                    },
                );

                let new_texture_handle = textures.add(Texture(new_texture));
                materials.get_mut(&ui.ui_material).base_color_texture = Some(new_texture_handle);
            }
        }
    }
}
