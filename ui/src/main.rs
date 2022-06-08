use flib::grid::kGridAtlasSize;
// use glow::{Context, HasContext, NativeProgram};

// // use glow::*;
// fn load_shaders(
//     gl: &Context,
//     program: &NativeProgram,
//     shader_sources: [(u32, &str); 2],
//     shader_version: &str,
// ) -> Vec<glow::NativeShader> {
//     let mut shaders = Vec::with_capacity(shader_sources.len());

//     unsafe {
//         for (shader_type, shader_source) in shader_sources.iter() {
//             let shader = gl
//                 .create_shader(*shader_type)
//                 .expect("Cannot create shader");
//             gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
//             gl.compile_shader(shader);
//             if !gl.get_shader_compile_status(shader) {
//                 panic!("{}", gl.get_shader_info_log(shader));
//             }
//             gl.attach_shader(*program, shader);
//             shaders.push(shader);
//         }
//     }

//     shaders
// }

// fn prepare_texture(gl: &Context) -> Result<(), String> {
//     unsafe {
//         // let buffer = gl.create_buffer()?;
//         let tex = gl.create_texture()?;
//         gl.bind_texture(glow::TEXTURE_BUFFER, Some(tex));

//     // glBindBuffer(GL_TEXTURE_BUFFER, group.glyphDataBufId);
// 	// glGenTextures(1, &group.glyphDataBufTexId);
// 	// glBindTexture(GL_TEXTURE_BUFFER, group.glyphDataBufTexId);
// 	// glTexBuffer(GL_TEXTURE_BUFFER, GL_RGBA8, group.glyphDataBufId);
//     // gl.tex_bu
//     }
//     Ok(())
// }

// fn loead_vertex_buffer(gl: &Context)->Result<(), String>{
//     unsafe{
//     let buffer = gl.create_buffer()?;
//     gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
//   	// glBindBuffer(GL_ARRAY_BUFFER, label->vertBuffer);
// 	// glEnableVertexAttribArray(0);
// 	// glEnableVertexAttribArray(1);
// 	// glEnableVertexAttribArray(2);
// 	// glVertexAttribPointer(0, 2, GL_FLOAT, GL_FALSE, sizeof(GLLabel::GlyphVertex), (void *)offsetof(GLLabel::GlyphVertex, pos));
// 	// glVertexAttribIPointer(1, 1, GL_UNSIGNED_INT, sizeof(GLLabel::GlyphVertex), (void *)offsetof(GLLabel::GlyphVertex, data));
// 	// glVertexAttribPointer(2, 4, GL_UNSIGNED_BYTE, GL_TRUE, sizeof(GLLabel::GlyphVertex), (void *)offsetof(GLLabel::GlyphVertex, color));
//     }
//     Ok(())
// }

fn main() {
    unsafe {
        let (ctx, shader_version, window, event_loop, gl) = {
            let event_loop = glutin::event_loop::EventLoop::new();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_inner_size(glutin::dpi::LogicalSize::new(1024.0 as f32, 768.0 as f32));
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();
            let ctx =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);

            let gl = gl::load_with(|s| window.get_proc_address(s) as *const std::os::raw::c_void);
            (ctx, "#version 410", window, event_loop, gl)
        };

        prepare_texture();

        let vertex_array = ctx
            .create_vertex_array()
            .expect("Cannot create vertex array");
        ctx.bind_vertex_array(Some(vertex_array));

        let program = ctx.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let shaders = load_shaders(&ctx, &program, shader_sources, shader_version);

        ctx.link_program(program);
        if !ctx.get_program_link_status(program) {
            panic!("{}", ctx.get_program_info_log(program));
        }

        for shader in shaders {
            ctx.detach_shader(program, shader);
            ctx.delete_shader(shader);
        }

        ctx.use_program(Some(program));

        gl::ClearColor(0.0, 0.3, 0.5, 1.0);
        // ctx.clear_color(1.0, 0.2, 0.3, 1.0);

        // We handle events differently between targets

        {
            use glutin::event::{Event, WindowEvent};
            use glutin::event_loop::ControlFlow;

            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;
                match event {
                    Event::LoopDestroyed => {
                        return;
                    }
                    Event::MainEventsCleared => {
                        window.window().request_redraw();
                    }
                    Event::RedrawRequested(_) => {
                        ctx.clear(glow::COLOR_BUFFER_BIT);
                        ctx.draw_arrays(glow::TRIANGLES, 0, 3);
                        window.swap_buffers().unwrap();
                    }
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            window.resize(*physical_size);
                        }
                        WindowEvent::CloseRequested => {
                            ctx.delete_program(program);
                            ctx.delete_vertex_array(vertex_array);
                            *control_flow = ControlFlow::Exit
                        }
                        _ => (),
                    },
                    _ => (),
                }
            });
        }
    }
}
use glow::{Context, HasContext, NativeProgram};

// use glow::*;
fn load_shaders(
    gl: &Context,
    program: &NativeProgram,
    shader_sources: [(u32, &str); 2],
    shader_version: &str,
) -> Vec<glow::NativeShader> {
    let mut shaders = Vec::with_capacity(shader_sources.len());

    unsafe {
        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(*program, shader);
            shaders.push(shader);
        }
    }

    shaders
}

fn prepare_texture() {
    unsafe {
        let mut glyph_data_buf_id = 0;
        gl::GenBuffers(1, &mut glyph_data_buf_id);
        gl::BindBuffer(gl::TEXTURE_BUFFER, glyph_data_buf_id);


        let mut glyph_data_buf_tex_id = 0;
        gl::GenTextures(1, &mut glyph_data_buf_tex_id);
        gl::BindTexture(gl::TEXTURE_BUFFER, glyph_data_buf_tex_id);
	gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA8, glyph_data_buf_id);


	let mut grid_atlas_id = 0;
	gl::GenTextures(1, &mut grid_atlas_id);
	gl::BindTexture(gl::TEXTURE_2D, grid_atlas_id);
	gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA8 as i32, kGridAtlasSize.into(), kGridAtlasSize.into(), 0, gl::RGBA, gl::UNSIGNED_BYTE, gridAtlas);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
	gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
    }
}
