use memoffset::offset_of;
use std::mem::size_of;

use flib::grid::{
    create_test_struct, kAtlasChannels, kBezierAtlasSize, kGridAtlasSize, text_framgent_shader,
    text_vertex_shader, GlVertex, Grid,
};
// use glow::{Context, HasContext, NativeProgram};

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
    println!("starting2");
    let width = 1280.0;
    let height = 800.0;
    unsafe {
        let (ctx, shader_version, window, event_loop, gl) = {
            println!("init");
            let event_loop = glutin::event_loop::EventLoop::new();
            let window_builder = glutin::window::WindowBuilder::new()
                .with_title("Hello triangle!")
                .with_inner_size(glutin::dpi::LogicalSize::new(width as f32, height as f32));
            println!("get window");
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap();
            println!("get context");
            let ctx =
                glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);

            println!("get gl");
            let gl = gl::load_with(|s| window.get_proc_address(s) as *const std::os::raw::c_void);
            (ctx, "#version 410", window, event_loop, gl)
        };

        println!("loaded window");

        let mut grid = create_test_struct();
        // let grid = [0_u8, 0];
        gl::Viewport(0, 0, width as i32, height as i32);

        prepare_texture(&ctx, &mut grid);

        //let vertex_array = ctx
        //    .create_vertex_array()
        //    .expect("Cannot create vertex array");
        //ctx.bind_vertex_array(Some(vertex_array));

        //let program = ctx.create_program().expect("Cannot create program");

        //let (vertex_shader_source, fragment_shader_source) = (
        //    r#"const vec2 verts[3] = vec2[3](
        //        vec2(0.5f, 1.0f),
        //        vec2(0.0f, 0.0f),
        //        vec2(1.0f, 0.0f)
        //    );
        //    out vec2 vert;
        //    void main() {
        //        vert = verts[gl_VertexID];
        //        gl_Position = vec4(vert - 0.5, 0.0, 1.0);
        //    }"#,
        //    r#"precision mediump float;
        //    in vec2 vert;
        //    out vec4 color;
        //    void main() {
        //        color = vec4(vert, 0.5, 1.0);
        //    }"#,
        //);

        //let shader_sources = [
        //    (glow::VERTEX_SHADER, vertex_shader_source),
        //    (glow::FRAGMENT_SHADER, fragment_shader_source),
        //];

        //let shaders = load_shaders(&ctx, &program, shader_sources, shader_version);

        //ctx.link_program(program);
        //if !ctx.get_program_link_status(program) {
        //    panic!("{}", ctx.get_program_info_log(program));
        //}

        //for shader in shaders {
        //    ctx.detach_shader(program, shader);
        //    ctx.delete_shader(shader);
        //}

        //ctx.use_program(Some(program));

        //gl::ClearColor(0.0, 0.3, 0.5, 1.0);
        //// ctx.clear_color(1.0, 0.2, 0.3, 1.0);

        //// We handle events differently between targets
        ////
        let iden: [f32; 16] = [
            3.39084e-05,
            0.0,
            0.0,
            0.0,
            0.0,
            5.42535e-05,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -0.9,
            0.6,
            0.0,
            1.0,
        ];
        // render(&grid, iden);

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
                        // ctx.clear(glow::COLOR_BUFFER_BIT);
                        // ctx.draw_arrays(glow::TRIANGLES, 0, 3);

                        render(&grid, iden);

                        window.swap_buffers().unwrap();
                    }
                    Event::WindowEvent { ref event, .. } => match event {
                        WindowEvent::Resized(physical_size) => {
                            window.resize(*physical_size);
                            gl::Viewport(
                                0,
                                0,
                                physical_size.width as i32,
                                physical_size.height as i32,
                            );
                        }
                        WindowEvent::CloseRequested => {
                            // ctx.delete_program(program);
                            // ctx.delete_vertex_array(vertex_array);
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

struct ExposedProgram(u32);

fn prepare_texture(ctx: &Context, grid: &mut Grid) {
    unsafe {
        // let mut vertex_array_id = 0;
        // gl::GenVertexArrays(1, &mut grid.vertex_array_id);
        // gl::BindVertexArray(vertex_array_id);

        let vertex_array = ctx
            .create_vertex_array()
            .expect("Cannot create vertex array");
        ctx.bind_vertex_array(Some(vertex_array));

        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);

        let vert = text_vertex_shader();
        let frag = text_framgent_shader();
        let program: NativeProgram = ctx.create_program().expect("Cannot create program");
        let shader_sources = [
            (glow::VERTEX_SHADER, vert.as_str()),
            (glow::FRAGMENT_SHADER, frag.as_str()),
        ];

        let prog_id: ExposedProgram = unsafe { std::mem::transmute(program) };
        grid.prog_id = prog_id.0;

        let shaders = load_shaders(&ctx, &program, shader_sources, "#version 330 core");

        let name = std::ffi::CString::new("vPosition").expect("CString::new failed");
        gl::BindAttribLocation(grid.prog_id, 0, name.as_ptr());
        let name = std::ffi::CString::new("vData").expect("CString::new failed");
        gl::BindAttribLocation(grid.prog_id, 1, name.as_ptr());
        let name = std::ffi::CString::new("vColor").expect("CString::new failed");
        gl::BindAttribLocation(grid.prog_id, 2, name.as_ptr());

        gl::LinkProgram(grid.prog_id);
        // ctx.link_program(program);
        // if !ctx.get_program_link_status(program) {
        //     panic!("{}", ctx.get_program_info_log(program));
        // }

        for shader in shaders {
            ctx.detach_shader(program, shader);
            ctx.delete_shader(shader);
        }

        ctx.use_program(Some(program));

        // gl::LoadShaderProgram(text_vertex_shader(), text_framgent_shader());
        // gl::UseProgram(3);
        //
        //
        let name = std::ffi::CString::new("uGridAtlas").expect("CString::new failed");
        let u_grid_atlas = gl::GetUniformLocation(grid.prog_id, name.as_ptr());
        let name = std::ffi::CString::new("uGlyphData").expect("CString::new failed");
        let u_glyph_data = gl::GetUniformLocation(grid.prog_id, name.as_ptr());
        let name = std::ffi::CString::new("uTransform").expect("CString::new failed");
        grid.u_transform = gl::GetUniformLocation(grid.prog_id, name.as_ptr()) as i32;

        gl::Uniform1i(u_grid_atlas, 0);
        gl::Uniform1i(u_glyph_data, 1);

        let iden: [f32; 16] = [
            3.39084e-05,
            0.0,
            0.0,
            0.0,
            0.0,
            5.42535e-05,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -0.9,
            0.6,
            0.0,
            1.0,
        ];
        gl::UniformMatrix4fv(grid.u_transform, 1, gl::FALSE, iden.as_ptr());

        let grid_atlas = grid.atlas_ptr();

        gl::GenBuffers(1, &mut grid.vert_buffer_id);
        gl::BindBuffer(gl::ARRAY_BUFFER, grid.vert_buffer_id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (grid.verts.capacity() * size_of::<GlVertex>()) as isize,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
        );
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            (grid.verts.len() * size_of::<GlVertex>()) as isize,
            grid.verts_ptr(),
        );

        gl::GenBuffers(1, &mut grid.glyph_data_buf_id);
        gl::BindBuffer(gl::TEXTURE_BUFFER, grid.glyph_data_buf_id);

        gl::GenTextures(1, &mut grid.glyph_data_buf_tex_id);
        gl::BindTexture(gl::TEXTURE_BUFFER, grid.glyph_data_buf_tex_id);
        gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA8, grid.glyph_data_buf_id);

        gl::GenTextures(1, &mut grid.grid_atlas_id);
        gl::BindTexture(gl::TEXTURE_2D, grid.grid_atlas_id);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as i32,
            kGridAtlasSize.into(),
            kGridAtlasSize.into(),
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            grid_atlas,
        );
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_R, gl::CLAMP_TO_EDGE as i32);
    }
}
fn render(grid: &Grid, transform: [f32; 16]) {
    // println!(
    //     "{} {} {}",
    //     grid.prog_id, grid.glyph_data_buf_id, grid.glyph_data_buf_tex_id
    // );
    unsafe {
        // render

        gl::ClearColor(160.0 / 255.0, 169.0 / 255.0, 175.0 / 255.0, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);

        gl::UseProgram(grid.prog_id);
        gl::UniformMatrix4fv(grid.u_transform, 1, gl::FALSE, transform.as_ptr());

        gl::BindBuffer(gl::TEXTURE_BUFFER, grid.glyph_data_buf_id);
        gl::BufferData(
            gl::TEXTURE_BUFFER,
            (kBezierAtlasSize as usize * kBezierAtlasSize as usize * (kAtlasChannels) as usize)
                as isize,
            grid.glgph_ptr(),
            gl::STREAM_DRAW,
        );

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, grid.grid_atlas_id);
        gl::ActiveTexture(gl::TEXTURE1);
        gl::BindTexture(gl::TEXTURE_BUFFER, grid.glyph_data_buf_tex_id);

        gl::Enable(gl::BLEND);
        gl::BindBuffer(gl::ARRAY_BUFFER, grid.vert_buffer_id);
        gl::EnableVertexAttribArray(0);
        gl::EnableVertexAttribArray(1);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            size_of::<GlVertex>() as i32,
            offset_of!(GlVertex, pos) as *const gl::types::GLvoid,
        );

        gl::VertexAttribIPointer(
            1,
            1,
            gl::UNSIGNED_INT,
            size_of::<GlVertex>() as i32,
            offset_of!(GlVertex, data) as *const gl::types::GLvoid,
        );
        gl::VertexAttribPointer(
            2,
            4,
            gl::UNSIGNED_BYTE,
            gl::TRUE,
            size_of::<GlVertex>() as i32,
            offset_of!(GlVertex, color) as *const gl::types::GLvoid,
        );

        // println!(
        //     "{} {}",
        //     size_of::<GlVertex>() as i32,
        //     offset_of!(GlVertex, pos)
        // );
        // println!(
        //     "{} {}",
        //     size_of::<GlVertex>() as i32,
        //     offset_of!(GlVertex, data)
        // );
        // println!(
        //     "{} {}",
        //     size_of::<GlVertex>() as i32,
        //     offset_of!(GlVertex, color)
        // );

        gl::DrawArrays(gl::TRIANGLES, 0, grid.verts.len() as i32);

        gl::DisableVertexAttribArray(0);
        gl::DisableVertexAttribArray(1);
        gl::DisableVertexAttribArray(2);
        gl::Disable(gl::BLEND);
    }
}
