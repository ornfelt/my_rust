extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use std::ffi::CString;
use std::ptr;
use std::str;
use std::mem;
use std::time::Duration;

static VERTEX_SHADER_SRC: &str = "
#version 330 core
layout(location = 0) in vec2 position;
uniform vec2 offset;
void main() {
    gl_Position = vec4(position + offset, 0.0, 1.0);
}";

static FRAGMENT_SHADER_SRC: &str = "
#version 330 core
out vec4 color;
void main() {
    color = vec4(1.0, 0.0, 0.0, 1.0);
}";

fn compile_shader(src: &str, ty: gl::types::GLenum) -> u32 {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for compilation errors
        let mut success: gl::types::GLint = 1;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
            buffer.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buffer.as_mut_ptr() as *mut gl::types::GLchar,
            );

            panic!(
                "Shader compilation failed: {}",
                str::from_utf8(&buffer).ok().expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

fn link_program(vs: u32, fs: u32) -> u32 {
    let program;
    unsafe {
        program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        // Check for linking errors
        let mut success: gl::types::GLint = 1;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: gl::types::GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

            let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
            buffer.set_len((len as usize) - 1); // subtract 1 to skip the trailing null character
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buffer.as_mut_ptr() as *mut gl::types::GLchar,
            );

            panic!(
                "Program linking failed: {}",
                str::from_utf8(&buffer).ok().expect("ProgramInfoLog not valid utf8")
            );
        }
    }
    program
}

fn main() {
    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    // Set OpenGL context attributes
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    // Create a window with an OpenGL context
    let window = video_subsystem
        .window("Rust SDL2 + OpenGL Example", 800, 600)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let _gl = gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    // Define the vertices of the square
    let vertices: [f32; 8] = [
        -0.1, -0.1, // Bottom-left
        0.1, -0.1,  // Bottom-right
        0.1, 0.1,   // Top-right
        -0.1, 0.1,  // Top-left
    ];

    // Generate a vertex buffer object (VBO) and vertex array object (VAO)
    let mut vbo: u32 = 0;
    let mut vao: u32 = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            (2 * mem::size_of::<f32>()) as gl::types::GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    // Compile and link shaders
    let vertex_shader = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER);
    let fragment_shader = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER);
    let shader_program = link_program(vertex_shader, fragment_shader);

    // Set up variables to handle square movement
    let mut offset_x = 0.0;
    let mut offset_y = 0.0;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(Keycode::W), .. } => offset_y += 0.1,
                Event::KeyDown { keycode: Some(Keycode::S), .. } => offset_y -= 0.1,
                Event::KeyDown { keycode: Some(Keycode::A), .. } => offset_x -= 0.1,
                Event::KeyDown { keycode: Some(Keycode::D), .. } => offset_x += 0.1,
                _ => {}
            }
        }

        // Clear the screen
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Use the shader program and bind the VAO
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);

            // Update the offset uniform
            let offset_location = gl::GetUniformLocation(shader_program, CString::new("offset").unwrap().as_ptr());
            gl::Uniform2f(offset_location, offset_x, offset_y);

            // Draw the square
            gl::DrawArrays(gl::TRIANGLE_FAN, 0, 4);

            // Unbind the VAO
            gl::BindVertexArray(0);
        }

        // Swap buffers
        window.gl_swap_window();

        // Control the frame rate
        ::std::thread::sleep(Duration::from_millis(16));
    }

    // Cleanup
    unsafe {
        gl::DeleteProgram(shader_program);
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
