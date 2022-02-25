use raw_gl_context::{GlConfig, GlContext};

use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let context = unsafe { GlContext::create(&window, GlConfig::default()).unwrap() };

    unsafe {
        context.make_current();
    }

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let now = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::RedrawRequested(_) => {
                unsafe {
                    context.make_current();
                }

                unsafe {
                    gl::ClearColor(now.elapsed().as_secs_f32().sin() * 0.5 + 0.5, 0.0, 1.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                context.swap_buffers();

                unsafe {
                    context.make_not_current();
                }
            }
            _ => {}
        }
    });
}
