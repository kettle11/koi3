var webgl_object = {
    setup(antialias) {
        canvas = document
            .getElementById("canvas");
        gl =
            canvas.getContext('webgl2', {
                alpha: false,
                desynchronized: false,
                antialias: antialias !== 0,
                depth: true,
                powerPreference: "high-performance"
            });

        if (gl === null) {
            console.log("Could not initialize WebGL2 canvas!");
            // This is probably over tailored to the use cases I'm using `koi` for
            let warning_message = document.getElementById("WebGLSupportMessage");
            if (warning_message) {
                warning_message.style.display = "block";
            }
        }

        function enable_extension(gl, extension) {
            if (!gl.getExtension(extension)) {
                console.log("COULD NOT ENABLE EXTENSION: " + extension);
                return false;
            }
            return true;
        }
        linear_float_filtering_supported = enable_extension(gl, 'OES_texture_float_linear');
        enable_extension(gl, 'OES_texture_float_linear');
        enable_extension(gl, 'EXT_color_buffer_float');

        gl.enable(gl.DEPTH_TEST);
        let vertex_array_object = gl.createVertexArray();
        gl.bindVertexArray(vertex_array_object);
    },
    new_pipeline(vertex_source_in, fragment_source_in) {
        let vertex_source = "#version 300 es\nprecision mediump float;\n" + vertex_source_in;
        let fragment_source = "#version 300 es\nprecision mediump float;\n" + fragment_source_in;

        function create_shader(shader_source, shader_type) {
            let shader = gl.createShader(shader_type);
            gl.shaderSource(shader, shader_source);
            gl.compileShader(shader);
            // These errors should be returned somehow for `kgraphics` to handle
            let message = gl.getShaderInfoLog(shader);
            if (message.length > 0) {
                console.error(message);
            }
            return shader;
        }
        let vertex_shader = create_shader(vertex_source, gl.VERTEX_SHADER);
        let fragment_shader = create_shader(fragment_source, gl.FRAGMENT_SHADER);

        let program = gl.createProgram();
        gl.attachShader(program, vertex_shader);
        gl.attachShader(program, fragment_shader);
        gl.linkProgram(program);

        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            var info = gl.getProgramInfoLog(program);
            console.error('Could not compile WebGL program. \n\n' + info);
            return null;
        } else {
            return program;
        }
    },
    new_texture() {
        let texture = gl.createTexture();
        return texture;
    },
    update_texture(texture_index, target, image_target, inner_pixel_format, width, height, depth, pixel_format, type_, js_data_object, data_ptr, data_length, min, mag, wrapping_horizontal, wrapping_vertical) {
        let data = self.kwasm_get_object(js_data_object);
        if (data_ptr !== 0) {
            if (type_ == gl.FLOAT) {
                // If it's a floating point array
                data = new Float32Array(self.kwasm_memory.buffer, data_ptr, data_length / 4);
            } else {
                data = new Uint8Array(self.kwasm_memory.buffer, data_ptr, data_length);
            }
        }

        if (type_ == gl.FLOAT) {
            // Some Android devices don't support linear filtering of float textures.
            // In those cases fall back to NEAREST filtering.
            if (!linear_float_filtering_supported) {
                min = gl.NEAREST;
                mag = gl.NEAREST;
            }
        }

        let texture = self.kwasm_get_object(texture_index);
        gl.bindTexture(target, texture);

        gl.texImage2D(
            image_target,
            0, /* mip level */
            inner_pixel_format,
            width,
            height,
            0, /* border */
            pixel_format,
            type_,
            data
        );

        gl.texParameteri(
            target,
            gl.TEXTURE_MIN_FILTER,
            min
        );
        gl.texParameteri(
            target,
            gl.TEXTURE_MAG_FILTER,
            mag
        );

        gl.texParameteri(
            target,
            gl.TEXTURE_WRAP_S,
            wrapping_horizontal
        );
        gl.texParameteri(
            target,
            gl.TEXTURE_WRAP_T,
            wrapping_vertical
        );

        /* Border color should be set here too */
    },
    generate_mip_maps(texture_index, texture_type) {
        let texture = self.kwasm_get_object(texture_index);
        gl.bindTexture(texture_type, texture);
        gl.generateMipmap(texture_type);
    },
    new_buffer(data_ptr, data_length, buffer_usage) {
        const data = new Uint8Array(self.kwasm_memory.buffer, data_ptr, data_length);
        let buffer = gl.createBuffer();
        gl.bindBuffer(buffer_usage, buffer);
        gl.bufferData(buffer_usage, data, gl.STATIC_DRAW);
        return buffer;
    },
    delete_buffer(buffer) {
        gl.deleteBuffer(buffer);
    },
    delete_texture(texture) {
        gl.deleteTexture(texture);
    },
    delete_program(program) {
        gl.deleteProgram(proram);
    },
    get_uniform_name_and_type(program_index, uniform_index) {
        let program = self.kwasm_get_object(program_index);
        let active_info = gl.getActiveUniform(program, uniform_index);
        self.kwasm_pass_string_to_client(active_info.name);
        return active_info.type;
    },
    get_uniform_location(program, name) {
        let result = gl.getUniformLocation(program, name);
        return result;
    },
    get_uniform_block_name_and_size(program_index, uniform_block_index) {
        let program = self.kwasm_get_object(program_index);
        let name = gl.getActiveUniformBlockName(program, uniform_block_index);
        let size_bytes = gl.getActiveUniformBlockParameter(program, uniform_block_index, gl.UNIFORM_BLOCK_DATA_SIZE);
        self.kwasm_pass_string_to_client(name);
        return size_bytes;
    },
    uniform_block_binding(program_index, uniform_block_index, binding_point) {
        let program = self.kwasm_get_object(program_index);
        gl.uniformBlockBinding(program, uniform_block_index, binding_point);
    },
    get_program_parameter(program_index, parameter) {
        let program = self.kwasm_get_object(program_index);
        return gl.getProgramParameter(program, parameter);
    },
    get_attribute_name_and_type(program_index, attribute_index) {
        let program = self.kwasm_get_object(program_index);
        let info = gl.getActiveAttrib(program, attribute_index);
        self.kwasm_pass_string_to_client(info.name);
        return info.type;
    },
    get_attribute_location(program, name) {
        let location = gl.getAttribLocation(program, name);
        return location;
    },
    // Stuff used by CommandBuffers
    clear(data_ptr) {
        gl.enable(gl.DEPTH_TEST);
        gl.clearDepth(1.0);

        const data = new Float32Array(self.kwasm_memory.buffer, data_ptr, 4);
        gl.clearColor(data[0], data[1], data[2], data[3]);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    },
    viewport(data_ptr) {
        const data = new Float32Array(self.kwasm_memory.buffer, data_ptr, 4);
        gl.viewport(data[0], data[1], data[2], data[3]);
    },
    set_pipeline(program_index, depth_func, culling, source_blend_factor, destination_blend_factor) {
        let program = self.kwasm_get_object(program_index);

        gl.useProgram(program);
        gl.depthFunc(depth_func);

        if (culling === 0) {
            gl.disable(gl.CULL_FACE);
        } else {
            gl.enable(gl.CULL_FACE);
            gl.cullFace(culling);
        }

        if (source_blend_factor === 0) {
            gl.disable(gl.BLEND);
        } else {
            gl.enable(gl.BLEND);
            gl.blendFunc(source_blend_factor, destination_blend_factor);
        }
    },
    set_uniform_block(uniform_block_index, buffer_index) {
        let buffer = self.kwasm_get_object(buffer_index);
        gl.bindBufferBase(gl.UNIFORM_BUFFER, uniform_block_index, buffer);
    },
    set_uniform_int(location_index, vec_dimensions, array_dimensions, data_ptr, data_length) {
        let location = self.kwasm_get_object(location_index);
        const data = new Int32Array(self.kwasm_memory.buffer, data_ptr, data_length / 4);
        switch (vec_dimensions) {
            case 1:
                if (array_dimensions == 1) {
                    gl.uniform1i(location, data);
                } else {
                    gl.uniform1iv(location, data);
                }
                break;
            case 2:
                gl.uniform2iv(location, data);
                break;
            case 3:
                gl.uniform3iv(location, data);
                break;
            case 4:
                gl.uniform4iv(location, data);
                break;
            default:
                break;
        }
    },
    set_uniform_float(vec_dimensions, location_index, data_ptr, data_length) {
        let location = self.kwasm_get_object(location_index);
        const data = new Float32Array(self.kwasm_memory.buffer, data_ptr, data_length / 4);
        switch (vec_dimensions) {
            case 1:
                gl.uniform1fv(location, data);
                break;
            case 2:
                gl.uniform2fv(location, data);
                break;
            case 3:
                gl.uniform3fv(location, data);
                break;
            case 4:
                gl.uniform4fv(location, data);
                break;
            case 9:
                gl.uniformMatrix3fv(location, data);
                break;
            case 16:
                gl.uniformMatrix4fv(location, data);
                break;
            default:
                break;
        }
    },
    set_texture(texture_unit, texture_type, texture_index) {
        let texture = self.kwasm_get_object(texture_index);
        gl.activeTexture(texture_unit);
        gl.bindTexture(texture_type, texture);
    },
    set_attribute(attribute_index, number_of_components, buffer_index, per_instance) {
        let buffer = kwasm_get_object(buffer_index);

        if (buffer === null) {
            gl.disableVertexAttribArray(attribute_index);
        } else {
            gl.bindBuffer(gl.ARRAY_BUFFER, buffer);

            let len = Math.max(number_of_components / 4, 1);
            for (let i = 0; i < len; i++) {
                gl.vertexAttribPointer(
                    attribute_index + i,               // Index
                    Math.min(number_of_components, 4), // Number of components. It's assumed that components are always 32 bit.
                    gl.FLOAT,
                    false,
                    number_of_components * 4, // 0 means to assume tightly packed
                    i * 16, // Offset
                );

                if (per_instance) {
                    gl.vertexAttribDivisor(attribute_index + i, 1);
                } else {
                    gl.vertexAttribDivisor(attribute_index + i, 0);
                }
                gl.enableVertexAttribArray(attribute_index + i);
            }
        }
    },
    set_attribute_to_constant(attribute_index, data_ptr, data_length) {
        const data = new Float32Array(self.kwasm_memory.buffer, data_ptr, data_length);
        gl.disableVertexAttribArray(attribute_index);
        gl.vertexAttrib4fv(attribute_index, data);
    },
    draw(index_buffer_index, count_vertices, offset_bytes, instances) {
        if (index_buffer_index !== 0) {
            let index_buffer = self.kwasm_get_object(index_buffer_index);
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, index_buffer);

            if (instances <= 1) {
                gl.drawElements(gl.TRIANGLES, count_vertices, gl.UNSIGNED_INT, offset_bytes);
            } else {
                gl.drawElementsInstanced(gl.TRIANGLES, count_vertices, gl.UNSIGNED_INT, offset_bytes, instances);
            }
        } else {
            if (instances <= 1) {
                gl.drawArrays(gl.TRIANGLES, 0, count_vertices);
            } else {
                gl.drawArraysInstanced(gl.TRIANGLES, 0, count_vertices, instances);
            }
        }
    }
};
webgl_object