use koi_events::EventHandlers;

use crate::*;

pub struct App {
    pub world: crate::World,
    pub resources: Resources,
}

impl Default for App {
    fn default() -> Self {
        let mut resources = Resources::new();
        resources.add(EventHandlers::new());
        resources.add(Time::new());
        Self {
            world: crate::World::new(),
            resources,
        }
    }
}

impl App {
    #[cfg(feature = "kapp")]
    #[inline]
    pub fn run(mut self, f: impl FnMut(&Event, &mut crate::World, &mut Resources) + 'static) {
        self.resources
            .get_mut::<EventHandlers>()
            .add_universal_handler(f);
        self.run_inner();
    }

    #[cfg(feature = "kapp")]
    fn run_inner(mut self) {
        let (kapp_app, kapp_event_loop) = kapp::initialize();

        self.resources.add(kapp_app);

        self.add_standard_plugins();

        kapp_event_loop.run(move |kapp_event| {
            self.handle_event(Event::KappEvent(kapp_event.clone()));
            self.run_fixed_update();

            match kapp_event {
                kapp_platform_common::Event::WindowCloseRequested { .. } => {
                    let kapp_app = self.resources.get_mut::<kapp::Application>();
                    kapp_app.quit()
                }
                kapp_platform_common::Event::Draw { .. } => {
                    self.handle_event(Event::Draw);
                    self.handle_event(Event::PostDraw);
                }
                kapp_platform_common::Event::Quit => {
                    // klog::log!("ABOUT TO QUIT");
                    // ktasks::shutdown_worker_threads();
                }
                _ => {}
            }
        });
    }

    pub fn handle_event(&mut self, event: Event) {
        // This funky memory-swap approach allows `EventHandlers` to be part of `Resources`
        let event_handlers = self.resources.get_mut::<EventHandlers>();
        let mut temp_event_handlers = EventHandlers::new();
        std::mem::swap(&mut temp_event_handlers, event_handlers);
        temp_event_handlers.handle_event(&event, &mut self.world, &mut self.resources);
        let event_handlers = self.resources.get_mut::<EventHandlers>();
        std::mem::swap(&mut temp_event_handlers, event_handlers);
    }

    /// This is called automatically when using `run`.
    /// But if you're running your own server you may want to use this.
    pub fn run_fixed_update(&mut self) {
        // Measure elapsed time since last event and add it to
        // the total time counter.
        self.resources.get_mut::<Time>().update();

        while self.resources.get_mut::<Time>().fixed_update_ready() {
            self.handle_event(Event::FixedUpdate);
            self.handle_event(Event::PostFixedUpdate);
        }
    }

    fn add_standard_plugins(&mut self) {
        #[cfg(feature = "koi_renderer")]
        koi_renderer::initialize_plugin(&mut self.resources);
        #[cfg(feature = "koi_input")]
        koi_input::initialize_plugin(&mut self.resources);
        #[cfg(feature = "koi_camera_controls")]
        koi_camera_controls::initialize_plugin(&mut self.resources);

        koi_transform::transform_plugin::initialize_plugin(&mut self.resources);
    }
}
