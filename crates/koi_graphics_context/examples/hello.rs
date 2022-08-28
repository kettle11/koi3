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
            r#"layout(location = 0) in vec3 a_position;

    void main()
    {
        gl_Position = vec4(a_position, 1.0);
    }"#,
            r#"
    precision mediump float;

    layout(location = 0) out vec4 color_out;

    uniform vec4 p_custom_color;

    void main()
    {
        color_out = p_custom_color;
    }"#,
            PipelineSettings::default(),
        )
        .unwrap();

    loop {
        let event = events.next().await;
        match event {
            Event::Draw { .. } => {
                let mut command_buffer = g.new_command_buffer();
                command_buffer.push(Command::Clear(kmath::Vec4::new(1.0, 0.0, 0.0, 1.0)));
                command_buffer.push(Command::Present);
                command_buffer.push(Command::SetPipeline(pipeline.clone()));
                command_buffer.push(Command::Draw {
                    triangle_buffer: None,
                    start_triangle: 0,
                    end_triangle: 4,
                    instances: 1,
                });
                g.execute_command_buffer(command_buffer);
                window.request_redraw();
            }
            Event::WindowCloseRequested { .. } => app.quit(),
            _ => {}
        }
    }
}
