use crate::*;

use core::ffi::{c_double, c_float, c_int, c_uint, c_void};
use kapp::*;

pub const GL_COLOR_BUFFER_BIT: c_uint = 0x4000;
pub const GL_DEPTH_BUFFER_BIT: c_uint = 0x100;
pub const GL_TEXTURE_2D: c_uint = 0x0DE1;
pub const GL_TEXTURE_CUBE_MAP_POSITIVE_X: c_uint = 0x8515;
pub const GL_RENDERBUFFER: c_uint = 0x8D41;
pub const GL_COMPILE_STATUS: c_uint = 0x8B81;
pub const GL_INFO_LOG_LENGTH: c_uint = 0x8B84;

pub const GL_FRAGMENT_SHADER: c_uint = 0x8B30;
pub const GL_VERTEX_SHADER: c_uint = 0x8B31;
pub const GL_LINK_STATUS: c_uint = 0x8B82;

pub const GL_DEPTH_TEST: GLenum = 0x0B71;
pub const GL_ALWAYS: GLenum = 0x0207;

pub const GL_LEQUAL: GLenum = 0x0203;
pub const GL_LESS: GLenum = 0x0201;

pub const GL_GEQUAL: GLenum = 0x0206;
pub const GL_GREATER: GLenum = 0x0204;

pub const GL_CULL_FACE: GLenum = 0x0B44;
pub const GL_BACK: GLenum = 0x0405;
pub const GL_FRONT: GLenum = 0x0404;
pub const GL_FRONT_AND_BACK: GLenum = 0x0408;

pub const GL_ONE: GLenum = 1;
pub const GL_ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
pub const GL_SRC_ALPHA: GLenum = 0x0302;
pub const GL_BLEND: GLenum = 0x0BE2;
pub const GL_ELEMENT_ARRAY_BUFFER: GLenum = 0x8893;

pub const GL_TRIANGLES: GLenum = 0x0004;
pub const GL_UNSIGNED_INT: GLenum = 0x1405;

type GLint = c_int;
type GLsizei = c_int;
type GLenum = c_uint;
type GLuint = c_uint;
type GLchar = u8;
type GLdouble = c_double;

pub struct GLBackend {
    pub gl_context: GLContext,
    pub clear: unsafe extern "system" fn(mask: c_uint),
    pub clear_color:
        unsafe extern "system" fn(red: c_float, green: c_float, blue: c_float, alpha: c_float),
    pub gen_vertex_arrays: unsafe extern "system" fn(n: GLsizei, arrays: *mut GLuint),
    pub bind_vertex_array: unsafe extern "system" fn(array: GLuint),
    pub gen_textures: unsafe extern "system" fn(n: GLsizei, textures: *mut GLuint),
    pub tex_image_2d: unsafe extern "system" fn(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ),
    pub tex_image_3d: unsafe extern "system" fn(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const c_void,
    ),
    pub bind_texture: unsafe extern "system" fn(target: GLenum, texture: GLuint),
    pub bind_renderbuffer: unsafe extern "system" fn(target: GLenum, renderbuffer: GLuint),
    pub gen_renderbuffers: unsafe extern "system" fn(n: GLsizei, textures: *mut GLuint),
    pub renderbuffer_storage_multisample: unsafe extern "system" fn(
        target: GLenum,
        samples: GLsizei,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
    ),
    pub create_shader: extern "system" fn(type_: GLenum) -> GLuint,
    pub shader_source: unsafe extern "system" fn(
        shader: GLuint,
        count: GLsizei,
        string: *const *const GLchar,
        length: *const GLint,
    ),
    pub compile_shader: unsafe extern "system" fn(shader: GLuint),
    pub get_shader_iv: unsafe extern "system" fn(shader: GLuint, pname: GLenum, params: *mut GLint),
    pub get_shader_info_log: unsafe extern "system" fn(
        shader: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        infoLog: *mut GLchar,
    ),
    pub create_program: unsafe extern "system" fn() -> GLuint,
    pub attach_shader: unsafe extern "system" fn(program: GLuint, shader: GLuint),
    pub link_program: unsafe extern "system" fn(program: GLuint),
    pub get_program_iv:
        unsafe extern "system" fn(program: GLuint, pname: GLenum, params: *mut GLint),
    pub get_program_info_log: unsafe extern "system" fn(
        program: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        infoLog: *mut GLchar,
    ),
    pub delete_shader: unsafe extern "system" fn(shader: GLuint),
    pub delete_program: unsafe extern "system" fn(program: GLuint),
    pub delete_textures: unsafe extern "system" fn(n: GLsizei, textures: *const GLuint),
    pub delete_renderbuffers: unsafe extern "system" fn(n: GLsizei, renderbuffers: *const GLuint),
    pub use_program: unsafe extern "system" fn(program: GLuint),
    pub enable: unsafe extern "system" fn(cap: GLenum),
    pub disable: unsafe extern "system" fn(cap: GLenum),
    pub depth_func: unsafe extern "system" fn(func: GLenum),
    pub cull_face: unsafe extern "system" fn(mode: GLenum),
    pub blend_func: unsafe extern "system" fn(sfactor: GLenum, dfactor: GLenum),
    pub clear_depth: unsafe extern "system" fn(depth: GLdouble),
    pub bind_buffer: unsafe extern "system" fn(target: GLenum, buffer: GLuint),
    pub draw_elements: unsafe extern "system" fn(
        mode: GLenum,
        count: GLsizei,
        type_: GLenum,
        indices: *const core::ffi::c_void,
    ),
}

