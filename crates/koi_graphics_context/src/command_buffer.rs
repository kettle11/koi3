use crate::*;

pub enum Command {
    Clear(kmath::Vec4),
    Present,
    SetPipeline(Pipeline),
    Draw {
        index_buffer: Option<Buffer<[u32; 3]>>,
        triangle_range: std::ops::Range<u32>,
        instances: u32,
    },
    SetVertexAttribute {
        attribute: VertexAttributeUntyped,
        buffer: Option<BufferUntyped>,
    },
    SetUniformBlock {
        uniform_block_index: u8,
        buffer: Option<BufferUntyped>,
    },
    SetTexture {
        texture_unit: u8,
        texture: Texture,
    },
}

pub struct CommandBuffer(pub(crate) Vec<Command>);

impl CommandBuffer {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn begin_render_pass(&mut self, color: Option<kmath::Vec4>) -> RenderPass {
        if let Some(color) = color {
            self.0.push(Command::Clear(color))
        }
        RenderPass {
            current_pipeline: None,
            command_buffer: self,
        }
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}

pub struct RenderPass<'a> {
    current_pipeline: Option<Handle<PipelineInner>>,
    command_buffer: &'a mut CommandBuffer,
}

impl<'a> RenderPass<'a> {
    pub fn set_pipeline(&mut self, pipeline: Pipeline) {
        self.current_pipeline = Some(pipeline.0.clone());
        self.command_buffer.0.push(Command::SetPipeline(pipeline))
    }

    pub fn set_vertex_attribute<D: BufferDataTrait>(
        &mut self,
        vertex_attribute: VertexAttribute<D>,
        buffer: Option<Buffer<D>>,
    ) {
        assert_eq!(
            Some(vertex_attribute.pipeline_index),
            self.current_pipeline
                .as_ref()
                .map(|p| p.inner().program_index),
            "`vertex attribute` is from a pipeline that is not currently bound."
        );
        self.command_buffer.0.push(Command::SetVertexAttribute {
            attribute: vertex_attribute.untyped(),
            buffer: buffer.map(|b| b.untyped()),
        })
    }

    pub fn set_uniform_block<D: BufferDataTrait>(
        &mut self,
        uniform_block_index: u8,
        buffer: Option<&Buffer<D>>,
    ) {
        if let Some(buffer) = buffer {
            assert_eq!(buffer.handle.inner().buffer_usage, BufferUsage::Data);
        }

        let pipeline_inner = self.current_pipeline.as_ref().unwrap();
        if let Some(uniform_buffer) = pipeline_inner
            .inner()
            .uniform_blocks
            .get(uniform_block_index as usize)
        {
            assert_eq!(
                uniform_buffer.size_bytes as usize,
                std::mem::size_of::<D>(),
                "Incorrectly sized block passed to uniform block"
            );

            self.command_buffer.0.push(Command::SetUniformBlock {
                uniform_block_index,
                buffer: buffer.map(|b| b.untyped()),
            })
        }
    }

    pub fn set_texture(&mut self, texture_unit: u8, texture: &Texture) {
        assert!(texture_unit < 16);
        self.command_buffer.0.push(Command::SetTexture {
            texture_unit,
            texture: texture.clone(),
        });
    }

    pub fn draw(
        &mut self,
        index_buffer: Option<Buffer<[u32; 3]>>,
        triangle_range: std::ops::Range<u32>,
        instances: u32,
    ) {
        if let Some(index_buffer) = index_buffer.as_ref() {
            assert_eq!(
                index_buffer.handle.inner().buffer_usage,
                BufferUsage::Index,
                "`index_buffer` was not declared with `BufferUsage::Index`"
            )
        }
        self.command_buffer.0.push(Command::Draw {
            index_buffer,
            triangle_range,
            instances,
        })
    }
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        self.command_buffer.0.push(Command::Present);
    }
}
