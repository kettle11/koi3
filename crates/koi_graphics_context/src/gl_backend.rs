use crate::gl_shared::*;
use crate::*;

use core::ffi::{c_double, c_float, c_int, c_uchar, c_uint, c_void};
use kapp::*;

pub(crate) type GLboolean = c_uchar;
pub(crate) type GLint = c_int;
pub(crate) type GLsizei = c_int;
pub(crate) type GLenum = c_uint;
pub(crate) type GLuint = c_uint;
pub(crate) type GLchar = u8;
pub(crate) type GLdouble = c_double;
pub(crate) type GLsizeiptr = isize;
pub(crate) type GLintptr = isize;
pub(crate) type GLfloat = f32;

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
    pub draw_elements_instanced: unsafe extern "system" fn(
        mode: GLenum,
        count: GLsizei,
        type_: GLenum,
        indices: *const core::ffi::c_void,
        instancecount: GLsizei,
    ),
    pub gen_buffers: unsafe extern "system" fn(n: GLsizei, buffers: *mut GLuint),
    pub buffer_data: unsafe extern "system" fn(
        target: GLenum,
        size: GLsizeiptr,
        data: *const core::ffi::c_void,
        usage: GLenum,
    ),
    pub delete_buffers: unsafe extern "system" fn(n: GLsizei, buffers: *const GLuint),
    pub viewport: unsafe extern "system" fn(x: GLint, y: GLint, width: GLsizei, height: GLsizei),
    pub draw_arrays: unsafe extern "system" fn(mode: GLenum, first: GLint, count: GLsizei),
    pub get_active_uniform_block_name: unsafe extern "system" fn(
        program: GLuint,
        uniformBlockIndex: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        uniformBlockName: *mut GLchar,
    ),
    pub get_active_uniform_block_iv: unsafe extern "system" fn(
        program: GLuint,
        uniformBlockIndex: GLuint,
        pname: GLenum,
        params: *mut GLint,
    ),
    pub uniform_block_binding: unsafe extern "system" fn(
        program: GLuint,
        uniformBlockIndex: GLuint,
        uniformBlockBinding: GLuint,
    ),
    pub get_active_uniform: unsafe extern "system" fn(
        program: GLuint,
        index: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        size: *mut GLint,
        type_: *mut GLenum,
        name: *mut GLchar,
    ),
    pub get_uniform_location:
        unsafe extern "system" fn(program: GLuint, name: *const GLchar) -> GLint,
    pub get_active_attrib: unsafe extern "system" fn(
        program: GLuint,
        index: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        size: *mut GLint,
        type_: *mut GLenum,
        name: *mut GLchar,
    ),
    pub get_attrib_location:
        unsafe extern "system" fn(program: GLuint, name: *const GLchar) -> GLint,
    pub vertex_attrib_pointer: unsafe extern "system" fn(
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const core::ffi::c_void,
    ),
    pub vertex_attrib_divisor: unsafe extern "system" fn(index: GLuint, divisor: GLuint),
    pub enable_vertex_attrib_array: unsafe extern "system" fn(index: GLuint),
    pub disable_vertex_attrib_array: unsafe extern "system" fn(index: GLuint),
    pub bind_buffer_range: unsafe extern "system" fn(
        target: GLenum,
        index: GLuint,
        buffer: GLuint,
        offset: GLintptr,
        size: GLsizeiptr,
    ),
    pub tex_sub_image_2d: unsafe extern "system" fn(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const std::ffi::c_void,
    ),
    pub tex_sub_image_3d: unsafe extern "system" fn(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        zoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const std::ffi::c_void,
    ),
    pub tex_parameter_i32: unsafe extern "system" fn(target: GLenum, pname: GLenum, param: GLint),
    pub generate_mipmap: unsafe extern "system" fn(target: GLenum),
    pub active_texture: unsafe extern "system" fn(texture: GLenum),
    pub uniform_1fv:
        unsafe extern "system" fn(location: GLint, count: GLsizei, value: *const GLfloat),
    pub uniform_2fv:
        unsafe extern "system" fn(location: GLint, count: GLsizei, value: *const GLfloat),
    pub uniform_3fv:
        unsafe extern "system" fn(location: GLint, count: GLsizei, value: *const GLfloat),
    pub uniform_4fv:
        unsafe extern "system" fn(location: GLint, count: GLsizei, value: *const GLfloat),
    pub uniform_matrix_4fv: unsafe extern "system" fn(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    ),
    pub uniform_1iv:
        unsafe extern "system" fn(location: GLint, count: GLsizei, value: *const GLint),
    pub uniform_1uiv:
        unsafe extern "system" fn(location: GLint, count: GLsizei, value: *const GLuint),
    pub vertex_attrib_4f:
        unsafe extern "system" fn(index: GLuint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat),
    pub get_error: unsafe extern "system" fn() -> GLenum,
    pub uniform_1i: unsafe extern "system" fn(location: GLint, v0: GLint),
}

