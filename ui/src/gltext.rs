use memoffset::offset_of;
use std::mem::size_of;

use flib::grid::{kGridAtlasSize, text_framgent_shader, text_vertex_shader, GlVertex, Grid};

use glow::{Context, HasContext, NativeProgram, NativeTexture};

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

pub struct GLState {
    program: NativeProgram,
    vert_buffer: glow::NativeBuffer,
    glyph_buffer: glow::NativeBuffer,
    glyph_texture: NativeTexture,
    u_transform: Option<glow::NativeUniformLocation>,
}

impl GLState {
    pub fn create(ctx: &Context, grid: &mut Grid) -> GLState {
        unsafe {
            let vertex_array = ctx
                .create_vertex_array()
                .expect("Cannot create vertex array");
            ctx.bind_vertex_array(Some(vertex_array));

            ctx.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
            ctx.enable(glow::BLEND);

            let vert = text_vertex_shader();
            let frag = text_framgent_shader();
            let program: NativeProgram = ctx.create_program().expect("Cannot create program");
            let shader_sources = [
                (glow::VERTEX_SHADER, vert.as_str()),
                (glow::FRAGMENT_SHADER, frag.as_str()),
            ];

            let shaders = load_shaders(ctx, &program, shader_sources, "#version 330 core");

            ctx.bind_attrib_location(program, 0, "vPosition");
            ctx.bind_attrib_location(program, 1, "vData");
            ctx.bind_attrib_location(program, 2, "vColor");

            ctx.link_program(program);
            if !ctx.get_program_link_status(program) {
                panic!("{}", ctx.get_program_info_log(program));
            }

            for shader in shaders {
                ctx.detach_shader(program, shader);
                ctx.delete_shader(shader);
            }

            ctx.use_program(Some(program));

            let u_grid_atlas = ctx.get_uniform_location(program, "uGridAtlas");
            ctx.uniform_1_i32(u_grid_atlas.as_ref(), 0);

            let u_glyph_data = ctx.get_uniform_location(program, "uGlyphData");
            ctx.uniform_1_i32(u_glyph_data.as_ref(), 1);

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

            let u_transform = ctx.get_uniform_location(program, "uTransform");
            ctx.uniform_matrix_4_f32_slice(u_transform.as_ref(), false, &iden);

            let vert_buffer = {
                let vert_buffer = ctx.create_buffer().unwrap();
                ctx.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
                ctx.buffer_data_size(
                    gl::ARRAY_BUFFER,
                    (grid.verts.capacity() * size_of::<GlVertex>()) as i32,
                    glow::DYNAMIC_DRAW,
                );
                ctx.buffer_sub_data_u8_slice(gl::ARRAY_BUFFER, 0, grid.verts());

                vert_buffer
            };

            let (glyph_buffer, glyph_texture) = {
                let glyph_buffer = ctx.create_buffer().unwrap();
                ctx.bind_buffer(glow::TEXTURE_BUFFER, Some(glyph_buffer));
                grid.glyph_data_buf_id = glyph_buffer.0.into();
                let glyph_texture = ctx.create_texture().unwrap();
                ctx.bind_texture(glow::TEXTURE_BUFFER, Some(glyph_texture));
                grid.glyph_data_buf_tex_id = glyph_texture.0.into();
                // not possible in glow?
                gl::TexBuffer(gl::TEXTURE_BUFFER, gl::RGBA8, grid.glyph_data_buf_id);

                (glyph_buffer, glyph_texture)
            };

            {
                let atlas_texture = ctx.create_texture().unwrap();
                ctx.bind_texture(glow::TEXTURE_BUFFER, Some(atlas_texture));

                ctx.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA8 as i32,
                    kGridAtlasSize.into(),
                    kGridAtlasSize.into(),
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    grid.atlas(),
                );

                ctx.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR as i32,
                );
                ctx.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::LINEAR as i32,
                );
                ctx.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_S,
                    glow::CLAMP_TO_EDGE as i32,
                );
                ctx.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_WRAP_R,
                    glow::CLAMP_TO_EDGE as i32,
                );
                atlas_texture
            };

            GLState {
                program,
                vert_buffer,
                glyph_buffer,
                glyph_texture,
                u_transform,
            }
        }
    }

    pub fn render(&self, ctx: &Context, grid: &Grid, transform: [f32; 16]) {
        unsafe {
            // render

            ctx.clear_color(160.0 / 255.0, 169.0 / 255.0, 175.0 / 255.0, 1.0);
            ctx.clear(glow::COLOR_BUFFER_BIT);

            ctx.use_program(Some(self.program));

            ctx.uniform_matrix_4_f32_slice(self.u_transform.as_ref(), false, &transform);

            ctx.bind_buffer(glow::TEXTURE_BUFFER, Some(self.glyph_buffer));
            ctx.buffer_data_u8_slice(gl::TEXTURE_BUFFER, grid.glyphs(), glow::STREAM_DRAW);

            ctx.active_texture(glow::TEXTURE1);
            ctx.bind_texture(glow::TEXTURE_BUFFER, Some(self.glyph_texture));

            ctx.enable(glow::BLEND);
            ctx.bind_buffer(glow::ARRAY_BUFFER, Some(self.vert_buffer));
            ctx.enable_vertex_attrib_array(0);
            ctx.enable_vertex_attrib_array(1);
            ctx.enable_vertex_attrib_array(2);

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

            ctx.draw_arrays(glow::TRIANGLES, 0, grid.verts.len() as i32);

            ctx.disable_vertex_attrib_array(0);
            ctx.disable_vertex_attrib_array(1);
            ctx.disable_vertex_attrib_array(2);

            ctx.disable(glow::BLEND);
        }
    }
}
