use koi3::{world_cloner::WorldCloner, *};
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
                    exposure: Exposure::EV100(20.0),
                    ..Default::default()
                },
                CameraControls::default(),
            ));

            /*
            world.spawn((
                Transform::new()
                    .with_position(Vec3::new(2.5, 5.0, 0.1))
                    .looking_at(Vec3::ZERO, Vec3::Y),
                PointLight {
                    intensity_lumens: 450.0,
                    color: color_temperatures::LIGHTBULB,
                    influence_radius: 20.0,
                },
                Mesh::SPHERE,
                Material::UNLIT,
            ));
            */

            println!("LUMINANCE HERE: {:?}", 2.0f32.powf(15.0) * (12.5 / 100.0));

            /*
            world.spawn((
                Transform::new()
                    .with_position(Vec3::ZERO)
                    .looking_at(Vec3::new(-1., -1., 1.), Vec3::Y),
                DirectionalLight {
                    intensity_illuminance: 120_000.0,
                    color: Color::from_linear_srgb(1.0, 0.96, 0.95, 1.0),
                },
            ));
            */

            world.spawn((Transform::new(), Mesh::SPHERE, Material::PHYSICALLY_BASED));

            let skybox_spherical_harmonics = resources
                .get::<AssetStore<Shader>>()
                .load("assets/skybox_sh.glsl", ShaderSettings::default());

            let cube_map = resources.get::<AssetStore<CubeMap>>().load(
                "assets/venice_sunset_1k.hdr",
                CubeMapSettings {
                    luminance_of_brightest_pixel: Some(luminance::SUN_AT_NOON),
                },
            );

            // Create a material that uses the custom shader
            let custom_material = resources.get::<AssetStore<Material>>().add(Material {
                shader: Shader::SKYBOX,
                cube_map: Some(cube_map.clone()),
                ..Default::default()
            });

            // Create a material that uses the custom shader
            let skybox_spherical_harmonics =
                resources.get::<AssetStore<Material>>().add(Material {
                    shader: skybox_spherical_harmonics.clone(),
                    cube_map: Some(cube_map),
                    ..Default::default()
                });
            let skybox_entity = world.spawn((
                Transform::new(),
                Mesh::CUBE_MAP_CUBE,
                custom_material.clone(),
            ));

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
                            shader: Shader::PHYSICALLY_BASED,
                            // base_color: Random::new().color(),
                            // metallic: i as f32 / rows as f32,
                            perceptual_roughness: (j as f32 / columns as f32).clamp(0.01, 1.0),
                            ..Default::default()
                        }),
                        Mesh::SPHERE,
                    ));
                }
            }

            let mut prefabs = resources.get::<AssetStore<Prefab>>();
            let prefab_handle = prefabs.load("assets/cat_statue/scene.gltf", ());
            /*
            struct Rotator;

            let mut parent_cube = world.spawn((
                Rotator,
                Mesh::CUBE,
                Material::PHYSICALLY_BASED,
                Transform::new(),
            ));

            for _ in 0..10 {
                let child_cube = world.spawn((
                    Mesh::CUBE,
                    Material::PHYSICALLY_BASED,
                    Transform::new().with_position(Vec3::Y * 6.0),
                ));

                let _ = world.set_parent(parent_cube, child_cube);
                parent_cube = child_cube;
            }
            */

            let mut cloned = false;
            move |event, world, resources| {
                {
                    if !cloned {
                        let mut prefabs = resources.get::<AssetStore<Prefab>>();

                        if prefabs.currently_loading() == 0 {
                            let prefab = prefabs.get_mut(&prefab_handle);

                            let size = 40;
                            let spacing = 4.0;

                            for i in 0..size {
                                for j in 0..size {
                                    let mut world_cloner = resources.get::<WorldCloner>();
                                    prefab.spawn_with_transform(
                                        world,
                                        &mut world_cloner,
                                        Transform::new().with_position(
                                            Vec3::new(i as f32, 0.0, j as f32) * spacing,
                                        ),
                                    );
                                }
                            }

                            //  world_cloner.clone_world(&mut prefab.0, world);
                            cloned = true;
                        }
                    }
                }
                match event {
                    Event::FixedUpdate => {
                        // When a key is pressed reload all shaders that were loaded from a path.
                        if resources.get::<Input>().key_down(Key::Space) {
                            resources.get::<AssetStore<Shader>>().reload();

                            let mut skybox_material =
                                world.get::<&mut Handle<Material>>(skybox_entity).unwrap();
                            if *skybox_material == custom_material {
                                *skybox_material = skybox_spherical_harmonics.clone();
                            } else {
                                *skybox_material = custom_material.clone();
                            }
                        }

                        /*
                        for (_rotator, (_, transform)) in
                            world.query::<(&mut Rotator, &mut Transform)>().iter()
                        {
                            transform.rotation =
                                Quat::from_angle_axis(0.05, Vec3::X) * transform.rotation
                        }
                        */
                    }
                    _ => {}
                }
            }
        });
}
