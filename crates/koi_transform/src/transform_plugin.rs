pub struct TransformHelper {
    command_buffer: koi_ecs::CommandBuffer,
}

pub fn add_global_transform(
    _event: &koi_events::Event,
    world: &mut koi_ecs::World,
    resources: &mut koi_resources::Resources,
) {
    // TODO: Only add these initial global transforms to root nodes.
    // TODO: Update all descendent transforms
    let transform_helper = resources.get_mut::<TransformHelper>();
    transform_helper.command_buffer.clear();

    {
        for (entity, transform) in world.query::<&crate::Transform>().iter() {
            transform_helper
                .command_buffer
                .insert_one(entity, crate::GlobalTransform(transform.clone()))
        }
    }
    transform_helper.command_buffer.run_on(world);
}

pub fn initialize_plugin(resources: &mut koi_resources::Resources) {
    resources.add(TransformHelper {
        command_buffer: koi_ecs::CommandBuffer::new(),
    });
    let event_handlers = resources.get_mut::<koi_events::EventHandlers>();
    event_handlers.add_handler(koi_events::Event::PostFixedUpdate, add_global_transform);
}
