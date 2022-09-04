use koi_ecs::WorldClonableTrait;

pub trait InterpolateTrait {
    fn interpolate(&self, other: &Self, amount: f32) -> Self;
}

pub mod animation_curves {
    pub fn smooth_step(amount: f32) -> f32 {
        amount * amount * (3.0 - 2.0 * amount)
    }

    pub fn linear(amount: f32) -> f32 {
        amount
    }

    pub fn step(_amount: f32) -> f32 {
        0.0
    }
}

pub struct AnimationPlayer {
    pub playing_animations: Vec<PlayingAnimation>,
}

impl WorldClonableTrait for AnimationPlayer {
    fn clone_with_context(&self, entity_migrator: &koi_ecs::EntityMigrator) -> Self {
        Self {
            playing_animations: self
                .playing_animations
                .iter()
                .map(|p| PlayingAnimation {
                    time: p.time,
                    entity_mapping: p
                        .entity_mapping
                        .iter()
                        .filter_map(|e| e.map(|e| entity_migrator.migrate(e)))
                        .collect(),
                    animation: p.animation.clone(),
                })
                .collect(),
        }
    }
}

impl AnimationPlayer {
    pub fn advance_time(
        &mut self,
        world: &koi_ecs::World,
        animations: &koi_assets::AssetStore<Animation>,
        time_seconds: f32,
    ) {
        for playing_animation in self.playing_animations.iter_mut() {
            let animation = animations.get(&playing_animation.animation);

            playing_animation.time += time_seconds;
            playing_animation.time %= animation.length;

            for animation_clip in animation.animation_clips.iter() {
                if let Some(Some(entity)) = playing_animation
                    .entity_mapping
                    .get(animation_clip.entity_mapping_index)
                {
                    if let Ok(entity) = world.entity(*entity) {
                        animation_clip.animate_entity(&entity, playing_animation.time);
                    }
                }
            }
        }
    }
}

pub struct PlayingAnimation {
    pub time: f32,
    // pub weight: f32,
    /// Let the [Animation] know which [koi_ecs::Entity]s to animate.
    pub entity_mapping: Vec<Option<koi_ecs::Entity>>,
    pub animation: koi_assets::Handle<Animation>,
}

/// An [Animation] represents a group of properties on [koi_ecs::Entity]s that are all animated together.
pub struct Animation {
    pub animation_clips: Vec<AnimationClip>,
    pub length: f32,
}

impl Animation {
    pub fn new(animation_clips: Vec<AnimationClip>) -> Self {
        let mut length: f32 = 0.0;
        for animation_clip in animation_clips.iter() {
            length = length.max(animation_clip.typed_animation_clip.length());
        }

        Self {
            animation_clips,
            length,
        }
    }
}

impl koi_assets::AssetTrait for Animation {
    type Settings = ();
}

/// Data that specifies how to animate a single value on a single [koi_ecs::Entity]
pub struct AnimationClip {
    pub animation_curve: fn(f32) -> f32,
    pub entity_mapping_index: usize,
    pub typed_animation_clip: Box<dyn TypedAnimationClipTrait>,
}

impl AnimationClip {
    pub fn animate_entity(&self, entity: &koi_ecs::EntityRef, time: f32) {
        self.typed_animation_clip
            .animate_entity(entity, self.animation_curve, time);
    }
}

pub struct TypedAnimationClip<T> {
    pub set_property: for<'a> fn(&'a koi_ecs::EntityRef, v0: &T, v1: &T, t: f32),
    pub key_frames: Vec<f32>,
    pub values: Vec<T>,
}

pub trait TypedAnimationClipTrait {
    fn animate_entity(
        &self,
        entity: &koi_ecs::EntityRef,
        animation_curve: fn(f32) -> f32,
        time: f32,
    );
    fn length(&self) -> f32;
    fn key_frame_count(&self) -> usize;
}

impl<T> TypedAnimationClipTrait for TypedAnimationClip<T> {
    fn length(&self) -> f32 {
        self.key_frames.last().copied().unwrap_or(0.0)
    }

    fn key_frame_count(&self) -> usize {
        self.key_frames.len()
    }
    fn animate_entity(
        &self,
        entity: &koi_ecs::EntityRef,
        animation_curve: fn(f32) -> f32,
        time: f32,
    ) {
        let mut index = match self
            .key_frames
            .binary_search_by(|k| k.partial_cmp(&time).unwrap())
        {
            Ok(i) | Err(i) => i,
        };

        if index == self.key_frames.len() {
            index = 0;
        }

        if index != 0 {
            index -= 1;
        }

        let k0 = &self.key_frames[index];
        let v0 = &self.values[index];

        let next_index = index + 1;

        // Some animations have a single keyframe to set a constant value during the
        // animation: https://github.com/KhronosGroup/glTF/issues/1597
        if let Some(k1) = self.key_frames.get(next_index) {
            let v1 = &self.values[next_index];
            let amount = ((time - k0) / (k1 - k0)).clamp(0.0, 1.0);
            let amount = (animation_curve)(amount);
            (self.set_property)(entity, v0, v1, amount)
        } else {
            (self.set_property)(entity, v0, v0, 0.0)
        }
    }
}

pub fn initialize_animation_plugin<T: InterpolateTrait + 'static + Sync + Send>(
    resources: &mut koi_resources::Resources,
) {
    resources.add(koi_assets::AssetStore::<Animation>::new(Animation {
        animation_clips: Vec::new(),
        length: 0.0,
    }));
    let event_handlers = resources.get_mut::<koi_events::EventHandlers>();
    event_handlers.add_handler(koi_events::Event::Draw, |_event, world, resources| {
        let animations = resources.get::<koi_assets::AssetStore<Animation>>();
        let time = resources.get::<koi_time::Time>();
        let amount_seconds = time.draw_delta_seconds as f32;
        for (_, animation_player) in world.query::<&mut AnimationPlayer>().iter() {
            animation_player.advance_time(world, &animations, amount_seconds)
        }
    });
    resources
        .get::<koi_ecs::WorldCloner>()
        .register_clone_type::<AnimationPlayer>();
}
