use koi_ecs::Component;

use crate::CubeMap;

/// Spawn a [LightProbe] in a [World] to change the lighting received by objects.
/// For now only one [LightProbe] is active at a time.
#[derive(Clone, Component)]
pub struct LightProbe {
    pub source: koi_assets::Handle<CubeMap>,
}
