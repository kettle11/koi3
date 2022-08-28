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
            out vec2 TexCoords;
 
            void main()
            {
                float x = -1.0 + float((gl_VertexID & 1) << 2);
                float y = -1.0 + float((gl_VertexID & 2) << 1);
                TexCoords.x = (x+1.0)*0.5;
                TexCoords.y = (y+1.0)*0.5;
                gl_Position = vec4(x, y, 0, 1);
            }
            "#,
            r#"
            out vec4 color_out;
            void main()
            {
                color_out = vec4(0.0, 0.0, 1.0, 1.0);
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
                command_buffer.push(Command::SetPipeline(pipeline.clone()));
                command_buffer.push(Command::Draw {
                    triangle_buffer: None,
                    triangle_range: 0..1,
                    instances: 1,
                });
                command_buffer.push(Command::Present);

                g.execute_command_buffer(command_buffer);
                window.request_redraw();
            }
            Event::WindowCloseRequested { .. } => app.quit(),
            _ => {}
        }
    }
}
