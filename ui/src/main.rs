use flib::grid::create_test_struct;

use crate::gltext::GLState;

mod gltext;

fn main() {
    let width = 1280.0;
    let height = 800.0;
    unsafe {
        let (ctx, window, event_loop) = {
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
            gl::load_with(|s| window.get_proc_address(s) as *const std::os::raw::c_void);
            // (ctx, "#version 410", window, event_loop, gl)
            (ctx, window, event_loop)
        };

        println!("loaded window");

        let mut grid = create_test_struct();
        // let grid = [0_u8, 0];
        gl::Viewport(0, 0, width as i32, height as i32);

        let state = GLState::create(&ctx, &mut grid);

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
                    Event::LoopDestroyed => {}
                    Event::MainEventsCleared => {
                        window.window().request_redraw();
                    }
                    Event::RedrawRequested(_) => {
                        // ctx.clear(glow::COLOR_BUFFER_BIT);
                        // ctx.draw_arrays(glow::TRIANGLES, 0, 3);

                        state.render(&ctx, &grid, iden);

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
