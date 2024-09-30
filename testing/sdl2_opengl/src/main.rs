extern crate gl;
extern crate sdl2;

use gl::types::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::ffi::{CStr, CString};
use std::ptr;
use std::str;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const WIN_WIDTH: u32 = 800;
const WIN_HEIGHT: u32 = 600;

static VERTEX_SHADER_SRC: &str = "
    #version 330 core
    layout(location = 0) in vec2 position;
    uniform vec2 offset;
    void main() {
        gl_Position = vec4(position + offset, 0.0, 1.0);
    }
";

static FRAGMENT_SHADER_SRC: &str = "
    #version 330 core
    out vec4 color;
    uniform vec4 rectColor;
    void main() {
        color = rectColor;
    }
";

static OBSTACLE_VERTEX_SHADER_SRC: &str = "
    #version 330 core
    layout(location = 0) in vec2 position;
    uniform vec2 offset;
    void main() {
        gl_Position = vec4(position + offset, 0.0, 1.0);
    }
";

static OBSTACLE_FRAGMENT_SHADER_SRC: &str = "
    #version 330 core
    out vec4 color;
    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
";

fn check_shader_compile_status(shader: GLuint) {
    let mut success = gl::FALSE as GLint;
    unsafe {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    }
    if success == gl::FALSE as GLint {
        let mut info_log = Vec::with_capacity(512);
        unsafe {
            info_log.set_len(511);
            gl::GetShaderInfoLog(shader, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
        }
        panic!(
            "ERROR::SHADER::COMPILATION_FAILED\n{}",
            str::from_utf8(&info_log).unwrap()
        );
    }
}

fn check_program_link_status(program: GLuint) {
    let mut success = gl::FALSE as GLint;
    unsafe {
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    }
    if success == gl::FALSE as GLint {
        let mut info_log = Vec::with_capacity(512);
        unsafe {
            info_log.set_len(511);
            gl::GetProgramInfoLog(program, 512, ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
        }
        panic!(
            "ERROR::PROGRAM::LINKING_FAILED\n{}",
            str::from_utf8(&info_log).unwrap()
        );
    }
}

fn check_collision(rect_x: f32, rect_y: f32, tri_x: f32, tri_y: f32, tri_size: f32) -> bool {
    let half_size = 0.1;
    rect_x + half_size > tri_x - tri_size
        && rect_x - half_size < tri_x + tri_size
        && rect_y + half_size > tri_y - tri_size
        && rect_y - half_size < tri_y + tri_size
}

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();

    let window = video_subsystem
        .window("SDL2 + OpenGL in Rust", WIN_WIDTH, WIN_HEIGHT)
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const _);

    let vertex_shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    let fragment_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };
    let obstacle_vertex_shader = unsafe { gl::CreateShader(gl::VERTEX_SHADER) };
    let obstacle_fragment_shader = unsafe { gl::CreateShader(gl::FRAGMENT_SHADER) };

    unsafe {
        let c_str_vert = CString::new(VERTEX_SHADER_SRC.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);
        check_shader_compile_status(vertex_shader);

        let c_str_frag = CString::new(FRAGMENT_SHADER_SRC.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);
        check_shader_compile_status(fragment_shader);

        let c_str_obst_vert = CString::new(OBSTACLE_VERTEX_SHADER_SRC.as_bytes()).unwrap();
        gl::ShaderSource(obstacle_vertex_shader, 1, &c_str_obst_vert.as_ptr(), ptr::null());
        gl::CompileShader(obstacle_vertex_shader);
        check_shader_compile_status(obstacle_vertex_shader);

        let c_str_obst_frag = CString::new(OBSTACLE_FRAGMENT_SHADER_SRC.as_bytes()).unwrap();
        gl::ShaderSource(obstacle_fragment_shader, 1, &c_str_obst_frag.as_ptr(), ptr::null());
        gl::CompileShader(obstacle_fragment_shader);
        check_shader_compile_status(obstacle_fragment_shader);
    }

    let shader_program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);
        check_program_link_status(shader_program);
    }

    let obstacle_shader_program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(obstacle_shader_program, obstacle_vertex_shader);
        gl::AttachShader(obstacle_shader_program, obstacle_fragment_shader);
        gl::LinkProgram(obstacle_shader_program);
        check_program_link_status(obstacle_shader_program);
    }

    unsafe {
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        gl::DeleteShader(obstacle_vertex_shader);
        gl::DeleteShader(obstacle_fragment_shader);
    }

    let vertices: [f32; 8] = [
        -0.1, -0.1, 0.1, -0.1, 0.1, 0.1, -0.1, 0.1,
    ];
    let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

    let mut vao: GLuint = 0;
    let mut vbo: GLuint = 0;
    let mut ebo: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indices.len() * std::mem::size_of::<GLuint>()) as GLsizeiptr,
            indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    let triangle_vertices: [f32; 6] = [
        0.0, 0.1, -0.1, -0.1, 0.1, -0.1,
    ];

    let mut triangle_vao: GLuint = 0;
    let mut triangle_vbo: GLuint = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut triangle_vao);
        gl::GenBuffers(1, &mut triangle_vbo);

        gl::BindVertexArray(triangle_vao);

        gl::BindBuffer(gl::ARRAY_BUFFER, triangle_vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (triangle_vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            triangle_vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 2 * std::mem::size_of::<GLfloat>() as GLsizei, ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    let mut event_pump = sdl.event_pump().unwrap();
    let mut running = true;

    let mut x_offset: f32 = 0.0;
    let mut y_offset: f32 = 0.0;
    let move_speed: f32 = 0.01;

    let mut triangle_x: f32 = (rand::random::<f32>() * 2.0) - 1.0;
    let mut triangle_y: f32 = (rand::random::<f32>() * 2.0) - 1.0;
    let triangle_move_speed: f32 = 0.005;

    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => running = false,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => running = false,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::W) => y_offset += move_speed,
                    Some(Keycode::S) => y_offset -= move_speed,
                    Some(Keycode::A) => x_offset -= move_speed,
                    Some(Keycode::D) => x_offset += move_speed,
                    _ => (),
                },
                _ => (),
            }
        }

        let is_colliding = check_collision(x_offset, y_offset, triangle_x, triangle_y, 0.1);

        let rect_color: [f32; 4] = if is_colliding {
            [1.0, 0.0, 0.0, 1.0]
        } else {
            [0.0, 1.0, 0.0, 1.0]
        };

        if is_colliding {
            match event_pump.keyboard_state().pressed_scancodes().next() {
                Some(sdl2::keyboard::Scancode::W) => y_offset -= move_speed,
                Some(sdl2::keyboard::Scancode::S) => y_offset += move_speed,
                Some(sdl2::keyboard::Scancode::A) => x_offset += move_speed,
                Some(sdl2::keyboard::Scancode::D) => x_offset -= move_speed,
                _ => (),
            }
        }

        triangle_x += (rand::random::<f32>() * 2.0 - 1.0) * triangle_move_speed;
        triangle_y += (rand::random::<f32>() * 2.0 - 1.0) * triangle_move_speed;

        if triangle_x > 1.0 || triangle_x < -1.0 {
            triangle_x = 0.0;
        }
        if triangle_y > 1.0 || triangle_y < -1.0 {
            triangle_y = 0.0;
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(shader_program);
            let offset_location = gl::GetUniformLocation(shader_program, CString::new("offset").unwrap().as_ptr());
            gl::Uniform2f(offset_location, x_offset, y_offset);
            let color_location = gl::GetUniformLocation(shader_program, CString::new("rectColor").unwrap().as_ptr());
            gl::Uniform4fv(color_location, 1, rect_color.as_ptr());

            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
            gl::BindVertexArray(0);

            gl::UseProgram(obstacle_shader_program);
            let triangle_offset_location = gl::GetUniformLocation(obstacle_shader_program, CString::new("offset").unwrap().as_ptr());
            gl::Uniform2f(triangle_offset_location, triangle_x, triangle_y);

            gl::BindVertexArray(triangle_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
        }

        window.gl_swap_window();
    }

    unsafe {
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ebo);
        gl::DeleteVertexArrays(1, &triangle_vao);
        gl::DeleteBuffers(1, &triangle_vbo);
        gl::DeleteProgram(shader_program);
        gl::DeleteProgram(obstacle_shader_program);
    }
}
