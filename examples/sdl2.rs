use raw_gl_context::{GlConfig, GlContext};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn main() {
    // Setup SDL
    let sdl2_context = sdl2::init().expect("Failed to initialize sdl2");
    let video_subsystem = sdl2_context
        .video()
        .expect("Failed to create sdl video subsystem");

    // Create the window
    let window = video_subsystem
        .window("Rafx Demo", 900, 600)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .build()
        .expect("Failed to create window");

    let mut event_pump = sdl2_context
        .event_pump()
        .expect("Could not create sdl event pump");

    let gl_context = GlContext::create(&window, GlConfig::default()).unwrap();

    gl_context.make_current();

    gl::load_with(|symbol| gl_context.get_proc_address(symbol) as *const _);

    let now = std::time::Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        gl_context.make_current();

        unsafe {
            gl::ClearColor(now.elapsed().as_secs_f32().sin() * 0.5 + 0.5, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        gl_context.swap_buffers();
        gl_context.make_not_current();
    }
}
