use crate::cube_map::*;

/// Calculated spherical harmonic coefficients with 2 bands and 9 coefficients.
pub struct SphericalHarmonics<const CHANNELS: usize> {
    pub coefficients: [kmath::Vector<f32, CHANNELS>; 9],
}

impl<const CHANNELS: usize> SphericalHarmonics<CHANNELS> {
    pub fn from_cube_map(faces: &[&[kmath::Vector<f32, CHANNELS>]]) -> Self {
        let face_dimensions = (faces[0].len() as f32).sqrt() as usize;

        let mut solid_angle_total = 0.0;
        let mut coefficients = [kmath::Vector::<f32, CHANNELS>::ZERO; 9];
        for (index, face) in faces.iter().enumerate() {
            for (data_index, sample) in face.iter().enumerate() {
                let y = data_index / face_dimensions;
                let x = data_index % face_dimensions;

                let direction =
                    get_direction_for(index, x as f32, y as f32, face_dimensions as f32);

                let solid_angle = solid_angle2(face_dimensions as f32, x as f32, y as f32);

                solid_angle_total += solid_angle;
                let sh_value = spherical_harmonic_sample_analytical_order2(
                    direction.x,
                    direction.y,
                    direction.z,
                );

                for i in 0..9 {
                    coefficients[i] += *sample * sh_value[i] * solid_angle;
                }
            }
        }

        let normalization_factor = (core::f32::consts::PI * 4.0) / solid_angle_total;
        for i in 0..9 {
            // The coefficients should be normalized to PI * 4.0
            // The sum of all the solid angles is PI * 4.0, so normalize by that here.
            coefficients[i] *= normalization_factor;
        }

        //println!("COEFFICIENTS: {:#?}", coefficients);
        Self { coefficients }
    }

    pub fn scale(&mut self, scale: f32) {
        for c in self.coefficients.iter_mut() {
            *c *= scale;
        }
    }

    /// Convolves with cos irradiance and multiplies by constants.
    pub fn premultiply_constants(&self) -> [kmath::Vector<f32, CHANNELS>; 9] {
        let mut coefficients = self.coefficients;
        for i in 0..9 {
            coefficients[i] *= A[i];
        }
        coefficients
    }

    /// Convolves with cos irradiance and multiplies by constants.
    pub fn convolve_with_cos_irradiance_and_premultiply_constants(
        &self,
        exposure: f32,
    ) -> [kmath::Vector<f32, CHANNELS>; 9] {
        let mut coefficients = convolve_with_cos_irradiance(self.coefficients);
        for i in 0..9 {
            coefficients[i] *= A[i] * exposure;
        }
        coefficients
    }

    /// Get an approximate sample from a direction.
    pub fn sample_direction(&self, direction: kmath::Vec3) -> kmath::Vector<f32, CHANNELS> {
        let mut total = kmath::Vector::<f32, CHANNELS>::ZERO;
        let v: [f32; 9] =
            spherical_harmonic_sample_analytical_order2(direction.x, direction.y, direction.z)
                .into();
        for i in 0..9 {
            total += v[i] * self.coefficients[i];
        }
        total
    }
}

// Constant coefficients
const A: [f32; 9] = [
    // Order l = 0
    0.28209479177475605, // 0.5 * (1.0 / PI).sqrt()
    // Order l = 1
    -0.4886025119132201, // -0.5 * (3.0 / PI).sqrt()
    0.4886025119132201,  // 0.5 * (3.0 / PI).sqrt()
    -0.4886025119132201, // -0.5 * (3.0 / PI).sqrt()
    // Order l = 2
    1.0925484305933868,  // 0.5 * (15.0 / PI).sqrt()
    -1.0925484305933868, // -0.5 * (15.0 / PI).sqrt()
    0.94617469576,       // 3.0 * 0.25 * (5.0 / PI).sqrt() * z * z - (0.25 * (5.0 / PI).sqrt())
    -1.0925484305933868, // -0.5 * (15.0 / PI).sqrt()
    0.5462742152966934,  // 0.25 * (15.0 / PI).sqrt()
];