impl GLBackend {
    pub unsafe fn new(settings: GraphicsContextSettings, initial_window: &kapp::Window) -> Self {
        let mut gl_context_builder = GLContext::builder();
        gl_context_builder.high_resolution_framebuffer(settings.high_resolution_framebuffer);
        gl_context_builder.samples(settings.samples);
        /* gl_context_builder.color_space(settings.color_space.map(|c| match c {
            crate::ColorSpace::SRGB => kapp::ColorSpace::SRGB,
            crate::ColorSpace::DisplayP3 => kapp::ColorSpace::DisplayP3,
        }));
        */

        #[cfg(target_arch = "wasm32")]
        gl_context_builder.webgl2();

        let gl_context = gl_context_builder.build().unwrap();

        // Why doesn't this reliably panic for bogus names? Does the transmute somehow dodge it?
        fn get_f(gl_context: &GLContext, addr: &str) -> *const core::ffi::c_void {
            let address = gl_context.get_proc_address(addr);
            if address as usize == 0 {
                panic!("NULL ADDRESS: {addr}");
            }
            address
        }

        let mut s = Self {
            clear: std::mem::transmute(get_f(&gl_context, "glClear")),
            clear_color: std::mem::transmute(get_f(&gl_context, "glClearColor")),
            gen_vertex_arrays: std::mem::transmute(get_f(&gl_context, "glGenVertexArrays")),
            bind_vertex_array: std::mem::transmute(get_f(&gl_context, "glBindVertexArray")),
            gen_textures: std::mem::transmute(get_f(&gl_context, "glGenTextures")),
            tex_image_2d: std::mem::transmute(get_f(&gl_context, "glTexImage2D")),
            tex_image_3d: std::mem::transmute(get_f(&gl_context, "glTexImage3D")),
            bind_texture: std::mem::transmute(get_f(&gl_context, "glBindTexture")),
            bind_renderbuffer: std::mem::transmute(get_f(&gl_context, "glBindRenderbuffer")),
            gen_renderbuffers: std::mem::transmute(get_f(&gl_context, "glGenRenderbuffers")),
            renderbuffer_storage_multisample: std::mem::transmute(get_f(
                &gl_context,
                "glRenderbufferStorageMultisample",
            )),
            create_shader: std::mem::transmute(get_f(&gl_context, "glCreateShader")),
            shader_source: std::mem::transmute(get_f(&gl_context, "glShaderSource")),
            compile_shader: std::mem::transmute(get_f(&gl_context, "glCompileShader")),
            get_shader_iv: std::mem::transmute(get_f(&gl_context, "glGetShaderiv")),
            get_shader_info_log: std::mem::transmute(get_f(&gl_context, "glGetShaderInfoLog")),
            create_program: std::mem::transmute(get_f(&gl_context, "glCreateProgram")),
            attach_shader: std::mem::transmute(get_f(&gl_context, "glAttachShader")),
            link_program: std::mem::transmute(get_f(&gl_context, "glLinkProgram")),
            get_program_iv: std::mem::transmute(get_f(&gl_context, "glGetProgramiv")),
            get_program_info_log: std::mem::transmute(get_f(&gl_context, "glGetProgramInfoLog")),
            delete_shader: std::mem::transmute(get_f(&gl_context, "glDeleteShader")),
            delete_program: std::mem::transmute(get_f(&gl_context, "glDeleteProgram")),
            delete_textures: std::mem::transmute(get_f(&gl_context, "glDeleteTextures")),
            delete_renderbuffers: std::mem::transmute(get_f(&gl_context, "glDeleteRenderbuffers")),
            use_program: std::mem::transmute(get_f(&gl_context, "glUseProgram")),
            enable: std::mem::transmute(get_f(&gl_context, "glEnable")),
            disable: std::mem::transmute(get_f(&gl_context, "glDisable")),
            depth_func: std::mem::transmute(get_f(&gl_context, "glDepthFunc")),
            cull_face: std::mem::transmute(get_f(&gl_context, "glCullFace")),
            blend_func: std::mem::transmute(get_f(&gl_context, "glBlendFunc")),
            clear_depth: std::mem::transmute(get_f(&gl_context, "glClearDepth")),
            bind_buffer: std::mem::transmute(get_f(&gl_context, "glBindBuffer")),
            draw_elements: std::mem::transmute(get_f(&gl_context, "glDrawElements")),
            gl_context,
        };

        // A vertex array object must always be bound.
        let mut vertex_array = 0;
        (s.gen_vertex_arrays)(1, &mut vertex_array);
        (s.bind_vertex_array)(vertex_array);

        s.gl_context.set_window(Some(&initial_window)).unwrap();
        s.gl_context.resize();

        s
    }
}