impl GLBackend {
    pub unsafe fn new(settings: GraphicsContextSettings) -> Self {
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

        let s = Self {
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
            draw_elements_instanced: std::mem::transmute(get_f(
                &gl_context,
                "glDrawElementsInstanced",
            )),
            gen_buffers: std::mem::transmute(get_f(&gl_context, "glGenBuffers")),
            buffer_data: std::mem::transmute(get_f(&gl_context, "glBufferData")),
            delete_buffers: std::mem::transmute(get_f(&gl_context, "glDeleteBuffers")),
            viewport: std::mem::transmute(get_f(&gl_context, "glViewport")),
            draw_arrays: std::mem::transmute(get_f(&gl_context, "glDrawArrays")),
            get_active_uniform_block_name: std::mem::transmute(get_f(
                &gl_context,
                "glGetActiveUniformBlockName",
            )),
            get_active_uniform_block_iv: std::mem::transmute(get_f(
                &gl_context,
                "glGetActiveUniformBlockiv",
            )),
            uniform_block_binding: std::mem::transmute(get_f(&gl_context, "glUniformBlockBinding")),
            get_active_uniform: std::mem::transmute(get_f(&gl_context, "glGetActiveUniform")),
            get_uniform_location: std::mem::transmute(get_f(&gl_context, "glGetUniformLocation")),
            get_active_attrib: std::mem::transmute(get_f(&gl_context, "glGetActiveAttrib")),
            get_attrib_location: std::mem::transmute(get_f(&gl_context, "glGetAttribLocation")),
            vertex_attrib_pointer: std::mem::transmute(get_f(&gl_context, "glVertexAttribPointer")),
            vertex_attrib_divisor: std::mem::transmute(get_f(&gl_context, "glVertexAttribDivisor")),
            enable_vertex_attrib_array: std::mem::transmute(get_f(
                &gl_context,
                "glEnableVertexAttribArray",
            )),
            disable_vertex_attrib_array: std::mem::transmute(get_f(
                &gl_context,
                "glDisableVertexAttribArray",
            )),
            bind_buffer_range: std::mem::transmute(get_f(&gl_context, "glBindBufferRange")),
            tex_sub_image_2d: std::mem::transmute(get_f(&gl_context, "glTexSubImage2D")),
            tex_sub_image_3d: std::mem::transmute(get_f(&gl_context, "glTexSubImage3D")),
            tex_parameter_i32: std::mem::transmute(get_f(&gl_context, "glTexParameteri")),
            generate_mipmap: std::mem::transmute(get_f(&gl_context, "glGenerateMipmap")),
            active_texture: std::mem::transmute(get_f(&gl_context, "glActiveTexture")),
            uniform_1fv: std::mem::transmute(get_f(&gl_context, "glUniform1fv")),
            uniform_2fv: std::mem::transmute(get_f(&gl_context, "glUniform2fv")),
            uniform_3fv: std::mem::transmute(get_f(&gl_context, "glUniform3fv")),
            uniform_4fv: std::mem::transmute(get_f(&gl_context, "glUniform4fv")),
            uniform_matrix_4fv: std::mem::transmute(get_f(&gl_context, "glUniformMatrix4fv")),
            uniform_1iv: std::mem::transmute(get_f(&gl_context, "glUniform1iv")),
            uniform_1uiv: std::mem::transmute(get_f(&gl_context, "glUniform1uiv")),
            vertex_attrib_4f: std::mem::transmute(get_f(&gl_context, "glVertexAttrib4f")),
            get_error: std::mem::transmute(get_f(&gl_context, "glGetError")),
            uniform_1i: std::mem::transmute(get_f(&gl_context, "glUniform1i")),
            gl_context,
        };

        // A vertex array object must always be bound.
        let mut vertex_array = 0;
        (s.gen_vertex_arrays)(1, &mut vertex_array);
        (s.bind_vertex_array)(vertex_array);

        s
    }
}

