use koi3::*;
use koi_camera_controls::CameraControls;

fn main() {
    App::default()
        .with_resource(InitialSettings {
            color_space: kgraphics::ColorSpace::SRGB,
            window_width: 400,
            window_height: 400,
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
                    //exposure: Exposure::EV100(15.0),
                    projection_mode: ProjectionMode::Perspective {
                        field_of_view_y_radians: 22.5f32.to_radians(),
                        z_near: 0.01,
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

            /*
            world.spawn((
                Transform::new()
                    .with_position(Vec3::fill(3.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                PointLight {
                    intensity_luminous_power: 500.0,
                    color: color_temperatures::FLUORESCENT_LIGHT,
                    influence_radius: 20.0,
                },
                //Mesh::SPHERE,
                //Material::UNLIT,
            ));
            world.spawn((
                Transform::new()
                    .with_position(Vec3::new(-10.0, 0.0, 10.0))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                PointLight {
                    intensity_luminous_power: 1_000.0,
                    color: color_temperatures::LIGHTBULB,
                    influence_radius: 20.0,
                },
                //Mesh::SPHERE,
                //Material::UNLIT,
            ));
            */

            // Load a custom shader from a path
            let custom_shader = resources.get::<AssetStore<Shader>>().load(
                "examples/assets/custom_shader.glsl",
                ShaderSettings::default(),
            );

            let cube_map = resources.get::<AssetStore<CubeMap>>().load(
                "examples/assets/venice_sunset_small.hdr",
                CubeMapSettings::default(),
            );

            // Create a material that uses the custom shader
            let custom_material = resources.get::<AssetStore<Material>>().add(Material {
                shader: custom_shader,
                base_color: Color::new(0.81, 0.0, 0.0, 1.0),
                perceptual_roughness: 0.05,
                // reflectance: 0.5,
                cube_map: Some(cube_map),
                ..Default::default()
            });

            // Spawn an entity that references the custom material.
            world.spawn((Transform::new(), Mesh::SPHERE, custom_material));

            /*
            let mut materials = resources.get::<AssetStore<Material>>();
            let rows = 6;
            let columns = 6;

            let spacing = 2.0;

            for i in 0..rows {
                for j in 0..columns {
                    world.spawn((
                        Transform::new().with_position(Vec3::new(
                            j as f32 * spacing,
                            i as f32 * spacing,
                            -2.0,
                        )),
                        materials.add(Material {
                            shader: custom_shader.clone(),
                            base_color: Random::new().color(),
                            //metallic: i as f32 / rows as f32,
                            roughness: (j as f32 / columns as f32).clamp(0.01, 1.0),
                            ..Default::default()
                        }),
                        Mesh::SPHERE,
                    ));
                }
            }
            */

            |event, _world, resources| {
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
