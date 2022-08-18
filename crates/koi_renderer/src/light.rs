// Koi's lighting model is heavily based on this excellent document: https://google.github.io/filament/Filament.html#lighting

/// For light sources that emit from a point, like a lightbulb.
pub struct PointLight {
    pub intensity_luminous_power: f32,
    pub color: kcolor::Color,
    pub influence_radius: f32,
}

impl PointLight {
    pub fn get_square_falloff_attenuation(
        &self,
        light_position: kmath::Vec3,
        target_position: kmath::Vec3,
    ) -> f32 {
        let inverse_radius = 1.0 / self.influence_radius;
        let target_to_light = light_position - target_position;
        let distance_squared = target_to_light.length_squared();
        let factor = distance_squared * inverse_radius * inverse_radius;
        let factor_squared = factor * factor;
        let smooth_factor = (1.0 - factor_squared).max(0.0);
        (smooth_factor * smooth_factor) / distance_squared.max(0.0001)
    }
}

/// For large light sources that effect an entire environment, like the sun.
/// Refer to this table for lighting values:
/// https://en.wikipedia.org/wiki/Lux
pub struct DirectionalLight {
    pub intensity_illuminance: f32,
    pub color: kcolor::Color,
}

// Should these constants be just the measurement of the light source?
// Or the light source + ambient light?
impl DirectionalLight {
    pub const MOONLESS_OVERCAST_NIGHT: Self = DirectionalLight {
        intensity_illuminance: 0.0001,
        color: color_temperatures::NIGHT_SKY,
    };
    pub const MOONLESS_CLEAR_NIGHT: Self = DirectionalLight {
        intensity_illuminance: 0.002,
        color: color_temperatures::NIGHT_SKY,
    };
    pub const FULLMOON_CLEAR_NIGHT: Self = DirectionalLight {
        intensity_illuminance: 0.175,
        color: color_temperatures::NIGHT_SKY,
    };
    pub const LATE_TWILIGHT: Self = DirectionalLight {
        intensity_illuminance: 1.0,
        color: color_temperatures::NIGHT_SKY,
    };
    pub const EARLY_TWILIGHT: Self = DirectionalLight {
        intensity_illuminance: 10.0,
        color: color_temperatures::NIGHT_SKY,
    };
    pub const DARK_LIVING_ROOM: Self = DirectionalLight {
        intensity_illuminance: 50.0,
        color: color_temperatures::INCADESCENT_LAMP,
    };
    pub const VERY_DARK_OVERCAST_DAY: Self = DirectionalLight {
        intensity_illuminance: 100.0,
        color: color_temperatures::OVERCAST_SKY,
    };
    pub const OFFICE_LIGHTING: Self = DirectionalLight {
        intensity_illuminance: 410.0,
        color: color_temperatures::FLUORESCENT_LIGHT,
    };
    pub const SUNRISE_OR_SUNSET: Self = DirectionalLight {
        intensity_illuminance: 400.0,
        color: color_temperatures::CANDLE_FLAME,
    };
    pub const OVERCAST_DAY: Self = DirectionalLight {
        intensity_illuminance: 1000.0,
        color: color_temperatures::OVERCAST_SKY,
    };
    pub const SUNNY_CLEAR_DAY_NOON: Self = DirectionalLight {
        intensity_illuminance: 130_000.0,
        color: color_temperatures::DAYLIGHT,
    };
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self::OVERCAST_DAY
    }
}

pub mod color_temperatures {
    use kcolor::Color;
    pub const CANDLE_FLAME: Color = Color {
        x: 1.3457389,
        y: 1.0,
        z: 0.10495334,
        alpha: 1.0,
    };
    pub const SUNRISE_OR_SUNSET: Color = Color {
        x: 1.3457389,
        y: 1.0,
        z: 0.10495334,
        alpha: 1.0,
    };
    pub const INCADESCENT_LAMP: Color = Color {
        x: 1.1717641,
        y: 1.0,
        z: 0.23801358,
        alpha: 1.0,
    };
    pub const LIGHTBULB: Color = Color {
        x: 1.1193128,
        y: 1.0,
        z: 0.31398392,
        alpha: 1.0,
    };
    /// A totally made up value. TODO: Research this more
    pub const NIGHT_SKY: Color = Color {
        x: 1.098603,
        y: 1.0,
        z: 0.352944,
        alpha: 1.0,
    };
    pub const FLUORESCENT_LIGHT: Color = Color {
        x: 1.037247,
        y: 1.0,
        z: 0.5216907,
        alpha: 1.0,
    };
    pub const MOON: Color = Color {
        x: 1.0060449,
        y: 1.0,
        z: 0.66890204,
        alpha: 1.0,
    };
    pub const HORIZON_DAYLIGHT: Color = Color {
        x: 0.9821848,
        y: 1.0,
        z: 0.86439204,
        alpha: 1.0,
    };
    pub const SUN_AT_NOON: Color = Color {
        x: 0.97908896,
        y: 1.0,
        z: 0.90345436,
        alpha: 1.0,
    };
    pub const DAYLIGHT: Color = Color {
        x: 0.97137773,
        y: 1.0,
        z: 1.0447035,
        alpha: 1.0,
    };
    pub const COMPUTER_MONITOR: Color = Color {
        x: 0.96912414,
        y: 1.0,
        z: 1.1218442,
        alpha: 1.0,
    };
    pub const OVERCAST_SKY: Color = Color {
        x: 0.9684853,
        y: 1.0,
        z: 1.157538,
        alpha: 1.0,
    };
    pub const PARTLY_CLOUD_SKY: Color = Color {
        x: 0.9701533,
        y: 1.0,
        z: 1.4104165,
        alpha: 1.0,
    };
}

#[test]
fn print_colors() {
    use kcolor::Color;
    println!(
        "pub const CANDLE_FLAME: Color = {:#?};",
        Color::from_temperature(1800.0)
    );

    println!(
        "pub const SUNRISE_OR_SUNSET: Color = {:#?};",
        Color::from_temperature(1800.0)
    );

    println!(
        "pub const INCADESCENT_LAMP: Color = {:#?};",
        Color::from_temperature(2400.0)
    );

    println!(
        "pub const LIGHTBULB: Color = {:#?};",
        Color::from_temperature(2700.0)
    );

    println!(
        "/// A totally made up value. TODO: Research this more\npub const NIGHT_SKY: Color = {:#?};",
        Color::from_temperature(2850.0)
    );

    println!(
        "pub const FLUORESCENT_LIGHT: Color = {:#?};",
        Color::from_temperature(3500.0)
    );

    println!(
        "pub const MOON: Color = {:#?};",
        Color::from_temperature(4100.0)
    );

    println!(
        "pub const HORIZON_DAYLIGHT: Color = {:#?};",
        Color::from_temperature(5000.0)
    );

    println!(
        "pub const SUN_AT_NOON: Color = {:#?};",
        Color::from_temperature(5200.0)
    );

    println!(
        "pub const DAYLIGHT: Color = {:#?};",
        Color::from_temperature(6000.0)
    );

    println!(
        "pub const COMPUTER_MONITOR: Color = {:#?};",
        Color::from_temperature(6500.0)
    );

    println!(
        "pub const OVERCAST_SKY: Color = {:#?};",
        Color::from_temperature(6750.0)
    );

    println!(
        "pub const PARTLY_CLOUD_SKY: Color = {:#?};",
        Color::from_temperature(9000.0)
    );
}
