pub trait InterpolateTrait {
    fn interpolate(&self, other: &Self, amount: f32) -> Self;
}

pub fn smooth_step(amount: f32) -> f32 {
    amount * amount * (3.0 - 2.0 * amount)
}

pub struct Animation<T: InterpolateTrait> {
    pub key_frames: Vec<KeyFrame<T>>,
}

impl<T: InterpolateTrait + 'static> koi_assets::AssetTrait for Animation<T> {
    type Settings = ();
}

impl<T: InterpolateTrait> Default for Animation<T> {
    fn default() -> Self {
        Animation {
            key_frames: Vec::new(),
        }
    }
}

pub struct KeyFrame<T: InterpolateTrait> {
    pub timestamp: f32,
    pub value: T,
}

pub struct AnimationPlayer<T: InterpolateTrait + 'static> {
    pub time: f32,
    pub animation: koi_assets::Handle<Animation<T>>,
}

impl<T: InterpolateTrait + 'static> AnimationPlayer<T> {
    pub fn advance(&mut self, animation: &Animation<T>, t: &mut T, amount_seconds: f32) {
        // TODO: This could be optimized by retaining the last index and avoiding
        // the binary search in set_time.
        self.set_time(animation, t, self.time + amount_seconds)
    }

    pub fn set_time(&mut self, animation: &Animation<T>, t: &mut T, time: f32) {
        if animation.key_frames.len() < 2 {
            return;
        }
        let time = time % animation.key_frames.last().map_or(0.0, |k| k.timestamp);
        let index = match animation
            .key_frames
            .binary_search_by(|v| v.timestamp.partial_cmp(&time).unwrap())
        {
            Ok(i) => i,
            Err(i) => i,
        };
        let next_index = index + 1 % animation.key_frames.len();
        let k0 = &animation.key_frames[index];
        let k1 = &animation.key_frames[next_index];

        let amount = (k1.timestamp - k0.timestamp) / (time - k0.timestamp);
        *t = k0.value.interpolate(&k1.value, amount);
        self.time = time;
    }
}

pub fn run_animations<T: InterpolateTrait + 'static + Sync + Send>(
    world: &koi_ecs::World,
    animations: &koi_assets::AssetStore<Animation<T>>,
    time: &koi_time::Time,
) {
    let amount_seconds = time.delta_seconds_f64 as f32;
    for (_, (t, animation_player)) in world.query::<(&mut T, &mut AnimationPlayer<T>)>().iter() {
        let animation = animations.get(&animation_player.animation);
        animation_player.advance(animation, t, amount_seconds);
    }
}

pub fn initialize_animation_plugin<T: InterpolateTrait + 'static + Sync + Send>(
    resources: &mut koi_resources::Resources,
) {
    resources.add(koi_assets::AssetStore::<Animation<T>>::new(
        Animation::default(),
    ));
    let event_handlers = resources.get_mut::<koi_events::EventHandlers>();
    event_handlers.add_handler(koi_events::Event::Draw, |_event, world, resources| {
        let mut animations = resources.get::<koi_assets::AssetStore<Animation<T>>>();
        let mut time = resources.get::<koi_time::Time>();
        run_animations(world, &mut animations, &mut time)
    })
}