fn spherical_harmonic_sample_analytical_order2(x: f32, y: f32, z: f32) -> kmath::Vector<f32, 9> {
    // Based on listing 47 on this page:
    // https://google.github.io/filament/Filament.md.html#annex/sphericalharmonics
    // These values have a sign unlike this page. Why?
    // https://patapom.com/blog/SHPortal/#analytical-expressions-for-the-first-sh-coefficients

    [
        // Order l = 0
        A[0], // 0.5 * (1.0 / PI).sqrt()
        // Order l = 1
        A[1] * y, // -0.5 * (3.0 / PI).sqrt() * y
        A[2] * z, // 0.5 * (3.0 / PI).sqrt() * z
        A[3] * x, // -0.5 * (3.0 / PI).sqrt() * x
        // Order l = 2
        A[4] * x * y,                      // 0.5 * (15.0 / PI).sqrt() * x * y
        A[5] * y * z,                      // -0.5 * (15.0 / PI).sqrt() * y * z
        A[6] * z * z - 0.3153915652535312, // 0.25 * (5.0 / PI).sqrt() * (3.0 * z * z - 1.0)
        A[7] * x * z,                      // -0.5 * (15.0 / PI).sqrt() * x * z
        A[8] * (x * x - y * y),            // 0.25 * (15.0 / PI).sqrt() * (x * x - y * y)
    ]
    .into()
}

pub fn convolve_with_cos_irradiance<const CHANNELS: usize>(
    coefficients: [kmath::Vector<f32, CHANNELS>; 9],
) -> [kmath::Vector<f32, CHANNELS>; 9] {
    // Based upon "An Efficient Representation for Irradiance Environment Maps"
    // https://cseweb.ucsd.edu/~ravir/papers/envmap/envmap.pdf
    // But the explanation on this page is what made it click for me:
    // https://imdoingitwrong.wordpress.com/2011/04/14/spherical-harmonics-wtf/
    // Spherical harmonics for the *radiance* of a cubemap can be transformed into
    // *irradiance* for a given direction with the following simple transformation.
    const B: [f32; 3] = [
        core::f32::consts::PI,
        2.094395, // (2*PI) / 3
        0.785398, // PI / 4
    ];

    [
        // Order l = 0
        coefficients[0] * B[0],
        // Order l = 1
        coefficients[1] * B[1],
        coefficients[2] * B[1],
        coefficients[3] * B[1],
        // Order l = 2
        coefficients[4] * B[2],
        coefficients[5] * B[2],
        coefficients[6] * B[2],
        coefficients[7] * B[2],
        coefficients[8] * B[2],
    ]
}

/*
// Quick approximation of solid angle.
// Seems fairly accurate.
fn solid_angle_of_cube_map_pixel(dim: f32, x: f32, y: f32) -> f32 {
    let s = (2.0 * (x + 0.5)) / dim - 1.0;
    let t = (2.0 * (y + 0.5)) / dim - 1.0;
    let temp = 1.0 + s * s + t * t;
    4.0 / (temp.sqrt() * temp * dim * dim)
}
*/

// More complex but slightly more accurate approach to calculating solid angle.

fn sphere_quadrant_area(x: f32, y: f32) -> f32 {
    (x * y).atan2((x * x + y * y + 1.0).sqrt())
}

fn solid_angle2(dim: f32, u: f32, v: f32) -> f32 {
    let i_dim = 1.0 / dim;
    let s = ((u + 0.5) * 2.0 * i_dim) - 1.0;
    let t = ((v + 0.5) * 2.0 * i_dim) - 1.0;
    let x0 = s - i_dim;
    let y0 = t - i_dim;
    let x1 = s + i_dim;
    let y1 = t + i_dim;
    let solid_angle =
        sphere_quadrant_area(x0, y0) - sphere_quadrant_area(x0, y1) - sphere_quadrant_area(x1, y0)
            + sphere_quadrant_area(x1, y1);
    solid_angle
}
