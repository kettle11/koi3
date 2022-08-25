use kmath::*;
use koi_animation::InterpolateTrait;
use koi_ecs::Component;

pub mod transform_plugin;

#[derive(Clone, Copy, Debug, Component)]
pub struct GlobalTransform(Transform);

impl core::ops::Deref for GlobalTransform {
    type Target = Transform;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct Transform {
    /// Position relative to parent
    pub position: Vec3,
    /// Rotation relative to parent
    pub rotation: Quat,
    /// Scale relative to parent
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn new_looking_at(origin: Vec3, target: Vec3, up: Vec3) -> Self {
        let transform = Self {
            position: origin,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        transform.looking_at(target, up)
    }

    pub fn from_mat4(mat4: Mat4) -> Self {
        let (position, rotation, scale) = mat4.to_translation_rotation_scale();
        Self {
            position,
            rotation,
            scale,
        }
    }

    /// Get the matrix that transforms points from local
    /// coordinate space to the global coordinate space.
    /// Also commonly referred to as the "model" matrix.
    pub fn local_to_world(&self) -> Mat4 {
        Mat4::from_translation_rotation_scale(self.position, self.rotation, self.scale)
    }

    /// This doesn't correctly respect global vs local transforms.
    #[must_use]
    pub fn looking_at(mut self, target: Vec3, up: Vec3) -> Self {
        let rotation = Mat4::looking_at(self.position, target, up)
            .inversed()
            .extract_rotation();
        self.rotation = rotation;
        self
    }

    #[inline]
    pub fn right(&self) -> Vec3 {
        self.rotation.rotate_vector3(Vec3::X)
    }

    #[inline]
    pub fn left(&self) -> Vec3 {
        self.rotation.rotate_vector3(-Vec3::X)
    }

    #[inline]
    pub fn up(&self) -> Vec3 {
        self.rotation.rotate_vector3(Vec3::Y)
    }

    #[inline]
    pub fn down(&self) -> Vec3 {
        self.rotation.rotate_vector3(-Vec3::Y)
    }

    #[inline]
    pub fn forward(&self) -> Vec3 {
        self.rotation.rotate_vector3(-Vec3::Z)
    }

    #[inline]
    pub fn back(&self) -> Vec3 {
        self.rotation.rotate_vector3(Vec3::Z)
    }
}

impl InterpolateTrait for Transform {
    /// Linearly interpolate transform and scale. Spherically interpolate rotation.
    fn interpolate(&self, other: &Self, amount: f32) -> Self {
        let position = self.position.lerp(other.position, amount);
        let rotation = self.rotation.slerp(other.rotation, amount);
        let scale = self.scale.lerp(other.scale, amount);
        Transform {
            position,
            rotation,
            scale,
        }
    }
}
