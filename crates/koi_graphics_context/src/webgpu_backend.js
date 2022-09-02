let entry = null;
let device = null;
let context = null;
let presentation_format = null;

var web_gpu_object = {
    async setup() {
        entry = navigator.gpu;
        if (!entry) {
            console.log("WebGPU is *NOT* supported!");
            return null;
        }

        console.log("WebGPU is supported!");

        const adapter = await navigator.gpu.requestAdapter();
        if (!adapter) { return null; }
        device = await adapter.requestDevice();

        let canvas = document.getElementById("canvas");
        context = canvas.getContext('webgpu');
        presentation_format = navigator.gpu.getPreferredCanvasFormat();

        context.configure({
            device: device,
            format: presentation_format,
            alphaMode: 'opaque'
        });

        // TODO: This should be configurable
        /*
        const depthTextureDesc = {
            size: [canvas.width, canvas.height, 1],
            dimension: '2d',
            format: 'depth24plus-stencil8',
            usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
        };
        let depthTexture = device.createTexture(depthTextureDesc);
        let depthTextureView = depthTexture.createView();

        let color_texture = context.getCurrentTexture();
        let color_texture_view = color_texture.createView();
        */

        return 1;
    },
    new_pipeline(vertex_source, fragment_source) {
        const vert_module = device.createShaderModule({ code: vertex_source });
        const frag_module = device.createShaderModule({ code: fragment_source });

        const pipeline = device.createRenderPipeline({
            layout: "auto",
            vertex: {
                module: vert_module,
                entryPoint: "main"
            },
            fragment: {
                module: frag_module,
                entryPoint: "main",
                targets: [
                    {
                        format: presentation_format,
                    },
                ],
            },
            primitive: {
                topology: 'triangle-list',
            },
        });
        return pipeline;
    },
    new_buffer(data_ptr, data_length, usage) {
        // There may be better ways to approach this.
        // See: https://github.com/toji/webgpu-best-practices/blob/main/buffer-uploads.md

        const data = new Uint8Array(self.kwasm_memory.buffer, data_ptr, data_length);
        let buffer = device.createBuffer({
            size: data_length, // Does this need to be aligned?
            usage: usage,
            mappedAtCreation: true
        });
        new Uint8Array(buffer.getMappedRange()).set(data);
        buffer.unmap();
        return buffer;
    },
    new_texture(width, height, depth, format) {
        // TODO: Proper format
        let texture = device.createTexture({
            size: [width, height, depth],
            format: 'rgba8unorm',
            usage:
                GPUTextureUsage.TEXTURE_BINDING |
                GPUTextureUsage.COPY_DST |
                GPUTextureUsage.RENDER_ATTACHMENT,
        });
        return texture;
    },
    update_texture(texture_index, width, height, bytes_per_row, data_ptr, data_length) {
        // TODO: How can we write to a subset of a texture?

        let texture = self.kwasm_get_object(texture_index);
        let data = new Uint8Array(self.kwasm_memory.buffer, data_ptr, data_length);

        // TODO: Allow textures from JS objects
        device.queue.writeTexture(
            { texture: texture },
            data,
            { bytesPerRow: bytes_per_row, rowsPerImage: height },
            { width: width, height: height }
        );
    },
    destroy(item) {
        item.destroy();
    },
    execute_commands() {
        console.log("TODO: Execute commands");
        const commandEncoder = device.createCommandEncoder();
        const textureView = context.getCurrentTexture().createView();

        const renderPassDescriptor = {
            colorAttachments: [
                {
                    view: textureView,
                    clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
                    loadOp: 'clear',
                    storeOp: 'store',
                },
            ],
        };

        const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
        //passEncoder.setPipeline(pipeline);
        //passEncoder.draw(3, 1, 0, 0);
        passEncoder.end();

        device.queue.submit([commandEncoder.finish()]);
    },
};
web_gpu_object