use koi_ecs::Component;

#[derive(Clone, Component)]
pub struct AudioListener {
    pub(crate) previous_position: Option<kmath::Vec3>,
    pub(crate) previous_velocity: Option<kmath::Vec3>,
}

impl AudioListener {
    pub fn new() -> Self {
        Self {
            previous_position: None,
            previous_velocity: None,
        }
    }
}
