use koi3::*;

fn main() {
    App::default().setup_and_run(|world, resources| {
        world.spawn((
            Transform::new().with_position(Vec3::Z * 2.0),
            Camera {
                clear_color: Some(Color::BLACK),
                exposure: Exposure::EV100(6.0),
                ..Default::default()
            },
            koi_camera_controls::CameraControls::new(),
        ));

        world.spawn((
            Transform::new()
                .with_position(Vec3::fill(10.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            DirectionalLight::OFFICE_LIGHTING,
        ));

        let new_texture = resources.get::<AssetStore<Texture>>().load(
            "examples/assets/test.JPG",
            kgraphics::TextureSettings::default(),
        );
        let custom_shader = resources.get::<AssetStore<Shader>>().load(
            "examples/assets/custom_shader.glsl",
            ShaderSettings::default(),
        );

        let new_material = resources.get::<AssetStore<Material>>().add(Material {
            shader: custom_shader,
            base_color_texture: Some(new_texture),
            perceptual_roughness: 0.05,
            ..Default::default()
        });

        world.spawn((Transform::new(), Mesh::CUBE, new_material));

        // This function will run for major events liked a FixedUpdate occuring
        // and for any input events from the application.
        // See [koi::Event]
        |event, world, resources| match event {
            Event::FixedUpdate => {
                // When a key is pressed reload all shaders that were loaded from a path.
                if resources.get_mut::<Input>().key_down(Key::Space) {
                    resources.get::<AssetStore<Shader>>().reload();
                }
            }
            _ => {}
        }
    });
}
