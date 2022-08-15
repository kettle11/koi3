#[derive(Clone, Debug)]
pub enum Event {
    FixedUpdate,
    PostFixedUpdate,
    Draw,
    PostDraw,
    KappEvent(kapp_platform_common::Event),
}

type Callback = Box<dyn FnMut(&Event, &mut koi_ecs::World, &mut koi_resources::Resources)>;

pub struct EventHandlers {
    universal_handlers: Vec<Callback>,
    handlers: std::collections::HashMap<std::mem::Discriminant<Event>, Vec<Callback>>,
}

impl Default for EventHandlers {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandlers {
    pub fn new() -> Self {
        Self {
            universal_handlers: Vec::new(),
            handlers: std::collections::HashMap::new(),
        }
    }

    #[inline]
    pub fn add_universal_handler(
        &mut self,
        callback: impl FnMut(&Event, &mut koi_ecs::World, &mut koi_resources::Resources) + 'static,
    ) {
        let callback = Box::new(callback);
        self.universal_handlers.push(callback);
    }

    #[inline]
    pub fn add_handler(
        &mut self,
        event_type: Event,
        callback: impl FnMut(&Event, &mut koi_ecs::World, &mut koi_resources::Resources) + 'static,
    ) {
        let callback = Box::new(callback);
        self.handlers
            .entry(std::mem::discriminant(&event_type))
            .or_insert_with(|| Vec::new())
            .push(callback);
    }

    pub fn handle_event(
        &mut self,
        event: &Event,
        world: &mut koi_ecs::World,
        resources: &mut koi_resources::Resources,
    ) {
        for handler in self.universal_handlers.iter_mut() {
            handler(event, world, resources);
        }

        if let Some(handlers) = self.handlers.get_mut(&std::mem::discriminant(event)) {
            for handler in handlers.iter_mut() {
                handler(event, world, resources);
            }
        }
    }
}
