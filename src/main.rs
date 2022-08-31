mod nes;
mod renderer;

use std::ffi::{c_void, CStr};
use glfw::{Context};

use nes::nes::NES;
use renderer::context;

fn main() {
    let nes = NES::new();
    nes.powerup();

    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    let (mut window, events) = glfw.create_window(800, 800, "Rusty NES Emulator", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW window.");

    window.set_key_polling(true);
    window.make_current();

    let res: i32;
    unsafe 
    {
        res = context::init_opengl(
            &mut glfw as *mut _ as *mut c_void, 
            |glfw, name| (&mut *(glfw as *mut glfw::Glfw)).get_proc_address_raw(CStr::from_ptr(name).to_str().unwrap()));
    }

    if res != 0 {
        eprintln!("Failed to initialize GLAD.");
        return;
    }

    while !window.should_close()
    {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events)
        {
            match event 
            {
                _ => {}
            }
        }

        unsafe { context::clear(); }

        window.swap_buffers();
    }

    return;
}
