//! This example demonstrates how to use a [MorphableMesh] animation.
//! These animations blend a mesh between sets of vertices.
use koi3::*;
use koi_camera_controls::CameraControls;

fn main() {
    App::default().setup_and_run(|world, resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 5.0),
            Camera {
                clear_color: Some(Color::ORANGE),
                ..Default::default()
            },
            CameraControls::new(),
        ));

        // TODO: This setup is rather inelegant. How could it be improved?
        let (base_mesh, mesh_data, squished) = {
            let mut meshes = resources.get::<AssetStore<Mesh>>();
            let graphics = &mut resources.get::<Renderer>().raw_graphics_context;

            let mesh_data = uv_sphere(10, 10, Vec2::ONE);
            let base_mesh = meshes.add(Mesh::new(graphics, mesh_data.clone()));
            let mut squished = mesh_data.clone();
            squished.apply_transform(Transform::new().with_scale(Vec3::new(1.0, 0.5, 1.0)));
            (base_mesh, mesh_data, squished)
        };
        let morphable_mesh_data =
            MorphableMeshData::new(resources, mesh_data, &[squished.clone(), squished]);
        let morphable_mesh_data = resources
            .get::<AssetStore<MorphableMeshData>>()
            .add(morphable_mesh_data);

        let morph_material = resources.get::<AssetStore<Material>>().add(Material {
            shader: Shader::PHYSICALLY_BASED_WITH_MORPH,
            morph_weights: vec![0.5],
            morphable_mesh_data: Some(morphable_mesh_data),
            ..Default::default()
        });

        world.spawn((Transform::new(), morph_material, base_mesh));

        move |event, _world, _resources| match event {
            Event::FixedUpdate => {}
            _ => {}
        }
    });
}
