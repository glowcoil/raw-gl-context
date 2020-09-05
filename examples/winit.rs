use raw_gl_context::GlContext;

use raw_window_handle::HasRawWindowHandle;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let context = GlContext::create(window.raw_window_handle()).unwrap();

    context.make_current();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            winit::event::Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            winit::event::Event::RedrawRequested(_) => {
                unsafe {
                    gl::ClearColor(1.0, 0.0, 1.0, 1.0);
                    gl::Clear(gl::COLOR_BUFFER_BIT);
                }

                context.swap_buffers();
            }
            _ => {}
        }
    });
}
