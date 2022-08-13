#[derive(Clone)]
pub enum LightMode {
    /// For large light sources that effect an entire environment, like the sun.
    Directional,
    /// For light sources that emit from a point, like a lamp.
    Point { radius: f32 },
}

#[derive(Clone)]
pub struct Light {
    pub intensity: f32,
    pub light_mode: LightMode,
    pub ambient_light_amount: f32,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            light_mode: LightMode::Directional,
            ambient_light_amount: 0.0,
        }
    }
}
