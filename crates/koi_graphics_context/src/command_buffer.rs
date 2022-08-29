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
    current_pipeline: Option<u32>,
    command_buffer: &'a mut CommandBuffer,
}

impl<'a> RenderPass<'a> {
    pub fn set_pipeline(&mut self, pipeline: Pipeline) {
        self.current_pipeline = Some(pipeline.0.inner().program_index);
        self.command_buffer.0.push(Command::SetPipeline(pipeline))
    }

    pub fn set_vertex_attribute<D: BufferDataTrait>(
        &mut self,
        vertex_attribute: VertexAttribute<D>,
        buffer: Option<Buffer<D>>,
    ) {
        self.command_buffer.0.push(Command::SetVertexAttribute {
            attribute: vertex_attribute.untyped(),
            buffer: buffer.map(|b| b.untyped()),
        })
    }
    pub fn draw(
        &mut self,
        index_buffer: Option<Buffer<[u32; 3]>>,
        triangle_range: std::ops::Range<u32>,
        instances: u32,
    ) {
        if let Some(index_buffer) = index_buffer.as_ref() {
            assert_eq!(index_buffer.handle.inner().buffer_usage, BufferUsage::Index)
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
