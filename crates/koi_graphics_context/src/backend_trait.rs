use kapp::Window;

use crate::*;

#[allow(clippy::missing_safety_doc)]
pub trait BackendTrait {
    unsafe fn set_main_window(&mut self, window: &Window);
    unsafe fn execute_command_buffer(
        &mut self,
        command_buffer: &crate::CommandBuffer,
        buffer_sizes: &[u32],
        texture_sizes: &[(u32, u32, u32)],
    );
    unsafe fn new_texture(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        pixel_format: PixelFormat,
        settings: TextureSettings,
    ) -> TextureInner;
    unsafe fn update_texture(
        &mut self,
        texture: &TextureInner,
        x: u32,
        y: u32,
        z: u32,
        width: u32,
        height: u32,
        depth: u32,
        data: &[u8],
        settings: TextureSettings,
    );

    unsafe fn delete_texture(&mut self, texture_inner: TextureInner);

    unsafe fn new_cube_map(
        &mut self,
        width: u32,
        height: u32,
        pixel_format: PixelFormat,
        texture_settings: TextureSettings,
    ) -> CubeMapInner;

    unsafe fn update_cube_map(
        &mut self,
        cube_map: &CubeMapInner,
        width: u32,
        height: u32,
        data: &[&[u8]; 6],
        texture_settings: TextureSettings,
    );
    unsafe fn delete_cube_map(&mut self, cube_map_inner: CubeMapInner);

    unsafe fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: PipelineSettings,
    ) -> Result<PipelineInner, String>;
    unsafe fn delete_pipeline(&mut self, pipeline_inner: PipelineInner);

    unsafe fn new_buffer(&mut self, buffer_usage: BufferUsage, data: &[u8]) -> BufferInner;
    unsafe fn delete_buffer(&mut self, buffer_inner: BufferInner);

    #[cfg(target_arch = "wasm32")]
    unsafe fn new_texture_from_js_object(
        &mut self,
        _width: u32,
        _height: u32,
        _js_object_data: &kwasm::JSObjectDynamic,
        _pixel_format: PixelFormat,
        _texture_settings: TextureSettings,
    ) -> TextureInner;
}
