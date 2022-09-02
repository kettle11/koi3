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
    destroy() {

    },
    execute_commands() {
        
    }
};
webgl_object