impl GLBackend {
    pub fn check_for_error(&self) {
        unsafe {
            let error = (self.get_error)();
            if error != 0 {
                panic!("GL ERROR: {:?}", error);
            }
        }
    }
}

impl crate::backend_trait::BackendTrait for GLBackend {
    unsafe fn set_main_window(&mut self, window: &kapp::Window) {
        self.gl_context.set_window(Some(&window)).unwrap();
        self.gl_context.resize();
    }

    unsafe fn execute_command_buffer(
        &mut self,
        command_buffer: &crate::CommandBuffer,
        buffer_sizes: &[u32],
        texture_sizes: &[(u32, u32, u32)],
    ) {
        self.gl_context.resize();

        // These are constant across all pipelines.
        (self.enable)(DEPTH_TEST);
        (self.clear_depth)(1.0);

        let mut current_program = None;

        for command in command_buffer.commands.iter() {
            // println!("COMMAND: {:?}", command.name());
            // self.check_for_error();
            match command {
                Command::SetPipeline {
                    pipeline_index,
                    pipeline_settings,
                } => {
                    if current_program != Some(pipeline_index) {
                        (self.use_program)(*pipeline_index);
                        current_program = Some(pipeline_index);
                    }

                    match pipeline_settings.depth_test {
                        crate::DepthTest::AlwaysPass => {
                            (self.depth_func)(ALWAYS);
                        }
                        crate::DepthTest::Less => {
                            (self.depth_func)(LESS);
                        }
                        crate::DepthTest::Greater => {
                            (self.depth_func)(GREATER);
                        }
                        crate::DepthTest::LessOrEqual => {
                            (self.depth_func)(LEQUAL);
                        }
                        crate::DepthTest::GreaterOrEqual => {
                            (self.depth_func)(GEQUAL);
                        }
                    };

                    match pipeline_settings.faces_to_render {
                        FacesToRender::Front => {
                            (self.enable)(CULL_FACE);
                            (self.cull_face)(BACK)
                        }
                        FacesToRender::Back => {
                            (self.enable)(CULL_FACE);
                            (self.cull_face)(FRONT)
                        }
                        FacesToRender::FrontAndBack => {
                            (self.disable)(CULL_FACE);
                        }
                        FacesToRender::None => {
                            (self.enable)(CULL_FACE);
                            (self.cull_face)(FRONT_AND_BACK)
                        }
                    };

                    if let Some((source_blend_factor, destination_blend_factor)) =
                        pipeline_settings.blending
                    {
                        fn blending_to_gl(blending: BlendFactor) -> GLenum {
                            match blending {
                                BlendFactor::One => ONE,
                                BlendFactor::OneMinusSourceAlpha => ONE_MINUS_SRC_ALPHA,
                                BlendFactor::SourceAlpha => SRC_ALPHA,
                            }
                        }

                        (self.enable)(BLEND);
                        (self.blend_func)(
                            blending_to_gl(source_blend_factor),
                            blending_to_gl(destination_blend_factor),
                        );
                    } else {
                        (self.disable)(BLEND);
                    }
                }
                &Command::SetViewPort {
                    x,
                    y,
                    width,
                    height,
                } => {
                    // TODO: x, y, width, and height should be passed in from 0 to 1.0 instead.
                    (self.viewport)(x as i32, y as i32, width as i32, height as i32)
                }
                Command::SetUniform {
                    uniform_info,
                    bump_handle,
                } => {
                    if let Some(location) = uniform_info.location {
                        let location = location as i32;
                        let data = command_buffer.bump_allocator.get_raw_bytes(*bump_handle);

                        let data = data.as_ptr();

                        // TODO: It may be faster to use the non array types when the value is 1.
                        match uniform_info.uniform_type {
                            UniformType::Int(n) => (self.uniform_1iv)(location, n as _, data as _),
                            UniformType::UInt(n) => {
                                (self.uniform_1uiv)(location, n as _, data as _)
                            }
                            UniformType::Float(n) => {
                                (self.uniform_1fv)(location, n as _, data as _)
                            }
                            UniformType::Vec2(n) => (self.uniform_2fv)(location, n as _, data as _),
                            UniformType::Vec3(n) => (self.uniform_3fv)(location, n as _, data as _),
                            UniformType::Vec4(n) => (self.uniform_4fv)(location, n as _, data as _),
                            UniformType::Mat4(n) => {
                                (self.uniform_matrix_4fv)(location, n as _, 0, data as _)
                            }
                            UniformType::Sampler2d
                            | UniformType::Sampler3d
                            | UniformType::SamplerCube => {
                                (self.uniform_1iv)(location, 1, data as _)
                            }
                        }
                    }
                }
                Command::SetUniformBlock {
                    uniform_block_index,
                    buffer,
                } => {
                    if let Some(buffer) = buffer {
                        let size_bytes = buffer_sizes[buffer.handle.inner().index as usize];
                        (self.bind_buffer_range)(
                            UNIFORM_BUFFER,
                            *uniform_block_index as _, // Index
                            buffer.handle.inner().index,
                            0,
                            size_bytes as _,
                        );
                    }
                }
                Command::SetAttribute {
                    attribute,
                    buffer,
                    per_instance,
                } => {
                    if let Some(info) = &attribute.info {
                        if let Some(buffer) = buffer {
                            (self.bind_buffer)(ARRAY_BUFFER, buffer.handle.inner().index);

                            let attribute_index = info.location;
                            let byte_size = info.byte_size;

                            for i in 0..(info.byte_size / 16).max(1) {
                                (self.vertex_attrib_pointer)(
                                    attribute_index + i as u32,    // Index
                                    (byte_size as i32 / 4).min(4), // Number of components. It's assumed that components are always 32 bit.
                                    FLOAT,            // TODO: This shouldn't always be float.
                                    0,                // false
                                    byte_size as i32, // 0 means to assume tightly packed
                                    (i * 16) as _,    // Offset
                                );

                                (self.vertex_attrib_divisor)(
                                    attribute_index + i,
                                    if *per_instance { 1 } else { 0 },
                                );

                                (self.enable_vertex_attrib_array)(attribute_index + i);
                            }
                        } else {
                            (self.disable_vertex_attrib_array)(info.location);
                        }
                    }
                }
                Command::SetAttributeToConstant { attribute, value } => {
                    if let Some(info) = &attribute.info {
                        (self.disable_vertex_attrib_array)(info.location);
                        (self.vertex_attrib_4f)(
                            info.location,
                            value[0],
                            value[1],
                            value[2],
                            value[3],
                        );
                    }
                }
                Command::SetTexture {
                    texture_unit,
                    texture_index,
                } => {
                    let is_3d = texture_sizes[*texture_index as usize].2 > 1;
                    // self.gl.uniform_1_i32(Some(uniform_location), unit as i32);
                    (self.active_texture)(TEXTURE0 + *texture_unit as u32);
                    (self.bind_texture)(
                        if is_3d { TEXTURE_3D } else { TEXTURE_2D },
                        *texture_index,
                    );
                }
                Command::SetCubeMap {
                    texture_unit,
                    cube_map_index,
                } => {
                    // self.gl.uniform_1_i32(Some(uniform_location), unit as i32);
                    (self.active_texture)(TEXTURE0 + *texture_unit as u32);
                    (self.bind_texture)(TEXTURE_CUBE_MAP, *cube_map_index);
                }
                Command::Draw {
                    index_buffer_index,
                    triangle_range,
                    instances,
                } => {
                    let count = triangle_range.end - triangle_range.start;
                    let count_vertices = (count * 3) as i32;

                    if let Some(index_buffer_index) = index_buffer_index {
                        (self.bind_buffer)(ELEMENT_ARRAY_BUFFER, *index_buffer_index);

                        let offset_bytes =
                            (triangle_range.start * 3 * std::mem::size_of::<u32>() as u32) as _;

                        if *instances > 1 {
                            (self.draw_elements_instanced)(
                                TRIANGLES,
                                count_vertices,
                                UNSIGNED_INT,
                                offset_bytes,
                                *instances as _,
                            );
                        } else {
                            (self.draw_elements)(
                                TRIANGLES,
                                count_vertices,
                                UNSIGNED_INT,
                                offset_bytes,
                            );
                        }
                    } else if *instances > 1 {
                        todo!()
                    } else {
                        (self.draw_arrays)(TRIANGLES, 0, count_vertices);
                    }
                }
                Command::BeginRenderPass { clear_color } => {
                    (self.clear_color)(clear_color.x, clear_color.y, clear_color.z, clear_color.w);
                    (self.clear)(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT)
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
        self.check_for_error();

        unsafe fn get_shader_info_log(gl: &GLBackend, shader: u32) -> String {
            let mut length = 0;
            (gl.get_shader_iv)(shader, INFO_LOG_LENGTH, &mut length);
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
            (gl.get_shader_iv)(shader, COMPILE_STATUS, &mut status);
            let success = 1 == status;

            if success {
                Ok(shader)
            } else {
                Err(get_shader_info_log(gl, shader))
            }
        }

        unsafe fn get_program_info_log(gl: &GLBackend, program: u32) -> String {
            let mut length = 0;
            (gl.get_program_iv)(program, INFO_LOG_LENGTH, &mut length);
            if length > 0 {
                let mut log: Vec<u8> = vec![0; length as usize];

                (gl.get_program_info_log)(program, length, &mut length, log.as_mut_ptr());
                log.truncate(length as usize);
                String::from_utf8(log).unwrap()
            } else {
                String::from("")
            }
        }

        let vertex_function = compile_shader(self, VERTEX_SHADER, vertex_source)?;
        let fragment_function = compile_shader(self, FRAGMENT_SHADER, fragment_source)?;

        let program = (self.create_program)();
        (self.attach_shader)(program, vertex_function);
        (self.attach_shader)(program, fragment_function);
        (self.link_program)(program);

        (self.delete_shader)(vertex_function);
        (self.delete_shader)(fragment_function);

        let mut status = 0;
        (self.get_program_iv)(program, LINK_STATUS, &mut status);
        let success = 1 == status;

        pub unsafe fn get_uniform_block_name_and_size(
            gl: &GLBackend,
            program: GLuint,
            uniform_block_index: u32,
        ) -> Option<(String, u32)> {
            let mut max_name_length = 0;
            (gl.get_program_iv)(
                program,
                ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
                &mut max_name_length,
            );

            // Todo: Don't allocate a new vec here each time.
            let mut name: Vec<u8> = vec![0; max_name_length as usize];
            let mut length = 0;

            (gl.get_active_uniform_block_name)(
                program,
                uniform_block_index,
                max_name_length,
                &mut length,
                name.as_mut_ptr(),
            );
            name.truncate(length as usize);
            let name = String::from_utf8(name).unwrap();

            let mut size_bytes: i32 = 0;
            (gl.get_active_uniform_block_iv)(
                program,
                uniform_block_index,
                UNIFORM_BLOCK_DATA_SIZE,
                &mut size_bytes,
            );
            Some((name, size_bytes as u32))
        }

        if success {
            let mut uniforms = std::collections::HashMap::new();
            let mut uniform_blocks = Vec::new();
            let mut vertex_attributes = std::collections::HashMap::new();

            fn get_id(name: &str) -> Option<u32> {
                name[2..name.find('_')?].parse().ok()
            }

            (self.use_program)(program);

            // First read all uniforms
            {
                unsafe fn get_uniform_info(
                    gl: &GLBackend,
                    program: u32,
                    index: u32,
                ) -> Option<(String, UniformInfo)> {
                    let mut uniform_max_length = 0;
                    (gl.get_program_iv)(
                        program,
                        ACTIVE_UNIFORM_MAX_LENGTH,
                        &mut uniform_max_length,
                    );

                    // Todo: Don't allocate a new vec here each time.
                    let mut name: Vec<u8> = vec![0; uniform_max_length as usize];
                    let mut length = 0;
                    let mut size_members = 0;
                    let mut uniform_type = 0;

                    (gl.get_active_uniform)(
                        program,
                        index,
                        uniform_max_length,
                        &mut length,
                        &mut size_members,
                        &mut uniform_type,
                        name.as_mut_ptr(),
                    );
                    name.truncate(length as usize);
                    let name = String::from_utf8(name).unwrap();

                    let uniform_location =
                        (gl.get_uniform_location)(program, name.as_ptr() as *const u8);
                    let location = if uniform_location < 0 {
                        None
                    } else {
                        Some(uniform_location as u32)
                    };
                    let size_members = size_members as u8;
                    let uniform_type = gl_uniform_type_to_uniform_type(uniform_type, size_members);

                    Some((
                        name,
                        UniformInfo {
                            pipeline_index: program,
                            uniform_type,
                            location: Some(location?),
                        },
                    ))
                }

                let mut uniform_count = 0;
                (self.get_program_iv)(program, ACTIVE_UNIFORMS, &mut uniform_count);
                let uniform_count = uniform_count as u32;

                for i in 0..uniform_count {
                    let uniform_info = get_uniform_info(self, program, i);

                    if let Some((name, uniform_info)) = uniform_info {
                        match uniform_info.uniform_type {
                            UniformType::Sampler2d
                            | UniformType::Sampler3d
                            | UniformType::SamplerCube => {
                                // Bind the location once
                                if let Some(location) = uniform_info.location {
                                    let id = get_id(&name).expect(&name) as i32;

                                    self.check_for_error();

                                    (self.uniform_1i)(location as _, id);
                                    self.check_for_error();
                                }
                            }
                            _ => {}
                        }

                        uniforms.insert(name, uniform_info);
                    }
                }
            }

            // Next read all uniform blocks.
            {
                let mut uniform_block_count = 0;
                (self.get_program_iv)(program, ACTIVE_UNIFORM_BLOCKS, &mut uniform_block_count);
                let uniform_block_count = uniform_block_count as u32;

                let mut max_name_length = 0;
                (self.get_program_iv)(
                    program,
                    ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
                    &mut max_name_length,
                );

                for i in 0..uniform_block_count {
                    let (name, size_bytes) =
                        get_uniform_block_name_and_size(self, program, i).unwrap();
                    let binding_location = get_id(&name).ok_or(
                     "Uniform blocks must be formatted with ub[binding_index]_name. EX: ub0_scene_info."
                    )?;
                    (self.uniform_block_binding)(program, i, binding_location);
                    uniform_blocks.push(UniformBlockInfo {
                        size_bytes,
                        location: i,
                    });
                }
            }

            // Read all vertex attributes
            {
                unsafe fn get_attribute_info(
                    gl: &GLBackend,
                    program: u32,
                    index: u32,
                ) -> Option<(String, VertexAttributeInfo)> {
                    let mut attribute_max_length = 0;
                    (gl.get_program_iv)(
                        program,
                        ACTIVE_ATTRIBUTE_MAX_LENGTH,
                        &mut attribute_max_length,
                    );
                    let mut name: Vec<u8> = vec![0; attribute_max_length as usize];
                    let mut length = 0;
                    let mut size_members = 0;
                    let mut attribute_type = 0;

                    (gl.get_active_attrib)(
                        program,
                        index,
                        attribute_max_length,
                        &mut length,
                        &mut size_members,
                        &mut attribute_type,
                        name.as_mut_ptr(),
                    );

                    name.truncate(length as usize);
                    let name = String::from_utf8(name).unwrap();

                    let byte_size = match attribute_type {
                        FLOAT => 4,
                        FLOAT_VEC2 => 8,
                        FLOAT_VEC3 => 12,
                        FLOAT_VEC4 => 16,
                        FLOAT_MAT4 => 64,
                        _ => return None,
                    };

                    let location = (gl.get_attrib_location)(program, name.as_ptr() as *const u8);

                    Some((
                        name,
                        VertexAttributeInfo {
                            byte_size,
                            location: location as u32,
                        },
                    ))
                }
                let mut count = 0;
                (self.get_program_iv)(program, ACTIVE_ATTRIBUTES, &mut count);
                let vertex_attribute_count = count as u32;

                for i in 0..vertex_attribute_count {
                    if let Some((name, attribute_info)) = get_attribute_info(self, program, i) {
                        vertex_attributes.insert(name, attribute_info);
                    }
                }
            }

            (self.use_program)(0);

            Ok(PipelineInner {
                program_index: program,
                pipeline_settings,
                uniforms,
                uniform_blocks,
                vertex_attributes,
            })
        } else {
            Err(get_program_info_log(self, program))
        }
    }

    unsafe fn delete_pipeline(&mut self, pipeline_inner: PipelineInner) {
        (self.delete_program)(pipeline_inner.program_index);
    }

    unsafe fn new_texture(
        &mut self,
        width: u32,
        height: u32,
        depth: u32,
        pixel_format_in: PixelFormat,
        settings: TextureSettings,
    ) -> TextureInner {
        let result = if settings.msaa_samples == 0 {
            let mut texture_index = 0;
            (self.gen_textures)(1, &mut texture_index);

            let texture = TextureInner {
                index: texture_index,
                texture_type: TextureType::Texture,
                pixel_format: pixel_format_in,
                mip: 0,
            };
            {
                let (target, texture_index) = match texture.texture_type {
                    TextureType::Texture => (
                        if depth > 1 { TEXTURE_3D } else { TEXTURE_2D },
                        texture_index,
                    ),
                    TextureType::CubeMapFace { face } => {
                        (TEXTURE_CUBE_MAP_POSITIVE_X + face as u32, texture_index)
                    }
                    TextureType::RenderBuffer { .. } => {
                        panic!("For now textures with MSAA cannot be updated by a call to `update_texture`")
                    }
                    TextureType::DefaultFramebuffer => {
                        panic!("Cannot update default framebuffer")
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
            (self.bind_renderbuffer)(RENDERBUFFER, renderbuffer);

            let (_pixel_format, inner_pixel_format, _type_) =
                crate::gl_shared::pixel_format_to_gl_format_and_inner_format_and_type(
                    pixel_format_in,
                    settings.srgb,
                );
            (self.renderbuffer_storage_multisample)(
                RENDERBUFFER,
                settings.msaa_samples as i32,
                inner_pixel_format,
                width as i32,
                height as i32,
            );

            TextureInner {
                index: renderbuffer,
                texture_type: TextureType::RenderBuffer,
                mip: 0,
                pixel_format: pixel_format_in,
            }
        };
        self.check_for_error();
        result
    }

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
    ) {
        let (target, texture_index) = match texture.texture_type {
            TextureType::Texture => (
                if depth > 1 { TEXTURE_3D } else { TEXTURE_2D },
                texture.index,
            ),
            TextureType::CubeMapFace { face } => {
                (TEXTURE_CUBE_MAP_POSITIVE_X + face as u32, texture.index)
            }
            TextureType::RenderBuffer { .. } => {
                panic!("For now textures with MSAA cannot be updated by a call to `update_texture`")
            }
            TextureType::DefaultFramebuffer => {
                panic!("Cannot update default framebuffer")
            }
        };
        let (pixel_format, _inner_pixel_format, type_) =
            crate::gl_shared::pixel_format_to_gl_format_and_inner_format_and_type(
                texture.pixel_format,
                settings.srgb,
            );
        (self.bind_texture)(target, texture_index);

        if depth > 1 {
            (self.tex_sub_image_3d)(
                target,
                0, /* mip level */
                x as i32,
                y as i32,
                z as i32,
                width as i32,
                height as i32,
                depth as i32,
                pixel_format, // This doesn't necessarily need to match the internal_format
                type_,
                data.as_ptr() as _,
            );
        } else {
            (self.tex_sub_image_2d)(
                target,
                0, /* mip level */
                x as i32,
                y as i32,
                width as i32,
                height as i32,
                pixel_format, // This doesn't necessarily need to match the internal_format
                type_,
                data.as_ptr() as _,
            );
        }

        let minification_filter = minification_filter_to_gl_enum(
            settings.minification_filter,
            settings.mipmap_filter,
            settings.generate_mipmaps,
        );
        let magnification_filter = magnification_filter_to_gl_enum(settings.magnification_filter);

        (self.tex_parameter_i32)(target, TEXTURE_MIN_FILTER, minification_filter as i32);

        (self.tex_parameter_i32)(target, TEXTURE_MAG_FILTER, magnification_filter as i32);

        let wrapping_horizontal = wrapping_to_gl_enum(settings.wrapping_horizontal);
        let wrapping_vertical = wrapping_to_gl_enum(settings.wrapping_vertical);

        (self.tex_parameter_i32)(target, TEXTURE_WRAP_S, wrapping_horizontal as i32);
        (self.tex_parameter_i32)(target, TEXTURE_WRAP_T, wrapping_vertical as i32);

        if settings.generate_mipmaps {
            (self.generate_mipmap)(target);
        }
        self.check_for_error();
    }

    unsafe fn delete_texture(&mut self, texture_inner: TextureInner) {
        unsafe {
            match texture_inner.texture_type {
                TextureType::Texture => (self.delete_textures)(1, &texture_inner.index),
                TextureType::CubeMapFace { .. } => {}
                TextureType::RenderBuffer => {
                    (self.delete_renderbuffers)(1, &texture_inner.index);
                }
                TextureType::DefaultFramebuffer => panic!("Cannot delete default framebuffer"),
            };
        }
    }

    unsafe fn new_cube_map(
        &mut self,
        _width: u32,
        _height: u32,
        pixel_format: PixelFormat,
        _texture_settings: TextureSettings,
    ) -> CubeMapInner {
        unsafe {
            let mut texture_index = 0;
            (self.gen_textures)(1, &mut texture_index);

            CubeMapInner {
                index: texture_index,
                pixel_format,
            }
        }
    }

    unsafe fn update_cube_map(
        &mut self,
        cube_map: &CubeMapInner,
        width: u32,
        height: u32,
        data: &[&[u8]; 6],
        texture_settings: TextureSettings,
    ) {
        let pixel_format = cube_map.pixel_format;
        let (pixel_format, inner_pixel_format, type_) =
            crate::gl_shared::pixel_format_to_gl_format_and_inner_format_and_type(
                pixel_format,
                texture_settings.srgb,
            );
        unsafe {
            (self.bind_texture)(TEXTURE_CUBE_MAP, cube_map.index);
            for i in 0..6 {
                (self.tex_image_2d)(
                    TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,                         /* mip level */
                    inner_pixel_format as i32, // Internal format, how the GPU stores these pixels.
                    width as i32,
                    height as i32,
                    0,            /* border: must be 0 */
                    pixel_format, // This doesn't necessarily need to match the internal_format
                    type_,
                    data[i].as_ptr() as _,
                );
            }

            let minification_filter = minification_filter_to_gl_enum(
                texture_settings.minification_filter,
                texture_settings.mipmap_filter,
                texture_settings.generate_mipmaps,
            );
            let magnification_filter =
                magnification_filter_to_gl_enum(texture_settings.magnification_filter);

            (self.tex_parameter_i32)(
                TEXTURE_CUBE_MAP,
                TEXTURE_MIN_FILTER,
                minification_filter as i32,
            );
            (self.tex_parameter_i32)(
                TEXTURE_CUBE_MAP,
                TEXTURE_MAG_FILTER,
                magnification_filter as i32,
            );

            let wrapping_horizontal = wrapping_to_gl_enum(texture_settings.wrapping_horizontal);
            let wrapping_vertical = wrapping_to_gl_enum(texture_settings.wrapping_vertical);

            (self.tex_parameter_i32)(TEXTURE_CUBE_MAP, TEXTURE_WRAP_S, wrapping_horizontal as i32);
            (self.tex_parameter_i32)(TEXTURE_CUBE_MAP, TEXTURE_WRAP_T, wrapping_vertical as i32);

            if texture_settings.generate_mipmaps {
                (self.generate_mipmap)(TEXTURE_CUBE_MAP);
            }
        }
    }

    unsafe fn delete_cube_map(&mut self, cube_map_inner: CubeMapInner) {
        unsafe { (self.delete_textures)(1, &cube_map_inner.index) }
    }

    unsafe fn new_buffer(&mut self, buffer_usage: BufferUsage, bytes: &[u8]) -> BufferInner {
        unsafe {
            let mut buffer = 0;
            (self.gen_buffers)(1, &mut buffer);
            let gl_buffer_usage = match buffer_usage {
                BufferUsage::Data => ARRAY_BUFFER,
                BufferUsage::Index => ELEMENT_ARRAY_BUFFER,
            };
            (self.bind_buffer)(gl_buffer_usage, buffer);

            (self.buffer_data)(
                gl_buffer_usage,
                bytes.len() as _,
                bytes.as_ptr() as *const std::ffi::c_void,
                STATIC_DRAW,
            );
            (self.bind_buffer)(gl_buffer_usage, 0);

            BufferInner {
                buffer_usage,
                index: buffer,
            }
        }
    }

    unsafe fn delete_buffer(&mut self, buffer_inner: BufferInner) {
        (self.delete_buffers)(1, &buffer_inner.index)
    }
}
