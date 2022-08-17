// Koi's lighting model is heavily based on this excellent document: https://google.github.io/filament/Filament.html#lighting

/// For light sources that emit from a point, like a lightbulb.
pub struct PointLight {
    pub luminous_power: f32,
    pub influence_radius: f32,
}

impl PointLight {
    /*
    float getSquareFalloffAttenuation(vec3 posToLight, float lightInvRadius) {
        float distanceSquare = dot(posToLight, posToLight);
        float factor = distanceSquare * lightInvRadius * lightInvRadius;
        float smoothFactor = max(1.0 - factor * factor, 0.0);
        return (smoothFactor * smoothFactor) / max(distanceSquare, 1e-4);
    }
    */

    pub fn get_square_falloff_attenuation(
        &self,
        light_position: kmath::Vec3,
        target_position: kmath::Vec3,
    ) -> f32 {
        let inverse_radius = 1.0 / self.influence_radius;
        let target_to_light = light_position - target_position;
        let distance_squared = target_to_light.length_squared();
        let factor = distance_squared * inverse_radius * inverse_radius;
        let factor_squared = factor*factor;
        let smooth_factor = (1.0 - factor_squared).max(0.0);
        (smooth_factor * smooth_factor) / distance_squared.max(0.0001)
    }
}

/// For large light sources that effect an entire environment, like the sun.
/// Refer to this table for lighting values:
/// https://en.wikipedia.org/wiki/Lux
pub struct DirectionalLight {
    pub illuminance: f32,
}

// Should these constants be just the measurement of the light source?
// Or the light source + ambient light?
impl DirectionalLight {
    pub const MOONLESS_OVERCAST_NIGHT: Self = DirectionalLight {
        illuminance: 0.0001,
    };
    pub const MOONLESS_CLEAR_NIGHT: Self = DirectionalLight { illuminance: 0.002 };
    pub const FULLMOON_CLEAR_NIGHT: Self = DirectionalLight { illuminance: 0.175 };

    pub const LATE_TWILIGHT: Self = DirectionalLight { illuminance: 1.0 };
    pub const EARLY_TWILIGHT: Self = DirectionalLight { illuminance: 10.0 };

    pub const DARK_LIVING_ROOM: Self = DirectionalLight { illuminance: 50.0 };
    pub const VERY_DARK_OVERCAST_DAY: Self = DirectionalLight { illuminance: 100.0 };
    pub const OFFICE_LIGHTING: Self = DirectionalLight { illuminance: 410.0 };

    pub const SUNRISE_OR_SUNSET: Self = DirectionalLight { illuminance: 400.0 };
    pub const OVERCAST_DAY: Self = DirectionalLight {
        illuminance: 1000.0,
    };
    pub const SUNNY_CLEAR_DAY_NOON: Self = DirectionalLight {
        illuminance: 130_000.0,
    };
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self { illuminance: 1.0 }
    }
}
