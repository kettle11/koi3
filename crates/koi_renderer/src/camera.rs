#[derive(Clone)]
pub struct Camera {
    pub clear_color: Option<kcolor::Color>,
    pub output_rect: kmath::Box2,
    pub projection_mode: ProjectionMode,
    /// How much light this camera will accept
    /// https://en.wikipedia.org/wiki/Exposure_value
    pub exposure: Exposure,
}

#[derive(Clone, Debug)]
pub enum Exposure {
    EV100(f32),
    PhysicalCamera {
        aperture_f_stops: f32,
        shutter_speed_seconds: f32,
        sensitivity_iso: f32,
    },
}

impl Exposure {
    pub fn to_ev100(&self) -> f32 {
        match *self {
            Self::EV100(v) => v,
            Self::PhysicalCamera {
                aperture_f_stops,
                shutter_speed_seconds,
                sensitivity_iso,
            } => ((aperture_f_stops * aperture_f_stops) / shutter_speed_seconds * 100.0
                / sensitivity_iso)
                .log2(),
        }
    }

    /// The max luminance possible without clipping.
    /// Is used as a scale factor to scale the scene.
    pub fn max_luminance_without_clipping(&self) -> f32 {
        //  println!("EXPOSURE VALUE: {:?}", self.exposure.to_ev100());
        let v = 2.0f32.powf(self.to_ev100()) * 1.2;
        v
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            clear_color: Some(kcolor::Color::BLACK),
            output_rect: kmath::Box2::new(kmath::Vec2::ZERO, kmath::Vec2::ONE),
            projection_mode: ProjectionMode::Perspective {
                field_of_view_y_radians: (72.0_f32).to_radians(),
                z_near: 0.3,
            },
            exposure: Exposure::EV100(8.0),
        }
    }
}

impl Camera {
    pub fn projection_matrix(&self, view_width: f32, view_height: f32) -> kmath::Mat4 {
        // This may need to be updated later.
        let (width, height) = self.output_rect.size().into();
        self.projection_mode
            .to_mat4((width * view_width) / (height * view_height))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ProjectionMode {
    Perspective {
        field_of_view_y_radians: f32,
        /// The near clipping plane
        z_near: f32,
    },
    Orthographic {
        height: f32,
        z_near: f32,
        z_far: f32,
    },
    Custom(kmath::Mat4),
}

impl ProjectionMode {
    pub fn to_mat4(&self, aspect_ratio: f32) -> kmath::Mat4 {
        match *self {
            Self::Perspective {
                field_of_view_y_radians,
                z_near,
            } => kmath::projection_matrices::perspective_infinite_gl(
                field_of_view_y_radians,
                aspect_ratio,
                z_near,
            ),

            Self::Orthographic {
                height,
                z_near,
                z_far,
            } => {
                let width = aspect_ratio * height;
                let half_width = width / 2.;
                let half_height = height / 2.;

                kmath::projection_matrices::orthographic_gl(
                    -half_width,
                    half_width,
                    -half_height,
                    half_height,
                    z_near,
                    z_far,
                )
            }
            ProjectionMode::Custom(m) => m,
        }
    }
}
