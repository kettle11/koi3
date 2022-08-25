// This file is primarily an adaptation of Filament's code here:
// https://github.com/google/filament/blob/80f13a8149147ce733cadf896931af5f79ead73e/libs/ibl/src/CubemapIBL.cpp#L748

use kmath::*;

fn hammersley(i: u32, num_samples: f32) -> Vec2 {
    let mut bits = i;
    bits = (bits << 16) | (bits >> 16);
    bits = ((bits & 0x55555555) << 1) | ((bits & 0xAAAAAAAA) >> 1);
    bits = ((bits & 0x33333333) << 2) | ((bits & 0xCCCCCCCC) >> 2);
    bits = ((bits & 0x0F0F0F0F) << 4) | ((bits & 0xF0F0F0F0) >> 4);
    bits = ((bits & 0x00FF00FF) << 8) | ((bits & 0xFF00FF00) >> 8);
    Vec2::new(i as f32 / num_samples, bits as f32 / 2.0f32.powf(32.0))
}

#[allow(non_snake_case)]
fn gdfg(NoV: f32, NoL: f32, a: f32) -> f32 {
    let a2 = a * a;
    let GGXL = NoV * f32::sqrt((-NoL * a2 + NoL) * NoL + a2);
    let GGXV = NoL * f32::sqrt((-NoV * a2 + NoV) * NoV + a2);
    (2.0 * NoL) / (GGXV + GGXL)
}

fn hemisphere_importance_sample_dggx(u: Vec2, a: f32) -> Vec3 {
    let phi = 2.0 * core::f32::consts::PI * u.x;
    let cos_theta2 = (1.0 - u.y) / (1.0 + (a + 1.0) * ((a - 1.0) * u.y));
    let cos_theta = f32::sqrt(cos_theta2);
    let sin_theta = f32::sqrt(1.0 - cos_theta2);
    return Vec3::new(
        sin_theta * f32::cos(phi),
        sin_theta * f32::sin(phi),
        cos_theta,
    );
}

fn visibility(NoV: f32, NoL: f32, a: f32) -> f32 {
    // Heitz 2014, "Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs"
    // Height-correlated GGX
    let a2 = a * a;
    let GGXL = NoV * f32::sqrt((NoL - NoL * a2) * NoL + a2);
    let GGXV = NoL * f32::sqrt((NoV - NoV * a2) * NoV + a2);
    0.5 / (GGXV + GGXL)
}

fn dfg(NoV: f32, linear_roughness: f32, sample_count: u32) -> Vec2 {
    // Filament's implementation is here:
    // https://github.com/google/filament/blob/80f13a8149147ce733cadf896931af5f79ead73e/libs/ibl/src/CubemapIBL.cpp#L748

    let v = Vec3::new(f32::sqrt(1.0 - NoV * NoV), 0.0, NoV);

    let mut r = Vec2::ZERO;
    for i in 0..sample_count {
        let u = hammersley(i, sample_count as f32);
        let h: Vec3 = hemisphere_importance_sample_dggx(u, linear_roughness);
        let l = 2.0 * Vector::dot(v, h) * h - v;

        let VoH = f32::clamp(Vector::dot(v, h), 0.0, 1.0);
        let NoL = f32::clamp(l.z, 0.0, 1.0);
        let NoH = f32::clamp(h.z, 0.0, 1.0);

        if NoL > 0.0 {
            let G = gdfg(NoV, NoL, linear_roughness);
            let Gv = G * VoH / NoH;
            let Fc = f32::powf(1.0 - VoH, 5.0);
            r.x += Gv * (1.0 - Fc);
            r.y += Gv * Fc;
        }
    }

    r * (1.0 / sample_count as f32)
}