impl crate::backend_trait::BackendTrait for GLBackend {
    unsafe fn execute_command_buffer(&mut self, command_buffer: &crate::CommandBuffer) {
        // These are constant across all pipelines.
        (self.enable)(GL_DEPTH_TEST);
        (self.clear_depth)(1.0);

        let mut current_program = None;

        for command in command_buffer.0.iter() {
            match command {
                Command::SetPipeline(pipeline) => {
                    let pipeline = pipeline.0.inner();
                    let program_index = pipeline.program_index;
                    if current_program != Some(program_index) {
                        (self.use_program)(program_index);
                        current_program = Some(program_index);
                    }

                    match pipeline.pipeline_settings.depth_test {
                        crate::DepthTest::AlwaysPass => {
                            (self.depth_func)(GL_ALWAYS);
                        }
                        crate::DepthTest::Less => {
                            (self.depth_func)(GL_LESS);
                        }
                        crate::DepthTest::Greater => {
                            (self.depth_func)(GL_GREATER);
                        }
                        crate::DepthTest::LessOrEqual => {
                            (self.depth_func)(GL_LEQUAL);
                        }
                        crate::DepthTest::GreaterOrEqual => {
                            (self.depth_func)(GL_GEQUAL);
                        }
                    };

                    match pipeline.pipeline_settings.faces_to_render {
                        FacesToRender::Front => {
                            (self.enable)(GL_CULL_FACE);
                            (self.cull_face)(GL_BACK)
                        }
                        FacesToRender::Back => {
                            (self.enable)(GL_CULL_FACE);
                            (self.cull_face)(GL_FRONT)
                        }
                        FacesToRender::FrontAndBack => {
                            (self.disable)(GL_CULL_FACE);
                        }
                        FacesToRender::None => {
                            (self.enable)(GL_CULL_FACE);
                            (self.cull_face)(GL_FRONT_AND_BACK)
                        }
                    };

                    if let Some((source_blend_factor, destination_blend_factor)) =
                        pipeline.pipeline_settings.blending
                    {
                        fn blending_to_gl(blending: BlendFactor) -> GLenum {
                            match blending {
                                BlendFactor::One => GL_ONE,
                                BlendFactor::OneMinusSourceAlpha => GL_ONE_MINUS_SRC_ALPHA,
                                BlendFactor::SourceAlpha => GL_SRC_ALPHA,
                            }
                        }

                        (self.enable)(GL_BLEND);
                        (self.blend_func)(
                            blending_to_gl(source_blend_factor),
                            blending_to_gl(destination_blend_factor),
                        );
                    } else {
                        (self.disable)(GL_BLEND);
                    }
                }
                Command::Draw {
                    triangle_buffer,
                    start_triangle,
                    end_triangle,
                    instances,
                } => {
                    (self.bind_buffer)(
                        GL_ELEMENT_ARRAY_BUFFER,
                        triangle_buffer.as_ref().map_or(0, |t| t.0.inner().index),
                    );
                    if *instances > 1 {
                        todo!()
                    } else {
                        let count = end_triangle - start_triangle;
                        (self.draw_elements)(
                            GL_TRIANGLES,
                            (count * 3) as i32,
                            GL_UNSIGNED_INT,
                            (start_triangle * 3 * std::mem::size_of::<u32>() as u32) as _,
                        );
                    }
                }
                Command::Clear(color) => {
                    (self.clear_color)(color.x, color.y, color.z, color.w);
                    (self.clear)(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT)
                }
                Command::Present => {
                    self.gl_context.swap_buffers();
                }
            }
        }
    }
    unsafe fn new_pipeline(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
        pipeline_settings: PipelineSettings,
    ) -> Result<PipelineInner, String> {
        unsafe fn get_shader_info_log(gl: &GLBackend, shader: u32) -> String {
            let mut length = 0;
            (gl.get_shader_iv)(shader, GL_INFO_LOG_LENGTH, &mut length);
            if length > 0 {
                let mut log: Vec<u8> = vec![0; length as usize];
                (gl.get_shader_info_log)(shader, length, &mut length, log.as_mut_ptr());
                log.truncate(length as usize);
                String::from_utf8(log).unwrap()
            } else {
                String::from("")
            }
        }
        unsafe fn compile_shader(
            gl: &GLBackend,
            shader_type: GLenum,
            source: &str,
        ) -> Result<u32, String> {
            let version = "#version 410";

            let source = &format!("{}\n{}", version, source);
            let shader = (gl.create_shader)(shader_type);
            (gl.shader_source)(shader, 1, &(source.as_ptr()), &(source.len() as i32));
            (gl.compile_shader)(shader);

            let mut status = 0;
            (gl.get_shader_iv)(shader, GL_COMPILE_STATUS, &mut status);
            let success = 1 == status;

            if !success {
                Err(get_shader_info_log(gl, shader))
            } else {
                Ok(shader)
            }
        }

        unsafe fn get_program_info_log(gl: &GLBackend, program: u32) -> String {
            let mut length = 0;
            (gl.get_program_iv)(program, GL_INFO_LOG_LENGTH, &mut length);
            if length > 0 {
                let mut log: Vec<u8> = vec![0; length as usize];

                (gl.get_program_info_log)(program, length, &mut length, log.as_mut_ptr());
                log.truncate(length as usize);
                String::from_utf8(log).unwrap()
            } else {
                String::from("")
            }
        }

        let vertex_function = compile_shader(self, GL_VERTEX_SHADER, vertex_source)?;
        let fragment_function = compile_shader(self, GL_FRAGMENT_SHADER, fragment_source)?;

        let program = (self.create_program)();
        (self.attach_shader)(program, vertex_function);
        (self.attach_shader)(program, fragment_function);
        (self.link_program)(program);

        (self.delete_shader)(vertex_function);
        (self.delete_shader)(fragment_function);

        let mut status = 0;
        (self.get_program_iv)(program, GL_LINK_STATUS, &mut status);
        let success = 1 == status;

        if !success {
            Err(get_program_info_log(self, program))
        } else {
            Ok(PipelineInner {
                program_index: program,
                pipeline_settings,
            })
        }
    }

