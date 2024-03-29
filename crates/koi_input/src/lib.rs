pub type Input = kapp::StateTracker;

pub fn initialize_plugin(resources: &mut koi_resources::Resources) {
    let event_handlers = resources.get_mut::<koi_events::EventHandlers>();
    event_handlers.add_universal_handler(|event, _, resources| {
        if let koi_events::Event::KappEvent(event) = event {
            resources.get_mut::<Input>().handle_event(event);
        }
    });
    event_handlers.add_handler(koi_events::Event::PostFixedUpdate, |_, _, resources| {
        let input = resources.get_mut::<Input>();
        input.clear();
    });

    resources.add(Input::new());
}
