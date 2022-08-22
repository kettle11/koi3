use crate::cube_map::*;

pub fn spherical_harmonics_from_cubemap<const CHANNELS: usize>(
    faces: &[&[kmath::Vector<f32, CHANNELS>]],
    face_dimensions: usize,
) -> [kmath::Vector<f32, CHANNELS>; 9] {
    let mut coefficients = [kmath::Vector::<f32, CHANNELS>::ZERO; 9];
    for (index, face) in faces.iter().enumerate() {
        for (data_index, sample) in face.iter().enumerate() {
            let y = data_index / face_dimensions;
            let x = data_index % face_dimensions;

            let direction = get_direction_for(index, x as f32, y as f32, face_dimensions as f32);

            let solid_angle =
                solid_angle_of_cube_map_pixel(face_dimensions as f32, x as f32, y as f32);

            let sh_value =
                spherical_harmonic_sample_analytical_order2(direction.x, direction.y, direction.z);

            for i in 0..9 {
                coefficients[i] += *sample * sh_value[i] * solid_angle;
            }
        }
    }

    for i in 0..9 {
        // The sume of all the solid angles is PI * 4.0, so normalize by that here.
        coefficients[i] /= core::f32::consts::PI * 4.0;
    }
    coefficients
}

pub fn prepare_spherical_harmonics_for_shader<const CHANNELS: usize>(
    coefficients: &mut [kmath::Vector<f32, CHANNELS>; 9],
) {
    *coefficients = convolve_with_cos_irradiance(*coefficients);
    for i in 0..9 {
        coefficients[i] *= A[i];
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
    0.3153915652535312,  // 0.25 * (5.0 / PI).sqrt()
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
        A[4] * x * y,               // 0.5 * (15.0 / PI).sqrt() * x * y
        A[5] * y * z,               // -0.5 * (15.0 / PI).sqrt() * y * z
        A[6] * (3.0 * z * z - 1.0), // 0.25 * (5.0 / PI).sqrt() * (3.0 * z * z - 1.0)
        A[7] * x * z,               // -0.5 * (15.0 / PI).sqrt() * x * z
        A[8] * (x * x - y * y),     // 0.25 * (15.0 / PI).sqrt() * (x * x - y * y)
    ]
    .into()
}

fn reconstruct_from_spherical_harmonic_coefficients(
    //theta: f32,
    //phi: f32,
    x: f32,
    y: f32,
    z: f32,
    coefficients: &[f32; 9],
) -> f32 {
    let mut total = 0.0;
    let v: [f32; 9] = spherical_harmonic_sample_analytical_order2(x, y, z).into();
    for i in 0..9 {
        total += v[i] * coefficients[i];
    }
    total
}

fn convolve_with_cos_irradiance<const CHANNELS: usize>(
    coefficients: [kmath::Vector<f32, CHANNELS>; 9],
) -> [kmath::Vector<f32, CHANNELS>; 9] {
    // Based upon "An Efficient Representation for Irradiance Environment Maps"
    // https://cseweb.ucsd.edu/~ravir/papers/envmap/envmap.pdf
    // But the explanation on this page is what made it click for me:
    // https://imdoingitwrong.wordpress.com/2011/04/14/spherical-harmonics-wtf/
    // Spherical harmonics for the *radiance* of a cubemap can be transformed into
    // *irradiance* for a given direction with the following simple transformation.
    const A: [f32; 3] = [
        core::f32::consts::PI,
        2.094395, // (2*PI) / 3
        0.785398, // PI / 4
                  // These are unused but are the terms for the next few orders:
                  // 0
                  // -0.130900, // -PI / 24
                  // 0.049087,  // PI / 64
    ];

    [
        // Order l = 0
        coefficients[0] * A[0],
        // Order l = 1
        coefficients[1] * A[1],
        coefficients[2] * A[1],
        coefficients[3] * A[1],
        // Order l = 2
        coefficients[4] * A[2],
        coefficients[5] * A[2],
        coefficients[6] * A[2],
        coefficients[7] * A[2],
        coefficients[8] * A[2],
    ]
}

// Quick approximation of solid angle.
// Seems fairly accurate.
fn solid_angle_of_cube_map_pixel(dim: f32, x: f32, y: f32) -> f32 {
    let s = (2.0 * (x + 0.5)) / dim - 1.0;
    let t = (2.0 * (y + 0.5)) / dim - 1.0;
    let temp = 1.0 + s * s + t * t;
    4.0 / (temp.sqrt() * temp * dim * dim)
}

// More complex but slightly more accurate approach to calculating solid angle.
/*
fn sphere_quadrant_area(x: f32, y: f32) -> f32 {
    (x * y).atan2((x * x + y * y + 1.0).sqrt())
}

fn solid_angle(dim: f32, u: f32, v: f32) -> f32 {
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
*/

// Unused code that could be used for higher order cases or for verifying the above code
/*
/// Returns n! / d!
fn factorial(mut n: isize, mut d: isize) -> f32 {
    let mut r = 1.0;
    if n == d {
        // intentionally left blank
    } else if n > d {
        while n > d {
            r *= n as f32;
            n -= 1;
        }
    } else {
        while d > n {
            r *= d as f32;
            d -= 1;
        }

        r = 1.0 / r;
    }
    r
}

/// Spherical harmonic normalization factor
fn k_lm(l: isize, m: isize) -> f32 {
    // Filament's math code removes the / (4.0 * core::f32::consts::PI) and instead replaces
    // it with a constant.
    (((2 * l + 1) as f32 / (4.0 * core::f32::consts::PI)) * factorial(l - m, l + m)).sqrt()
}

/// Evaluate an Associated Legendre Polynomial P(l,m,x) at x
/// For more, see “Numerical Methods in C: The Art of Scientific Computing”, Cambridge University Press, 1992, pp 252-254
/// // Adapted from here: https://patapom.com/blog/SHPortal/
fn legendre_polynomial(l: isize, m: isize, x: f32) -> f32 {
    let mut pmm = 1.0;
    if m > 0 {
        let somx2 = ((1.0 - x) * (1.0 + x)).sqrt();
        let mut fact = 1.0;
        for _ in 1..=m {
            pmm *= (-fact) * somx2;
            fact += 2.0;
        }
    }
    if l == m {
        return pmm;
    }

    let mut pmmp1 = x * (2.0 * (m as f32) + 1.0) * pmm;
    if l == m + 1 {
        return pmmp1;
    }

    let mut pll = 0.0;
    for ll in m + 2..=l {
        let ll = ll as f32;
        let m = m as f32;
        pll = (2.0 * ll - 1.0) * x * pmmp1 - (ll + (m - 1.0) * pmm) / (ll - m);
        pmm = pmmp1;
        pmmp1 = pll;
    }

    pll
}

/// l is the band.
/// m is in the range [-l..l]
/// Theta and Phi are cartesian coordinate angles.
/// Theta is in the range [0..Pi]
/// Phi is in the range [0..2*PI]
fn spherical_harmonic_sample(l: isize, m: isize, theta: f32, phi: f32) -> f32 {
    let sqrt2 = 2.0f32.sqrt();
    if m == 0 {
        k_lm(l, 0) * legendre_polynomial(l, m, theta.cos())
    } else if m > 0 {
        sqrt2 * k_lm(l, m) * (m as f32 * phi).cos() * legendre_polynomial(l, m, theta.cos())
    } else {
        sqrt2 * k_lm(l, -m) * (-m as f32 * phi).sin() * legendre_polynomial(l, -m, theta.cos())
    }
}
*/
