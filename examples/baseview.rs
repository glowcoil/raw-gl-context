use raw_gl_context::GlContext;

use raw_window_handle::HasRawWindowHandle;

use baseview::{Event, Window, WindowHandler, WindowScalePolicy};

struct Example {
    context: GlContext,
}

impl WindowHandler for Example {
    type Message = ();

    fn on_frame(&mut self) {
        self.context.make_current();

        unsafe {
            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.context.swap_buffers();
    }

    fn on_event(&mut self, _window: &mut Window, event: Event) {
        match event {
            Event::Mouse(e) => println!("Mouse event: {:?}", e),
            Event::Keyboard(e) => println!("Keyboard event: {:?}", e),
            Event::Window(e) => println!("Window event: {:?}", e),
        }
    }

    fn on_message(&mut self, _window: &mut Window, _message: Self::Message) {}
}

fn main() {
    let window_open_options = baseview::WindowOpenOptions {
        title: "baseview".into(),
        size: baseview::Size::new(512.0, 512.0),
        scale: WindowScalePolicy::SystemScaleFactor,
        parent: baseview::Parent::None,
    };

    let handle = Window::open(window_open_options, |window| {
        let context = GlContext::create(window.raw_window_handle()).unwrap();
        context.make_current();
        gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

        Example { context }
    });
    handle.1.unwrap().app_run_blocking();
}
