use crate::{
    bump_allocator::{BumpAllocator, BumpHandle},
    *,
};

pub(crate) enum Command {
    BeginRenderPass {
        clear_color: kmath::Vec4,
    },
    Present,
    SetPipeline {
        pipeline_index: u32,
        // TODO: Remove the need for PipelineSettings here.
        pipeline_settings: PipelineSettings,
    },
    Draw {
        index_buffer_index: Option<u32>,
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
        texture_index: u32,
    },
    SetCubeMap {
        texture_unit: u8,
        cube_map_index: u32,
    },
}

impl Command {
    #[allow(unused)]
    pub fn name(&self) -> &str {
        match self {
            Command::BeginRenderPass { .. } => "BeginRenderPass",
            Command::Present => "Present",
            Command::SetPipeline { .. } => "SetPipeline",
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
    pipelines: Vec<Pipeline>,
    buffers: Vec<BufferUntyped>,
    textures: Vec<Texture>,
    cube_maps: Vec<CubeMap>,
}

impl CommandBuffer {
    pub(crate) fn new() -> Self {
        Self {
            bump_allocator: BumpAllocator::new(),
            pipelines: Vec::new(),
            commands: Vec::new(),
            buffers: Vec::new(),
            textures: Vec::new(),
            cube_maps: Vec::new(),
        }
    }

    pub fn begin_render_pass(&mut self, clear_color: Option<kmath::Vec4>) -> RenderPass {
        if let Some(clear_color) = clear_color {
            self.commands.push(Command::BeginRenderPass { clear_color })
        }
        RenderPass {
            current_pipeline: None,
            command_buffer: self,
        }
    }

    pub fn clear(&mut self) {
        self.bump_allocator.clear();
        self.commands.clear();
        self.pipelines.clear();
        self.buffers.clear();
        self.textures.clear();
        self.cube_maps.clear();
    }

    pub fn present(&mut self) {
        self.commands.push(Command::Present);
    }
}

pub struct RenderPass<'a> {
    current_pipeline: Option<Handle<PipelineInner>>,
    command_buffer: &'a mut CommandBuffer,
}

impl<'a> RenderPass<'a> {
    pub fn set_pipeline(&mut self, pipeline: &Pipeline) {
        self.current_pipeline = Some(pipeline.0.clone());
        self.command_buffer.pipelines.push(pipeline.clone());
        self.command_buffer.commands.push(Command::SetPipeline {
            pipeline_index: pipeline.0.inner().program_index,
            pipeline_settings: pipeline.0.inner().pipeline_settings,
        });
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
        if let Some(buffer) = buffer {
            self.command_buffer.buffers.push(buffer.untyped());
        }
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

        if let Some(buffer) = buffer {
            self.command_buffer.buffers.push(buffer.untyped());
        }

        // TODO: Check the uniform block sizes when Draw is called

        self.command_buffer.commands.push(Command::SetUniformBlock {
            uniform_block_index,
            buffer: buffer.map(|b| b.untyped()),
        })
    }

    pub fn set_texture(&mut self, texture_unit: u8, texture: &Texture) {
        assert!(texture_unit < 16);
        self.command_buffer.textures.push(texture.clone());
        self.command_buffer.commands.push(Command::SetTexture {
            texture_unit,
            texture_index: texture.0.inner().index,
        });
    }

    pub fn set_cube_map(&mut self, texture_unit: u8, cube_map: &CubeMap) {
        assert!(texture_unit < 16);
        self.command_buffer.cube_maps.push(cube_map.clone());
        self.command_buffer.commands.push(Command::SetCubeMap {
            texture_unit,
            cube_map_index: cube_map.0.inner().index,
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
            );

            self.command_buffer.buffers.push(index_buffer.untyped());
        }

        self.command_buffer.commands.push(Command::Draw {
            index_buffer_index: index_buffer.map(|i| i.handle.inner().index),
            triangle_range,
            instances,
        })
    }
}
