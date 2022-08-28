use crate::*;

pub trait BackendTrait {
    unsafe fn execute_command_buffer(&mut self, command_buffer: &crate::CommandBuffer);
    unsafe fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        pixel_format: PixelFormat,
        settings: TextureSettings,
    ) -> TextureInner;
    unsafe fn delete_texture(&mut self, texture_inner: TextureInner);

    unsafe fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: PipelineSettings,
    ) -> Result<PipelineInner, String>;
    unsafe fn delete_pipeline(&mut self, pipeline_inner: PipelineInner);

    unsafe fn new_triangle_buffer(&mut self, indices: &[[u32; 3]]) -> TriangleBufferInner;
    unsafe fn delete_triangle_buffer(&mut self, triangle_buffer_inner: TriangleBufferInner);
}