    unsafe fn delete_pipeline(&mut self, pipeline_inner: PipelineInner) {
        (self.delete_program)(pipeline_inner.program_index);
    }

    unsafe fn new_texture(
        &mut self,
        width: usize,
        height: usize,
        depth: usize,
        pixel_format_in: PixelFormat,
        settings: TextureSettings,
    ) -> TextureInner {
        if settings.msaa_samples == 0 {
            let mut texture_index = 0;
            (self.gen_textures)(1, &mut texture_index);

            let texture = TextureInner {
                index: texture_index,
                texture_type: TextureType::Texture,
                mip: 0,
            };
            {
                let (target, texture_index) = match texture.texture_type {
                    TextureType::Texture => (GL_TEXTURE_2D, texture_index),
                    TextureType::CubeMap { face } => {
                        (GL_TEXTURE_CUBE_MAP_POSITIVE_X + face as u32, texture_index)
                    }
                    TextureType::RenderBuffer { .. } => {
                        panic!("For now textures with MSAA cannot be updated by a call to `update_texture`")
                    }
                    TextureType::DefaultFramebuffer => {
                        panic!("Cannot update default framebuffer")
                    }
                    TextureType::None => {
                        panic!()
                    }
                };
                let (pixel_format, inner_pixel_format, type_) =
                    crate::gl_shared::pixel_format_to_gl_format_and_inner_format_and_type(
                        pixel_format_in,
                        settings.srgb,
                    );
                (self.bind_texture)(target, texture_index);

                if depth > 1 {
                    (self.tex_image_3d)(
                        target,
                        0,                         /* mip level */
                        inner_pixel_format as i32, // Internal format, how the GPU stores these pixels.
                        width as i32,
                        height as i32,
                        depth as i32,
                        0,            /* border: must be 0 */
                        pixel_format, // This doesn't necessarily need to match the internal_format
                        type_,
                        0 as _,
                    );
                } else {
                    (self.tex_image_2d)(
                        target,
                        0,                         /* mip level */
                        inner_pixel_format as i32, // Internal format, how the GPU stores these pixels.
                        width as i32,
                        height as i32,
                        0,            /* border: must be 0 */
                        pixel_format, // This doesn't necessarily need to match the internal_format
                        type_,
                        0 as _,
                    );
                }
            }

            texture
        } else {
            let mut renderbuffer = 0;
            (self.gen_textures)(1, &mut renderbuffer);
            (self.bind_renderbuffer)(GL_RENDERBUFFER, renderbuffer);

            let (_pixel_format, inner_pixel_format, _type_) =
                crate::gl_shared::pixel_format_to_gl_format_and_inner_format_and_type(
                    pixel_format_in,
                    settings.srgb,
                );
            (self.renderbuffer_storage_multisample)(
                GL_RENDERBUFFER,
                settings.msaa_samples as i32,
                inner_pixel_format,
                width as i32,
                height as i32,
            );

            TextureInner {
                index: renderbuffer,
                texture_type: TextureType::RenderBuffer,
                mip: 0,
            }
        }
    }

    unsafe fn delete_texture(&mut self, texture_inner: TextureInner) {
        unsafe {
            match texture_inner.texture_type {
                TextureType::Texture => (self.delete_textures)(1, &texture_inner.index),
                TextureType::CubeMap { .. } => {}
                TextureType::RenderBuffer => {
                    (self.delete_renderbuffers)(1, &texture_inner.index);
                }
                TextureType::DefaultFramebuffer => panic!("Cannot delete default framebuffer"),
                TextureType::None => {}
            };
        }
    }

    unsafe fn new_triangle_buffer(&mut self, indices: &[[u32; 3]]) -> TriangleBufferInner {
        todo!()
    }

    unsafe fn delete_triangle_buffer(&mut self, triangle_buffer_inner: TriangleBufferInner) {
        todo!()
    }
}
