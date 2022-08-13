//! This example demonstrates a pattern that may be useful for gamejams
//! or quick prototypes.
//!
//! Within the setup and run function gamestate can be initialized
//! and then used later in the loop without explicitly
//! storing it in state struct.
use koi3::*;

pub struct GameFunction(Box<dyn FnMut(&Event, &mut World, &mut Resources)>);

fn main() {
    App::default().run(|event, world, resources| {
        if resources.try_get::<GameFunction>().is_none() {
            let f = GameFunction(Box::new(setup_and_run(world, resources)));
            resources.add(f);
        }
        let mut game_function = resources.remove::<GameFunction>().unwrap();
        (game_function.0)(event, world, resources);
        resources.add(game_function);
    });
}

fn setup_and_run(
    world: &mut World,
    _resources: &mut Resources,
) -> impl FnMut(&Event, &mut World, &mut Resources) {
    // Initial setup logic goes here.
    world.spawn((
        Transform::new(),
        Camera {
            clear_color: Some(Color::ORANGE),
            projection_mode: ProjectionMode::Orthographic {
                height: 4.0,
                z_near: -5.0,
                z_far: 5.0,
            },
            ..Default::default()
        },
    ));

    let entity = world.spawn((Transform::new(), Mesh::VERTICAL_QUAD, Material::TEST));

    // Then this closure runs per-event but can access the variables declared
    // during the setup step.
    move |event, world, _resources| {
        let mut movement = Vec3::ZERO;
        match event {
            Event::KappEvent(kapp_platform_common::Event::KeyDown {
                key: kapp_platform_common::Key::Left,
                ..
            }) => {
                movement -= Vec3::X;
            }
            Event::KappEvent(kapp_platform_common::Event::KeyDown {
                key: kapp_platform_common::Key::Right,
                ..
            }) => {
                movement += Vec3::X;
            }
            Event::KappEvent(kapp_platform_common::Event::KeyDown {
                key: kapp_platform_common::Key::Up,
                ..
            }) => {
                movement += Vec3::Y;
            }
            Event::KappEvent(kapp_platform_common::Event::KeyDown {
                key: kapp_platform_common::Key::Down,
                ..
            }) => {
                movement -= Vec3::Y;
            }
            _ => {}
        }

        world.get::<&mut Transform>(entity).unwrap().position += movement;
    }
}
