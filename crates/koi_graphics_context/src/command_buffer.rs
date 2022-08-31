use crate::{
    bump_allocator::{BumpAllocator, BumpHandle},
    *,
};

pub(crate) enum Command {
    Clear(kmath::Vec4),
    Present,
    SetPipeline(Pipeline),
    Draw {
        index_buffer: Option<Buffer<[u32; 3]>>,
        triangle_range: std::ops::Range<u32>,
        instances: u32,
    },
    SetViewPort {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    SetUniform {
        uniform_info: UniformInfo,
        bump_handle: BumpHandle,
    },
    SetUniformBlock {
        uniform_block_index: u8,
        buffer: Option<BufferUntyped>,
    },
    SetAttribute {
        attribute: VertexAttributeUntyped,
        buffer: Option<BufferUntyped>,
        per_instance: bool,
    },
    SetAttributeToConstant {
        attribute: VertexAttributeUntyped,
        value: [f32; 4],
    },
    SetTexture {
        texture_unit: u8,
        texture: Texture,
    },
    SetCubeMap {
        texture_unit: u8,
        cube_map: CubeMap,
    },
}

impl Command {
    #[allow(unused)]
    pub fn name(&self) -> &str {
        match self {
            Command::Clear(_) => "Clear",
            Command::Present => "Present",
            Command::SetPipeline(_) => "SetPipeline",
            Command::Draw { .. } => "Draw",
            Command::SetViewPort { .. } => "SetViewPort",
            Command::SetUniform { .. } => "SetUniform",
            Command::SetUniformBlock { .. } => "SetUniformBlock",
            Command::SetAttribute { .. } => "SetAttribute",
            Command::SetAttributeToConstant { .. } => "SetAttributeToConstant",
            Command::SetTexture { .. } => "SetTexture",
            Command::SetCubeMap { .. } => "SetCubeMap",
        }
    }
}

pub struct CommandBuffer {
    pub(crate) bump_allocator: BumpAllocator,
    pub(crate) commands: Vec<Command>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            bump_allocator: BumpAllocator::new(),
            commands: Vec::new(),
        }
    }

    pub fn begin_render_pass(&mut self, color: Option<kmath::Vec4>) -> RenderPass {
        if let Some(color) = color {
            self.commands.push(Command::Clear(color))
        }
        RenderPass {
            current_pipeline: None,
            command_buffer: self,
        }
    }

    pub fn clear(&mut self) {
        self.bump_allocator.clear();
        self.commands.clear()
    }
}

pub struct RenderPass<'a> {
    current_pipeline: Option<Handle<PipelineInner>>,
    command_buffer: &'a mut CommandBuffer,
}

impl<'a> RenderPass<'a> {
    pub fn set_pipeline(&mut self, pipeline: &Pipeline) {
        self.current_pipeline = Some(pipeline.0.clone());
        self.command_buffer
            .commands
            .push(Command::SetPipeline(pipeline.clone()))
    }

    pub fn set_viewport(&mut self, x: f32, y: f32, width: f32, height: f32) {
        // TODO: x, y, width, and height should be between 0.0 and 1.0
        // assert!(x >= 0.0 && x <= 1.0);
        // assert!(y >= 0.0 && y <= 1.0);
        // assert!(width >= 0.0 && width <= 1.0);
        // assert!(height >= 0.0 && height <= 1.0);
        self.command_buffer.commands.push(Command::SetViewPort {
            x,
            y,
            width,
            height,
        })
    }

    #[inline]
    pub fn set_attribute<D: BufferDataTrait>(
        &mut self,
        vertex_attribute: &VertexAttribute<D>,
        buffer: Option<&Buffer<D>>,
        per_instance: bool,
    ) {
        assert_eq!(
            Some(vertex_attribute.pipeline_index),
            self.current_pipeline
                .as_ref()
                .map(|p| p.inner().program_index),
            "`vertex attribute` is from a pipeline that is not currently bound."
        );
        self.command_buffer.commands.push(Command::SetAttribute {
            attribute: vertex_attribute.untyped(),
            buffer: buffer.map(|b| b.untyped()),
            per_instance,
        })
    }

    pub fn set_attribute_to_constant(
        &mut self,
        attribute: &VertexAttribute<kmath::Vec4>,
        value: [f32; 4],
    ) {
        self.command_buffer
            .commands
            .push(Command::SetAttributeToConstant {
                attribute: attribute.untyped(),
                value,
            })
    }

    #[inline]
    pub fn set_uniform<U: UniformTypeTrait>(&mut self, uniform: &Uniform<U>, data: U) {
        assert_eq!(
            Some(uniform.uniform_info.pipeline_index),
            self.current_pipeline
                .as_ref()
                .map(|p| p.inner().program_index),
            "`vertex attribute` is from a pipeline that is not currently bound."
        );
        let bump_handle = self.command_buffer.bump_allocator.push(data);

        self.command_buffer.commands.push(Command::SetUniform {
            uniform_info: uniform.uniform_info.clone(),
            bump_handle,
        })
    }

    #[inline]
    pub fn set_uniform_block<D: BufferDataTrait>(
        &mut self,
        uniform_block_index: u8,
        buffer: Option<&Buffer<D>>,
    ) {
        if let Some(buffer) = buffer {
            assert_eq!(buffer.handle.inner().buffer_usage, BufferUsage::Data);
        }

        // TODO: Check the uniform block sizes when Draw is called

        self.command_buffer.commands.push(Command::SetUniformBlock {
            uniform_block_index,
            buffer: buffer.map(|b| b.untyped()),
        })
    }

    pub fn set_texture(&mut self, texture_unit: u8, texture: &Texture) {
        assert!(texture_unit < 16);
        self.command_buffer.commands.push(Command::SetTexture {
            texture_unit,
            texture: texture.clone(),
        });
    }

    pub fn set_cube_map(&mut self, texture_unit: u8, cube_map: &CubeMap) {
        assert!(texture_unit < 16);
        self.command_buffer.commands.push(Command::SetCubeMap {
            texture_unit,
            cube_map: cube_map.clone(),
        });
    }

    pub fn draw(
        &mut self,
        index_buffer: Option<&Buffer<[u32; 3]>>,
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
        self.command_buffer.commands.push(Command::Draw {
            index_buffer: index_buffer.cloned(),
            triangle_range,
            instances,
        })
    }
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        self.command_buffer.commands.push(Command::Present);
    }
}
