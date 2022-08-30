use crate::*;

impl<T: 'static, const R: usize, const C: usize> BufferDataTrait for kmath::Matrix<T, R, C> {}

impl UniformTypeTrait for kmath::Vec2 {
    const UNIFORM_TYPE: UniformType = UniformType::Vec2(1);
}

impl UniformTypeTrait for kmath::Vec3 {
    const UNIFORM_TYPE: UniformType = UniformType::Vec3(1);
}

impl UniformTypeTrait for kmath::Vec4 {
    const UNIFORM_TYPE: UniformType = UniformType::Vec4(1);
}

impl UniformTypeTrait for kmath::Mat4 {
    const UNIFORM_TYPE: UniformType = UniformType::Mat4(1);
}
