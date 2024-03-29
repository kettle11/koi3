use koi_ecs::*;

pub struct TransformHelper {
    command_buffer: koi_ecs::CommandBuffer,
}

fn add_global_transform_recursive(
    world: &koi_ecs::World,
    commands: &mut koi_ecs::CommandBuffer,
    parent_matrix: kmath::Mat4,
    entity: koi_ecs::Entity,
) -> Option<()> {
    // TODO: Right now this does not handle cases where intermediate children
    // do not have a Transform.
    // Should this continue to percolate the parent transform even in those cases?

    let transform = world.get::<&crate::Transform>(entity).ok()?;
    let new_matrix = parent_matrix * transform.local_to_world();
    let new_transform = crate::Transform::from_mat4(new_matrix);
    commands.insert_one(entity, crate::GlobalTransform(new_transform));
    for child in world.iterate_children(entity) {
        add_global_transform_recursive(world, commands, new_matrix, child);
    }

    Some(())
}

pub fn update_global_transforms(
    _event: &koi_events::Event,
    world: &mut koi_ecs::World,
    resources: &mut koi_resources::Resources,
) {
    let transform_helper = resources.get_mut::<TransformHelper>();
    transform_helper.command_buffer.clear();

    {
        let mut query = world.query::<koi_ecs::Without<&crate::Transform, &Child>>();
        for (entity, _transform) in query.iter() {
            add_global_transform_recursive(
                world,
                &mut transform_helper.command_buffer,
                kmath::Mat4::IDENTITY,
                entity,
            );
        }
    }
    transform_helper.command_buffer.run_on(world);
}

pub fn initialize_plugin(resources: &mut koi_resources::Resources) {
    let world_cloner = resources.get_mut::<WorldCloner>();
    world_cloner.register_clone_type::<crate::Transform>();
    world_cloner.register_clone_type::<crate::GlobalTransform>();

    resources.add(TransformHelper {
        command_buffer: koi_ecs::CommandBuffer::new(),
    });
    let event_handlers = resources.get_mut::<koi_events::EventHandlers>();
    event_handlers.add_handler(koi_events::Event::PostFixedUpdate, update_global_transforms);
    koi_animation::initialize_animation_plugin::<crate::Transform>(resources);
}
