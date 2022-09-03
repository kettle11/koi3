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

        let mut s = Self {
            world: crate::World::new(),
            resources,
        };
        s.setup_world_cloner();
        s
    }
}

impl App {
    pub fn setup_world_cloner(&mut self) {
        let mut world_cloner = WorldCloner::new();
        world_cloner.register_clone_type::<Child>();
        world_cloner.register_clone_type::<Parent>();
        self.resources.add(world_cloner);
    }

    #[inline]
    pub fn with_resource<Resource: 'static>(mut self, resource: Resource) -> Self {
        self.resources.add(resource);
        self
    }

    #[cfg(feature = "kapp")]
    #[inline]
    pub fn setup_and_run<
        Setup: FnMut(&mut crate::World, &mut Resources) -> Run + 'static,
        Run: FnMut(&Event, &mut crate::World, &mut Resources) + 'static,
    >(
        self,
        mut setup: Setup,
    ) {
        self.run(move |event, world, resources| {
            if resources.try_get::<Box<Run>>().is_none() {
                let f = Box::new(setup(world, resources));
                resources.add(f);
            }
            let mut game_function = resources.remove::<Box<Run>>().unwrap();
            (game_function)(event, world, resources);
            resources.add(game_function);
        });
    }

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
        // TODO: This should also run for headless builds.
        ktasks::create_workers_with_count(1);

        let (kapp_app, kapp_event_loop) = kapp::initialize();

        self.resources.add(kapp_app);

        // Spawning this as a task allows setup events to be asynchronous,
        // which is required for WebGPU.
        let plugin_setup_done = ktasks::spawn_local(self.add_standard_plugins());
        plugin_setup_done.run();
        let mut app: Option<App> = None;

        kapp_event_loop.run(move |kapp_event| {
            if let Some(app) = &mut app {
                app.handle_event(Event::KappEvent(kapp_event.clone()));
                app.run_fixed_update();

                match kapp_event {
                    kapp_platform_common::Event::WindowCloseRequested { .. } => {
                        let kapp_app = app.resources.get_mut::<kapp::Application>();
                        kapp_app.quit()
                    }
                    kapp_platform_common::Event::Draw { .. } => {
                        app.resources.get_mut::<Time>().update_draw();
                        app.handle_event(Event::Draw);
                        app.handle_event(Event::PostDraw);
                    }
                    kapp_platform_common::Event::Quit => {
                        ktasks::shutdown_worker_threads();
                    }
                    _ => {}
                }
            } else {
                ktasks::run_current_thread_tasks();
                app = plugin_setup_done.get_result();
                if app.is_some() {
                    klog::log!("Loading complete");
                }
            }
        });
    }

    pub fn handle_event(&mut self, event: Event) {
        ktasks::run_current_thread_tasks();
        ktasks::run_tasks_unless_there_are_workers();

        // This funky memory-swap approach allows `EventHandlers` to be part of `Resources`
        let event_handlers = self.resources.get_mut::<EventHandlers>();
        let mut temp_event_handlers = EventHandlers::new();
        core::mem::swap(&mut temp_event_handlers, event_handlers);
        temp_event_handlers.handle_event(&event, &mut self.world, &mut self.resources);
        let event_handlers = self.resources.get_mut::<EventHandlers>();
        core::mem::swap(&mut temp_event_handlers, event_handlers);
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

    async fn add_standard_plugins(mut self) -> Self {
        koi_prefabs::initialize_plugin(&mut self.resources);

        #[cfg(feature = "koi_renderer")]
        koi_renderer::initialize_plugin(&mut self.resources).await;
        #[cfg(feature = "koi_input")]
        koi_input::initialize_plugin(&mut self.resources);
        #[cfg(feature = "koi_camera_controls")]
        koi_camera_controls::initialize_plugin(&mut self.resources);

        #[cfg(feature = "koi_audio")]
        koi_audio::initialize_plugin(&mut self.resources);

        koi_transform::transform_plugin::initialize_plugin(&mut self.resources);
        self
    }
}
