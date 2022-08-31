use std::collections::HashMap;

use kwasm::*;

use crate::{backend_trait::BackendTrait, BufferInner, GraphicsContextSettings};

pub struct WebGPUBackend {
    new_pipeline: JSObject,
    new_buffer: JSObject,
    new_texture: JSObject,
    update_texture: JSObject,
    destroy: JSObject,
}

impl WebGPUBackend {
    pub async fn new(_settings: GraphicsContextSettings) -> Option<Self> {
        let o = JSObjectFromString::new(include_str!("webgpu_backend.js"));
        let setup = o.get_property("setup");

        let setup_result = kwasm::JSFuture::new(
            move || setup.call().unwrap(),
            |js_object| Some(Box::new(())),
        );
        setup_result.await;

        Some(Self {
            new_pipeline: o.get_property("new_pipeline"),
            new_buffer: o.get_property("new_buffer"),
            new_texture: o.get_property("new_texture"),
            update_texture: o.get_property("update_texture"),
            destroy: o.get_property("destroy"),
        })
    }
}

impl BackendTrait for WebGPUBackend {
    unsafe fn set_main_window(&mut self, _window: &kapp::Window) {}
    unsafe fn execute_command_buffer(
        &mut self,
        command_buffer: &crate::CommandBuffer,
        buffer_sizes: &Vec<u32>,
        texture_sizes: &Vec<(u32, u32, u32)>,
    ) {
        klog::log!("TODO: execute_command_buffer");
    }

    unsafe fn new_texture(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        pixel_format: crate::PixelFormat,
        settings: crate::TextureSettings,
    ) -> crate::TextureInner {
        // TODO: Use texture settings

        let js_object = self
            .new_texture
            .call_raw(&[
                width,
                height,
                depth,
                // TODO: Pass an actual pixel format.
                match pixel_format {
                    crate::PixelFormat::R8Unorm => 0,
                    crate::PixelFormat::RG8Unorm => 0,
                    crate::PixelFormat::RGB8Unorm => 0,
                    crate::PixelFormat::RGBA8Unorm => 0,
                    crate::PixelFormat::Depth16 => 0,
                    crate::PixelFormat::Depth24 => 0,
                    crate::PixelFormat::Depth32F => 0,
                    crate::PixelFormat::RGBA16F => 0,
                    crate::PixelFormat::RGBA32F => 0,
                },
            ])
            .unwrap();

        crate::TextureInner {
            index: js_object.leak(),
            texture_type: crate::TextureType::Texture,
            pixel_format,
            mip: 0,
        }
    }

    unsafe fn update_texture(
        &mut self,
        texture: &crate::TextureInner,
        x: u32,
        y: u32,
        z: u32,
        width: u32,
        height: u32,
        depth: u32,
        data: &[u8],
        settings: crate::TextureSettings,
    ) {
        let bytes_per_row = data.len() as u32 / (height * depth);
        self.update_texture.call_raw(&[
            texture.index,
            width,
            height,
            bytes_per_row,
            data.as_ptr() as u32,
            data.len() as u32,
        ]);
    }

    unsafe fn delete_texture(&mut self, texture_inner: crate::TextureInner) {
        todo!()
    }

    unsafe fn new_cube_map(
        &mut self,
        width: u32,
        height: u32,
        pixel_format: crate::PixelFormat,
        texture_settings: crate::TextureSettings,
    ) -> crate::CubeMapInner {
        klog::log!("TODO: new_cube_map");

        crate::CubeMapInner {
            index: 0,
            pixel_format,
        }
    }

    unsafe fn update_cube_map(
        &mut self,
        cube_map: &crate::CubeMapInner,
        width: u32,
        height: u32,
        data: &[&[u8]; 6],
        texture_settings: crate::TextureSettings,
    ) {
        klog::log!("TODO: update_cube_map");
    }

    unsafe fn delete_cube_map(&mut self, cube_map_inner: crate::CubeMapInner) {
        todo!()
    }

    unsafe fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: crate::PipelineSettings,
    ) -> Result<crate::PipelineInner, String> {
        let vertex_source = JSString::new(&vertex_source);
        let fragment_source = JSString::new(&fragment_source);
        let js_object = self
            .new_pipeline
            .call_2_arg(&vertex_source, &fragment_source)
            .unwrap();

        Ok(crate::PipelineInner {
            program_index: js_object.leak(),
            pipeline_settings,
            uniforms: HashMap::new(),
            uniform_blocks: Vec::new(),
            vertex_attributes: HashMap::new(),
        })
    }

    unsafe fn delete_pipeline(&mut self, pipeline_inner: crate::PipelineInner) {
        todo!()
    }

    unsafe fn new_buffer(
        &mut self,
        buffer_usage: crate::BufferUsage,
        data: &[u8],
    ) -> crate::BufferInner {
        let js_object = self
            .new_buffer
            .call_raw(&[
                data.as_ptr() as u32,
                data.len() as u32,
                match buffer_usage {
                    crate::BufferUsage::Data => 32,  // GPUBufferUsage.VERTEX
                    crate::BufferUsage::Index => 16, // GPUBufferUsage.INDEX
                },
            ])
            .unwrap();

        BufferInner {
            buffer_usage,
            index: js_object.leak(),
        }
    }

    unsafe fn delete_buffer(&mut self, buffer_inner: crate::BufferInner) {
        (self.destroy).call_1_arg(&JSObject::new_raw(buffer_inner.index));
    }
}
