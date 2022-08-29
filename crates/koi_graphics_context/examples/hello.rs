use kapp::*;
use koi_graphics_context::*;

fn main() {
    let (app, event_loop) = initialize();
    event_loop.run_async(app, run_async);
}

async fn run_async(app: Application, events: Events) {
    let window = app
        .new_window()
        .title("Hello from Koi Graphics")
        .size(800, 800)
        .build();
    window.request_redraw();

    let mut g = GraphicsContext::new(
        GraphicsContextSettings {
            high_resolution_framebuffer: true,
            ..Default::default()
        },
        &window,
    );

    g.new_texture(
        16,
        16,
        1,
        PixelFormat::RGB8Unorm,
        TextureSettings::default(),
    );

    let pipeline = g
        .new_pipeline(
            r#"
            layout(location = 0) in vec4 a_position;

            void main()
            {
                gl_Position = vec4(a_position, 1.0);
            }
            "#,
            r#"
            out vec4 color_out;
            void main()
            {
                color_out = vec4(0.0, 0.0, 1.0, 1.0);
            }"#,
            PipelineSettings {
                faces_to_render: FacesToRender::FrontAndBack,
                depth_test: DepthTest::AlwaysPass,
                ..Default::default()
            },
        )
        .unwrap();

    let position_attribute = pipeline
        .get_vertex_attribute::<[f32; 3]>("a_position")
        .unwrap();
    let positions = g.new_buffer::<[f32; 3]>(
        &[[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]],
        BufferUsage::Data,
    );
    let index_buffer = g.new_buffer(&[[0, 1, 2]], BufferUsage::Index);

    loop {
        let event = events.next().await;
        match event {
            Event::Draw { .. } => {
                let mut command_buffer = g.new_command_buffer();
                {
                    let mut render_pass = command_buffer
                        .begin_render_pass(Some(kmath::Vec4::new(1.0, 0.0, 0.0, 1.0)));
                    render_pass.set_pipeline(pipeline.clone());
                    render_pass
                        .set_vertex_attribute(position_attribute.clone(), Some(positions.clone()));
                    render_pass.draw(Some(index_buffer.clone()), 0..1, 1);
                }

                g.execute_command_buffer(command_buffer);
                window.request_redraw();
            }
            Event::WindowCloseRequested { .. } => app.quit(),
            _ => {}
        }
    }
}
