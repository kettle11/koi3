use crate::*;

pub trait BackendTrait {
    unsafe fn execute_command_buffer(
        &mut self,
        command_buffer: &crate::CommandBuffer,
        buffer_sizes: &Vec<u32>,
    );
    unsafe fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        pixel_format: PixelFormat,
        settings: TextureSettings,
    ) -> TextureInner;
    unsafe fn update_texture(
        &mut self,
        texture: &TextureInner,
        x: usize,
        y: usize,
        z: usize,
        width: usize,
        height: usize,
        depth: usize,
        data: &[u8],
        settings: TextureSettings,
    );
    unsafe fn delete_texture(&mut self, texture_inner: TextureInner);

    unsafe fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: PipelineSettings,
    ) -> Result<PipelineInner, String>;
    unsafe fn delete_pipeline(&mut self, pipeline_inner: PipelineInner);

    unsafe fn new_buffer(&mut self, buffer_usage: BufferUsage, data: &[u8]) -> BufferInner;
    unsafe fn delete_buffer(&mut self, buffer_inner: BufferInner);
}
