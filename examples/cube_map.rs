use koi3::*;
use koi_camera_controls::CameraControls;

fn main() {
    App::default()
        .with_resource(InitialSettings {
            color_space: kgraphics::ColorSpace::SRGB,
            window_width: 1200,
            window_height: 1200,
            ..Default::default()
        })
        .setup_and_run(|world, resources| {
            // Spawn a camera
            world.spawn((
                Transform::new()
                    .with_position(Vec3::new(0., 0., -3.1))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                Camera {
                    clear_color: Some(Color::BLACK),
                    exposure: Exposure::PhysicalCamera {
                        aperture_f_stops: 16.0,
                        shutter_speed_seconds: 1.0 / 125.0,
                        sensitivity_iso: 100.0,
                    },

                    ..Default::default()
                },
                CameraControls::default(),
            ));

            world.spawn((
                Transform::new()
                    .with_position(Vec3::ZERO)
                    .looking_at(Vec3::new(-1., -1., 1.), Vec3::Y),
                DirectionalLight {
                    intensity_illuminance: 120_000.0,
                    color: Color::from_linear_srgb(1.0, 0.96, 0.95, 1.0),
                },
            ));

            world.spawn((Transform::new(), Mesh::SPHERE, Material::PHYSICALLY_BASED));

            let cube_map = resources
                .get::<AssetStore<CubeMap>>()
                .load("assets/venice_sunset_1k.hdr", ());

            // Create a material that uses the custom shader
            let custom_material = resources.get::<AssetStore<Material>>().add(Material {
                shader: Shader::SKYBOX,
                cube_map: Some(cube_map),
                ..Default::default()
            });
            world.spawn((Transform::new(), Mesh::CUBE_MAP_CUBE, custom_material));

            |event, world, resources| {
                match event {
                    Event::FixedUpdate => {
                        // When a key is pressed reload all shaders that were loaded from a path.
                        if resources.get_mut::<Input>().key_down(Key::Space) {
                            resources.get::<AssetStore<Shader>>().reload();
                        }
                    }
                    _ => {}
                }
            }
        });
}
