//! This example demonstrates a pattern that may be useful for gamejams
//! or quick prototypes.
//!
//! Within the setup and run function gamestate can be initialized
//! and then used later in the loop without explicitly
//! storing it in a state struct.

use koi3::*;
use koi_camera_controls::CameraControls;

fn setup_and_run(
    world: &mut World,
    _resources: &mut Resources,
) -> impl FnMut(&Event, &mut World, &mut Resources) {
    // Initial setup logic goes here.
    world.spawn((
        Transform::new().with_position(Vec3::Z * 3.0),
        Camera {
            clear_color: Some(Color::ORANGE),
            ..Default::default()
        },
        CameraControls::new(),
    ));

    let entity = world.spawn((Transform::new(), Mesh::CUBE, Material::UNLIT));

    // Then this closure runs per-event but can access the variables declared
    // during the setup step.
    move |_event, world, resources| {
        let mut movement = Vec3::ZERO;

        let input = resources.get_mut::<Input>();
        if input.key(Key::Left) {
            movement -= Vec3::X;
        }
        if input.key(Key::Right) {
            movement += Vec3::X;
        }
        if input.key(Key::Up) {
            movement += Vec3::Y;
        }
        if input.key(Key::Down) {
            movement -= Vec3::Y;
        }

        world.get::<&mut Transform>(entity).unwrap().position +=
            movement.normalized_or_zero() * 0.01;
    }
}

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
